//! A server whose worktree scanner looks at a throwaway repository, with timers on and no real
//! process list, and the calls the worktree tests make over and over.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use argus_core::clock::iso_from_ms;
use argus_core::clock::now_ms;
use argus_server::CacheOptions;
use argus_testkit::{Reply, Request, TestOptions, TestServer, alpha_board, make_home, write_board};
use argus_worktrees::{GitExec, ProcInfo, ProcessLister, clean_path, worktree_id};
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::fixture::Fixture;

pub use crate::wait::{until, wakes_of};

/// A machine with no process on it: a removal never finds its folder in use.
pub struct NoProcs;

impl ProcessLister for NoProcs {
    fn list(&self) -> Result<Vec<ProcInfo>, String> {
        Ok(Vec::new())
    }
}

#[derive(Clone, Default)]
pub struct Setup {
    pub shadow: bool,
    pub enabled_off: bool,
    pub git: Option<Arc<dyn GitExec>>,
    /// A build cache folder to serve, with the prune timer off.
    pub cache_root: Option<PathBuf>,
}

pub struct Rig {
    pub t: TestServer,
    pub fx: Fixture,
    setup: Setup,
    home: TempDir,
    dash: TempDir,
}

/// The id of a worktree folder.
pub fn id_of(path: &std::path::Path) -> String {
    worktree_id(&path.display().to_string(), cfg!(windows))
}

pub fn rule(fx: &Fixture) -> Value {
    json!({ "id": "W-1", "path": clean_path(&fx.wts.display().to_string()), "kind": "folder", "include": [], "exclude": [], "thread": "ui", "note": "the ui thread builds here", "staleDays": null, "addedBy": "ui", "addedAt": iso_from_ms(now_ms()) })
}

/// A server on these homes with the scanner's timers on.
async fn serve(setup: &Setup, home: &Path, dash: &Path) -> TestServer {
    let Setup {
        shadow,
        enabled_off,
        git,
        cache_root,
    } = setup.clone();
    TestServer::start(TestOptions {
        shadow,
        coord_home: Some(home.to_path_buf()),
        dashboard_home: Some(dash.to_path_buf()),
        config: json!({ "worktrees": { "startDelayMs": 300, "liveMs": 1000 } }),
        tweak: Some(Arc::new(move |opts| {
            opts.worktrees.timers = true;
            opts.worktrees.enabled = !enabled_off;
            opts.worktrees.procs = Some(Arc::new(NoProcs));
            opts.worktrees.git.clone_from(&git);
            opts.cache = CacheOptions {
                enabled: cache_root.is_some(),
                timers: false,
                root: cache_root.clone(),
            };
        })),
        ..TestOptions::default()
    })
    .await
}

impl Rig {
    pub async fn start() -> Rig {
        Rig::with(Setup::default()).await
    }

    pub async fn with(setup: Setup) -> Rig {
        let fx = Fixture::new();
        let home = tempfile::tempdir().expect("a coord home");
        let dash = tempfile::tempdir().expect("a dashboard home");
        make_home(home.path());
        let mut board = alpha_board(home.path());
        board["worktreeRules"] = json!([rule(&fx)]);
        board["counters"]["worktreeRule"] = json!(1);
        write_board(home.path(), "alpha", &board);
        let t = serve(&setup, home.path(), dash.path()).await;
        Rig {
            t,
            fx,
            setup,
            home,
            dash,
        }
    }

    /// The server stopped and started again on the same homes (a restart of the owner's server).
    pub async fn restart(self) -> Rig {
        let Rig {
            t,
            fx,
            setup,
            home,
            dash,
        } = self;
        t.stop(true).await;
        let t = serve(&setup, home.path(), dash.path()).await;
        Rig {
            t,
            fx,
            setup,
            home,
            dash,
        }
    }

    /// The docs of the Worktrees tab by folder name.
    pub async fn docs(&self) -> serde_json::Map<String, Value> {
        let list = self.t.get("/api/p/alpha/db/worktrees").await.json();
        let rows = list["docs"].as_array().cloned().unwrap_or_default();
        rows.iter()
            .filter_map(|d| Some((d["data"]["name"].as_str()?.to_owned(), d["data"].clone())))
            .collect()
    }

    pub async fn meta(&self) -> Value {
        self.t.get("/api/p/alpha/db/meta/worktrees").await.json()["data"].clone()
    }

    /// Waits for the first scan.
    pub async fn scanned(&self) -> Value {
        until("the first scan", || async {
            let meta = self.meta().await;
            meta["scannedAt"].is_string().then_some(meta)
        })
        .await
    }

    /// A call as the owner's page makes it.
    pub async fn page(&self, path: &str, body: &Value) -> Reply {
        self.t
            .send(self.t.page(Request::new("POST", path).json(body)))
            .await
    }

    /// A call as a thread of the CLI makes it.
    pub async fn cli(&self, path: &str, body: &Value) -> Reply {
        self.t.write("POST", path, body).await
    }

    pub async fn activity(&self, kind: &str) -> Vec<String> {
        let rows = self
            .t
            .get(&format!("/api/activity?kind={kind}"))
            .await
            .json();
        let list = rows["activity"].as_array().cloned().unwrap_or_default();
        list.iter()
            .filter_map(|a| a["text"].as_str().map(str::to_owned))
            .collect()
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.fx.path[name].clone()
    }
}
