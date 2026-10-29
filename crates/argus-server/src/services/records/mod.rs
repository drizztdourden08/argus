//! The records of the local remote (`docs/local-remote.md`): the release store and its update
//! feed, Brock's check runs and the Overlord's local package registry. The stores are read
//! only; a folder watch keeps each index fresh. The service answers the routes of
//! `routes::records`, `routes::feed` and `routes::checks`.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use argus_records::checks::CheckStore;
use argus_records::folder_watch::FolderWatch;
use argus_records::registry::Registry;
use argus_records::releases::ReleaseStore;
use serde_json::Value;

use crate::adapters::records::DbIds;
use crate::{ServeOptions, Services};

mod consumers;
mod pr_checks;

pub use consumers::{ConsumerStatus, Consumers, NodeStatus};

use consumers::ConsumerCache;

/// The default place of the Overlord's tool.
const DEFAULT_SCRIPT: &str = "X:/ai-config/shared/tools/localpkg/localpkg.mjs";

/// Where the list of consumers of the local packages comes from.
#[derive(Clone, Default)]
pub enum ConsumerSource {
    /// `node localpkg.mjs status --json` (the `localpkg` setting of `config.json` names the
    /// `script` and the `node` program).
    #[default]
    Node,
    /// Not asked: the list is empty.
    Off,
    /// Another source (a test).
    Custom(Arc<dyn ConsumerStatus>),
}

/// How the records service starts; the fields after `enabled` are seams for tests.
#[derive(Clone)]
pub struct RecordsOptions {
    /// Run the service.
    pub enabled: bool,
    /// Where the consumers of the local packages are asked.
    pub consumers: ConsumerSource,
    /// Another `localpkg` home than `LOCALPKG_HOME` or `<COORD_HOME>/localpkg`.
    pub localpkg_home: Option<PathBuf>,
}

impl Default for RecordsOptions {
    fn default() -> Self {
        RecordsOptions {
            enabled: true,
            consumers: ConsumerSource::Node,
            localpkg_home: None,
        }
    }
}

/// The stores and the watches over their folders.
pub struct Records {
    /// The release store (`<COORD_HOME>/releases`).
    pub releases: Arc<ReleaseStore>,
    /// The check runs (`<COORD_HOME>/ci`).
    pub checks: Arc<CheckStore>,
    /// The local package builds.
    pub registry: Registry,
    pub(crate) consumers: ConsumerCache,
    watches: Mutex<Vec<FolderWatch>>,
}

fn localpkg_home(svc: &Services, opts: &RecordsOptions) -> PathBuf {
    opts.localpkg_home
        .clone()
        .or_else(|| {
            let env = std::env::var_os("LOCALPKG_HOME").filter(|v| !v.is_empty());
            env.map(PathBuf::from)
        })
        .unwrap_or_else(|| svc.paths.coord_home.join("localpkg"))
}

fn node_status(svc: &Services, home: &std::path::Path) -> Arc<dyn ConsumerStatus> {
    let setting = svc
        .config()
        .extra
        .get("localpkg")
        .cloned()
        .unwrap_or(Value::Null);
    let text = |key: &str, default: &str| {
        setting[key]
            .as_str()
            .filter(|s| !s.is_empty())
            .unwrap_or(default)
            .to_owned()
    };
    Arc::new(NodeStatus {
        node: text("node", "node"),
        script: PathBuf::from(text("script", DEFAULT_SCRIPT)),
        home: home.to_path_buf(),
    })
}

pub(crate) fn build(svc: &Services, opts: &ServeOptions) -> Option<Arc<Records>> {
    let options = &opts.records;
    let home = localpkg_home(svc, options);
    let source = match &options.consumers {
        ConsumerSource::Node => Some(node_status(svc, &home)),
        ConsumerSource::Off => None,
        ConsumerSource::Custom(source) => Some(Arc::clone(source)),
    };
    let ids = Arc::new(DbIds::new(Arc::clone(&svc.db)));
    options.enabled.then(|| {
        Arc::new(Records {
            releases: Arc::new(ReleaseStore::new(&svc.paths.coord_home.join("releases"))),
            checks: Arc::new(CheckStore::new(&svc.paths.coord_home.join("ci"), ids)),
            registry: Registry::new(&home),
            consumers: ConsumerCache::new(source),
            watches: Mutex::default(),
        })
    })
}

/// Reads both folders once, then watches them.
pub(crate) async fn start(svc: &Arc<Services>) {
    let Some(records) = svc.records.clone() else {
        return;
    };
    let first = Arc::clone(&records);
    let _ = tokio::task::spawn_blocking(move || {
        first.releases.rescan();
        first.checks.rescan();
    })
    .await;
    let (releases, checks) = (Arc::clone(&records.releases), Arc::clone(&records.checks));
    let watches = vec![
        FolderWatch::start(
            releases.root(),
            Arc::new({
                let store = Arc::clone(&releases);
                move || store.rescan()
            }),
        ),
        FolderWatch::start(checks.root(), Arc::new(on_checks_changed(svc, &checks))),
    ];
    *records
        .watches
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = watches;
    tokio::spawn(pr_checks::fill(Arc::clone(svc)));
}

/// What the watch of the check folder does: reads it again, then brings the check lists of the
/// pull requests in line. It holds the services weakly, so the watch never keeps them alive.
fn on_checks_changed(
    svc: &Arc<Services>,
    checks: &Arc<CheckStore>,
) -> impl Fn() + Send + Sync + 'static {
    let (store, weak) = (Arc::clone(checks), Arc::downgrade(svc));
    let runtime = tokio::runtime::Handle::current();
    move || {
        store.rescan();
        if let Some(svc) = weak.upgrade() {
            runtime.spawn(pr_checks::fill(svc));
        }
    }
}

pub(crate) fn stop(svc: &Services) {
    let Some(records) = &svc.records else {
        return;
    };
    let watches = std::mem::take(
        &mut *records
            .watches
            .lock()
            .unwrap_or_else(PoisonError::into_inner),
    );
    for watch in watches {
        watch.stop();
    }
}
