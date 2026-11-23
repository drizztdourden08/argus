//! The hosted Overlord's child, started with Argus when `config.overlord.mode` is `live` (a missing
//! mode is live) or `shadow` and a key is saved; `autostart` false keeps it down at boot, and the
//! keeper (`keeper.rs`) starts it later when it is not up. A shadow
//! child is the read-only copy of `shadow.rs`. While a child runs, a `HostedSink` is attached to the wake
//! engine for the project `portfolio` and the thread `coordinator`, so its wakes go to the child's
//! stdin. When the child dies the sink is detached and the wakes stay pending until the next start.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use argus_accounts::alerts::local_offset_seconds;
use argus_accounts::{find_app_data, find_cli_exe};
use argus_core::clock::now_ms;
use argus_core::log::Logger;
use argus_overlord::ports::{ChildSpec, HostFault, SecretStore, SpendLedger};
use argus_overlord::protocol::{Event as Wire, decode_line};
use argus_overlord::session::args::{SessionArgs, build_argv};
use argus_overlord::session::env::build_env;
use argus_overlord::session::guard::GuardFault;
use argus_overlord::session::supervisor::{self, Config, Link, Observer, SpecFactory};
use argus_overlord::shadow::shadow_args;
use argus_overlord::spend::{left_usd, month_key};
use argus_overlord::switch::live_intro;
use argus_store::Db;
use argus_wake::engine::AttachOptions;
use argus_wake::hosted::HostedSink;
use argus_wake::service::driver::SharedWake;
use serde_json::json;

use super::attachment::Attachment;
use super::config::{self, Live};
use super::live_session;
use super::shadow::{self, Shadow, ShadowInput};
use super::switch_board::handover_file;
use crate::Services;
use crate::adapters::overlord::gate::ApprovalGate;
use crate::adapters::overlord::host::SystemChildHost;
use crate::adapters::overlord::key::DpapiKeyStore;
use crate::adapters::overlord::spend::SqlLedger;

const THREAD: &str = "coordinator";
const LABEL: &str = "hosted overlord";

/// Attaches the wake sink while a child runs and detaches it when the child is gone.
struct Wakes {
    log: Arc<Logger>,
    hold: Attachment,
    /// The first message, sent at the first start only (a restart keeps its session).
    intro: Mutex<Option<String>>,
    db: Arc<Db>,
    /// A saved session was resumed and the child has not reported its id yet.
    unproven: AtomicBool,
}

impl Observer for Wakes {
    fn line(&self, line: &str) {
        if let Ok(Wire::Init(init)) = decode_line(line) {
            live_session::save(&self.db, &init.session_id);
            self.unproven.store(false, Ordering::SeqCst);
        }
    }

    fn started(&self, link: &Link) {
        let intro = self
            .intro
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(intro) = intro {
            let _ = link.send(intro);
        }
        let link = link.clone();
        let sink = HostedSink::new(Box::new(move |text| link.send(text)));
        let options = AttachOptions {
            v: 2,
            label: Some(LABEL.to_owned()),
        };
        self.hold.attach(THREAD, sink, options);
    }

    fn stopped(&self) {
        self.hold.end();
        if self.unproven.swap(false, Ordering::SeqCst) {
            live_session::clear(&self.db);
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
    budget_usd: Option<f64>,
    live: Live,
    gate: Arc<ApprovalGate>,
    /// The endpoint of the private MCP server, `http://127.0.0.1:<port>/mcp/overlord`.
    endpoint: String,
    /// Set in shadow mode: the child forks the desktop session and may only read.
    shadow: Option<ShadowInput>,
    /// Set at a switch-over with no shadow session: the live child forks this desktop session.
    fork_from: Option<String>,
}

/// Where a live child's first session comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Origin {
    /// A new session: a boot or the owner's start.
    Fresh,
    /// The shadow's session, resumed at the switch-over.
    Resume(String),
    /// A fork of the desktop session, at a switch-over with no shadow session.
    Fork(String),
}

/// The argument line: the shadow's, a fork of the desktop session at its first start, or the plain
/// live one.
fn args_of(inputs: &Inputs, base: &SessionArgs) -> Result<Vec<String>, HostFault> {
    match (&inputs.shadow, &inputs.fork_from) {
        (Some(shadow), _) => shadow::argv(shadow, base),
        (None, Some(desktop)) => Ok(build_argv(&shadow_args(base, desktop))),
        (None, None) => Ok(build_argv(base)),
    }
}

fn find_exe() -> PathBuf {
    find_app_data(None)
        .and_then(|app| find_cli_exe(&app))
        .unwrap_or_else(|| PathBuf::from("claude"))
}

/// The money left of this month's budget, for a budget that is set.
pub(super) fn budget_remaining(db: &Arc<Db>, budget_usd: f64) -> f64 {
    let month = month_key(now_ms(), local_offset_seconds());
    let spent = SqlLedger::open(Arc::clone(db))
        .and_then(|ledger| ledger.month_total(&month))
        .unwrap_or(0.0);
    left_usd(spent, budget_usd)
}

fn budget_left(inputs: &Inputs) -> Option<f64> {
    inputs
        .budget_usd
        .map(|usd| budget_remaining(&inputs.db, usd))
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
    let base = SessionArgs {
        model: inputs.live.model.clone(),
        effort: None,
        resume: resume.map(str::to_owned),
        fork: false,
        mcp_config: config.to_string_lossy().into_owned(),
        budget_left_usd: budget_left(inputs),
    };
    Ok(ChildSpec {
        program: inputs.exe.clone(),
        args: args_of(inputs, &base)?,
        env: build_env(std::env::vars(), &key, inputs.live.approval_timeout_min),
        cwd: inputs
            .live
            .cwd
            .clone()
            .unwrap_or_else(|| inputs.home.clone()),
        stderr_log: dir.join("child-stderr.log"),
    })
}

fn spec_factory(
    svc: &Services,
    live: &Live,
    shadow: Option<ShadowInput>,
    fork_from: Option<String>,
) -> SpecFactory {
    let inputs = Inputs {
        home: svc.paths.home.clone(),
        db: Arc::clone(&svc.db),
        exe: live.claude_exe.clone().unwrap_or_else(find_exe),
        budget_usd: config::budget(svc).budget_usd,
        live: live.clone(),
        gate: Arc::clone(&svc.overlord_approvals),
        endpoint: format!("http://127.0.0.1:{}/mcp/overlord", svc.port()),
        shadow,
        fork_from,
    };
    Box::new(move |resume| spec_of(&inputs, resume))
}

/// Why a start did not happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StartFault {
    Shadow,
    NotLive,
    NoWake,
    Running,
}

impl StartFault {
    pub(crate) fn sentence(self) -> &'static str {
        match self {
            StartFault::Shadow => "Argus runs in shadow mode; the Overlord does not start",
            StartFault::NotLive => {
                "the Overlord mode is off; set overlord.mode to live in config.json"
            }
            StartFault::NoWake => "the wake engine is off; the Overlord does not start",
            StartFault::Running => "the hosted Overlord is already running",
        }
    }
}

/// The boot path: starts when autostart, a saved key and the settings allow, and stays quiet when
/// they do not.
pub(crate) fn start(svc: &Arc<Services>) {
    super::boot_resume::spawn(svc);
    if config::autostart(svc) && super::keeper::key_saved(svc) {
        let _ = try_start(svc);
    }
}

/// Starts the supervisor when the mode is `live` or `shadow` and the wake engine is on, with a new
/// session for a live child.
///
/// # Errors
/// A [`StartFault`] naming why nothing started.
pub(crate) fn try_start(svc: &Arc<Services>) -> Result<(), StartFault> {
    try_start_from(svc, Origin::Fresh)
}

/// The live child's observer: its first message tells it to register, and at a switch-over to read
/// the desktop Overlord's handover.
fn live_observer(svc: &Services, wake: SharedWake, switching: bool, unproven: bool) -> Wakes {
    let file = handover_file(svc).display().to_string();
    let handover = switching.then_some(file);
    Wakes {
        log: Arc::clone(&svc.log),
        hold: Attachment::new(wake, Arc::clone(&svc.overlord_approvals)),
        intro: Mutex::new(Some(live_intro(handover.as_deref()))),
        db: Arc::clone(&svc.db),
        unproven: AtomicBool::new(unproven),
    }
}

/// The session to resume and the desktop session to fork at the first start.
fn first_session(origin: Origin) -> (Option<String>, Option<String>) {
    match origin {
        Origin::Fresh => (None, None),
        Origin::Resume(id) => (Some(id), None),
        Origin::Fork(id) => (None, Some(id)),
    }
}

/// [`try_start`] with the live child's first session named by `origin`; a shadow child ignores it.
///
/// # Errors
/// A [`StartFault`] naming why nothing started.
pub(crate) fn try_start_from(svc: &Arc<Services>, origin: Origin) -> Result<(), StartFault> {
    if svc.shadow {
        return Err(StartFault::Shadow);
    }
    let shadowing = config::is_shadow(svc);
    if !shadowing && !config::is_live(svc) {
        return Err(StartFault::NotLive);
    }
    let wake = svc.wake.clone().ok_or(StartFault::NoWake)?;
    let live = config::live(svc);
    let switching = origin != Origin::Fresh;
    let kept = live_session::origin_for(origin, live_session::saved(&svc.db).as_deref());
    let observer: Arc<dyn Observer> = if shadowing {
        Arc::new(Shadow::new(svc, wake))
    } else {
        let unproven = !switching && kept != Origin::Fresh;
        Arc::new(live_observer(svc, wake, switching, unproven))
    };
    let input = shadowing.then(|| ShadowInput {
        boards: svc.boards(),
    });
    let (resume_from, fork_from) = if shadowing {
        (None, None)
    } else {
        first_session(kept)
    };
    let setup = Config {
        spec: spec_factory(svc, &live, input, fork_from),
        restart_max: live.restart_max,
        resume_from,
    };
    svc.overlord
        .install(|| supervisor::start(Arc::new(SystemChildHost::default()), setup, observer))
}

/// Asks the supervisor to end the child by pid.
pub(crate) fn stop(svc: &Services) {
    if let Some(link) = svc.overlord.link() {
        link.stop();
    }
}
