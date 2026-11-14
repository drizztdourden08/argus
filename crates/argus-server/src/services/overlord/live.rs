//! The hosted Overlord's child, started with Argus when `config.overlord.mode` is `live` (the
//! default is `off`, so nothing starts). While a child runs, a `HostedSink` is attached to the wake
//! engine for the project `portfolio` and the thread `coordinator`, so its wakes go to the child's
//! stdin. When the child dies the sink is detached and the wakes stay pending until the next start.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use argus_accounts::alerts::local_offset_seconds;
use argus_accounts::{find_app_data, find_cli_exe};
use argus_core::clock::now_ms;
use argus_core::log::Logger;
use argus_overlord::ports::{ChildSpec, HostFault, SecretStore, SpendLedger};
use argus_overlord::session::args::{SessionArgs, build_argv};
use argus_overlord::session::env::build_env;
use argus_overlord::session::guard::GuardFault;
use argus_overlord::session::supervisor::{self, Config, Link, Observer, SpecFactory};
use argus_overlord::spend::{left_usd, month_key};
use argus_store::Db;
use argus_wake::engine::{AttachOptions, ListenerId};
use argus_wake::hosted::HostedSink;
use argus_wake::service::driver::{SharedWake, lock};
use serde_json::json;

use super::config::{self, Live};
use crate::Services;
use crate::adapters::overlord::gate::ApprovalGate;
use crate::adapters::overlord::host::SystemChildHost;
use crate::adapters::overlord::key::DpapiKeyStore;
use crate::adapters::overlord::spend::SqlLedger;

const SCOPE: &str = "portfolio";
const THREAD: &str = "coordinator";
const LABEL: &str = "hosted overlord";

/// Attaches the wake sink while a child runs and detaches it when the child is gone.
struct Wakes {
    wake: SharedWake,
    log: Arc<Logger>,
    gate: Arc<ApprovalGate>,
    attached: Mutex<Option<ListenerId>>,
}

impl Observer for Wakes {
    fn started(&self, link: &Link) {
        let link = link.clone();
        let sink = HostedSink::new(Box::new(move |text| link.send(text)));
        let options = AttachOptions {
            v: 2,
            label: Some(LABEL.to_owned()),
        };
        let attached = lock(&self.wake).attach(SCOPE, THREAD, Box::new(sink), options);
        *self.attached.lock().unwrap_or_else(PoisonError::into_inner) = Some(attached.id);
    }

    fn stopped(&self) {
        self.gate.end();
        let id = self
            .attached
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(id) = id {
            lock(&self.wake).detach(&id);
        }
    }

    fn guard_failed(&self, fault: &GuardFault) {
        self.log.warn(
            "hosted overlord ended",
            json!({ "fault": fault.to_string() }),
        );
    }
}

/// What a child is started from, kept for every restart.
struct Inputs {
    home: PathBuf,
    db: Arc<Db>,
    exe: PathBuf,
    budget_usd: f64,
    live: Live,
    gate: Arc<ApprovalGate>,
    /// The endpoint of the private MCP server, `http://127.0.0.1:<port>/mcp/overlord`.
    endpoint: String,
}

fn find_exe() -> PathBuf {
    find_app_data(None)
        .and_then(|app| find_cli_exe(&app))
        .unwrap_or_else(|| PathBuf::from("claude"))
}

fn budget_left(inputs: &Inputs) -> f64 {
    let month = month_key(now_ms(), local_offset_seconds());
    let spent = SqlLedger::open(Arc::clone(&inputs.db))
        .and_then(|ledger| ledger.month_total(&month))
        .unwrap_or(0.0);
    left_usd(spent, inputs.budget_usd)
}

/// The next child: the key is read now, so a replaced key takes effect at the next start. A fault
/// names a reason and never the key.
fn spec_of(inputs: &Inputs, resume: Option<&str>) -> Result<ChildSpec, HostFault> {
    let key = DpapiKeyStore::new(&inputs.home)
        .load()
        .map_err(|fault| HostFault(fault.to_string()))?
        .ok_or_else(|| HostFault("no API key is saved".to_owned()))?;
    let dir = inputs.home.join("overlord");
    std::fs::create_dir_all(&dir)
        .map_err(|_| HostFault("the overlord folder cannot be made".to_owned()))?;
    let config = inputs.gate.begin(&inputs.endpoint, &std::env::temp_dir())?;
    let args = build_argv(&SessionArgs {
        model: inputs.live.model.clone(),
        effort: None,
        resume: resume.map(str::to_owned),
        fork: false,
        mcp_config: config.to_string_lossy().into_owned(),
        budget_left_usd: budget_left(inputs),
    });
    Ok(ChildSpec {
        program: inputs.exe.clone(),
        args,
        env: build_env(std::env::vars(), &key, inputs.live.approval_timeout_min),
        cwd: inputs
            .live
            .cwd
            .clone()
            .unwrap_or_else(|| inputs.home.clone()),
        stderr_log: dir.join("child-stderr.log"),
    })
}

fn spec_factory(svc: &Services, live: &Live) -> SpecFactory {
    let inputs = Inputs {
        home: svc.paths.home.clone(),
        db: Arc::clone(&svc.db),
        exe: live.claude_exe.clone().unwrap_or_else(find_exe),
        budget_usd: config::budget(svc).budget_usd,
        live: live.clone(),
        gate: Arc::clone(&svc.overlord_approvals),
        endpoint: format!("http://127.0.0.1:{}/mcp/overlord", svc.port()),
    };
    Box::new(move |resume| spec_of(&inputs, resume))
}

/// Starts the supervisor when the mode is `live` and the wake engine is on.
pub(crate) fn start(svc: &Arc<Services>) {
    if svc.shadow || !config::is_live(svc) {
        return;
    }
    let Some(wake) = svc.wake.clone() else {
        return;
    };
    let live = config::live(svc);
    let observer = Arc::new(Wakes {
        wake,
        log: Arc::clone(&svc.log),
        gate: Arc::clone(&svc.overlord_approvals),
        attached: Mutex::new(None),
    });
    let setup = Config {
        spec: spec_factory(svc, &live),
        restart_max: live.restart_max,
        resume_from: None,
    };
    let handle = supervisor::start(Arc::new(SystemChildHost::default()), setup, observer);
    let _ = svc.overlord_link.set(handle.link);
    svc.tasks.keep("overlord", handle.task);
}

/// Asks the supervisor to end the child by pid.
pub(crate) fn stop(svc: &Services) {
    if let Some(link) = svc.overlord_link.get() {
        link.stop();
    }
}
