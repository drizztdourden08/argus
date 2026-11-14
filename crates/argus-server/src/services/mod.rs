//! The services every route reaches: config, paths, database, bus, log, the activity log, the
//! streams and the instance facts, and one slot per service crate. A service that is off is
//! `None` and its routes answer 503.
//!
//! One module per service lives beside this file: `build(&Services, &ServeOptions)` makes it
//! from what the services before it hold, `start(&Arc<Services>)` begins its timers and
//! `stop(&Services)` ends them. [`compose`] holds the three orders.

use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError, RwLock};
use std::time::Duration;

use argus_agents::{AgentReaper, AgentReconciler, AgentWatcher};
use argus_core::log::Logger;
use argus_core::remote::Switch;
use argus_core::{Bus, Config, Paths, Subscription};
use argus_launchers::RunService;
use argus_launchers::launcher::LauncherStale;
use argus_native::{Notifier, TrayFiles};
use argus_notify::Notify;
use argus_procs::ProcsService;
use argus_store::{ActivityLog, Db};
use argus_sync::{ScopeQueue, SyncService};
use argus_wake::service::WakeService;
use argus_worktrees::Service as WorktreeService;
use tokio_util::sync::CancellationToken;

use crate::adapters::overlord::feed::TranscriptFeed;
use crate::adapters::overlord::gate::ApprovalGate;
use crate::adapters::runs::RunHost;
use crate::boards::BoardReader;
use crate::local_names::LiveNames;
use crate::security::LoginCodes;
use crate::sse::{Fanout, MuxRegistry, RunLogs};
use crate::web::WebAssets;
use tasks::Tasks;

pub(crate) mod accounts;
pub(crate) mod accounts_alerts;
pub(crate) mod accounts_estimates;
pub(crate) mod accounts_handover;
pub(crate) mod accounts_resume;
pub(crate) mod accounts_review;
pub(crate) mod accounts_rules;
pub(crate) mod accounts_switch;
pub(crate) mod accounts_sync;
pub(crate) mod activity;
pub(crate) mod agents;
pub(crate) mod cache;
pub(crate) mod compose;
pub(crate) mod deps;
pub(crate) mod notify;
pub(crate) mod overlord;
pub(crate) mod procs;
pub(crate) mod records;
pub(crate) mod reminders;
pub(crate) mod runs;
pub(crate) mod stale;
pub(crate) mod stalls;
pub(crate) mod sync;
pub(crate) mod tasks;
pub(crate) mod wake;
pub(crate) mod wake_hook;
pub(crate) mod wake_relay;
pub(crate) mod worktrees;

pub use accounts::{AccountsOptions, AccountsService};
pub use accounts_alerts::AccountAlerts;
pub use accounts_estimates::{EstimateState, LeadRef, LeadState, estimate_state, learn_from_cycle};
pub use agents::AgentsOptions;
pub use cache::{CacheOptions, CacheService};
pub use notify::NotifyOptions;
pub use procs::ProcsOptions;
pub use records::{ConsumerSource, ConsumerStatus, Consumers, NodeStatus, Records, RecordsOptions};
pub use runs::RunsOptions;
pub use wake::WakeOptions;
pub use wake_hook::{WakeHook, WakeRequest};
pub use worktrees::WorktreesOptions;

/// The composed services.
pub struct Services {
    /// The live config.
    pub config: Arc<RwLock<Config>>,
    /// `DASHBOARD_PORT`, which a save must not write.
    pub env_port: Option<u16>,
    /// The data folders.
    pub paths: Paths,
    /// The local or remote switch (`<COORD_HOME>/remote.json`).
    pub remote: Arc<Switch>,
    /// The database.
    pub db: Arc<Db>,
    /// The event bus.
    pub bus: Bus,
    /// The server log.
    pub log: Arc<Logger>,
    /// The activity log every service writes to.
    pub activity: Arc<ActivityLog>,
    /// The files the tray leaves in the dashboard home.
    pub tray: TrayFiles,
    /// The sync service, when on.
    pub sync: Option<Arc<SyncService>>,
    /// The per-scope mutation queues.
    pub queue: ScopeQueue,
    /// The run service, when on.
    pub runs: Option<Arc<RunService<RunHost>>>,
    /// The launcher wakes (no master launcher, a launcher unused past its window), when on.
    pub launcher_stale: Option<Arc<LauncherStale>>,
    /// The wake engine and its registry, when on. Its calls take `&mut self`, so it lives in a
    /// mutex.
    pub wake: Option<Arc<Mutex<WakeService>>>,
    /// The live view of sub-agents (their transcripts), when on.
    pub agent_watcher: Option<Arc<AgentWatcher>>,
    /// The sub-agent reaper, when on.
    pub agent_reaper: Option<Arc<AgentReaper>>,
    /// The sub-agent reconciler, when on.
    pub agent_reconciler: Option<Arc<AgentReconciler>>,
    /// The process monitor, when on.
    pub procs: Option<Arc<ProcsService>>,
    /// The Claude accounts, when on.
    pub accounts: Option<Arc<AccountsService>>,
    /// The worktree scanner, when on.
    pub worktrees: Option<Arc<WorktreeService>>,
    /// The build cache of Brock and Tessera, when on.
    pub cache: Option<Arc<CacheService>>,
    /// The release store, the check runs and the local package registry, when on.
    pub records: Option<Arc<Records>>,
    /// The notifications to the owner, when on.
    pub notify: Option<Arc<Notify>>,
    /// The OS toast, when there is one.
    pub notifier: Option<Arc<dyn Notifier>>,
    /// This run's random id.
    pub instance: String,
    /// When it started.
    pub started_at: String,
    /// No outward effect: true in shadow mode and in a preview (no worktree removal, kill, run,
    /// Play, toast, remote call or cache prune).
    pub shadow: bool,
    /// Board writes (owner posts, test results, dependencies) are accepted: false only in
    /// shadow mode, where they answer `read_only`.
    pub board_writes: bool,
    /// Preview mode: a copied home with board writes and no outward effect.
    pub preview: bool,
    /// The SSE heartbeat.
    pub heartbeat: Duration,
    /// The web build.
    pub web: WebAssets,
    /// The doc stream hub.
    pub fanout: Arc<Fanout>,
    /// The open mux connections.
    pub mux: Arc<MuxRegistry>,
    /// The live tail of run logs.
    pub run_logs: Arc<RunLogs>,
    /// The live tail of the hosted Overlord transcript.
    pub overlord_feed: Arc<TranscriptFeed>,
    /// Open SSE streams.
    pub streams: Arc<AtomicUsize>,
    /// One-time login codes.
    pub codes: LoginCodes,
    /// Rings when the server should stop.
    pub stop: CancellationToken,
    pub(crate) tasks: Tasks,
    pub(crate) port: AtomicU16,
    pub(crate) names: OnceLock<LiveNames>,
    pub(crate) localhost_door: OnceLock<u16>,
    /// The running supervisor of the hosted Overlord (mode `live`).
    pub(crate) overlord: overlord::running::Running,
    /// The approvals the hosted Overlord asks for, and the bearer of its private endpoint.
    pub overlord_approvals: Arc<ApprovalGate>,
    pub(crate) links: Mutex<Vec<Subscription>>,
}

impl Services {
    /// The port the listener got.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port.load(Ordering::SeqCst)
    }

    /// A copy of the config.
    #[must_use]
    pub fn config(&self) -> Config {
        self.config
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The current token.
    #[must_use]
    pub fn token(&self) -> String {
        self.config
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .token
            .clone()
    }

    /// Whether the token is required.
    #[must_use]
    pub fn require_token(&self) -> bool {
        self.config
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .require_token
    }

    /// The server version.
    #[must_use]
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// The boards: the sync service's last good copy, else the file on disk. Take it after the
    /// sync service is built; a later change of `sync` is not seen by an older copy.
    #[must_use]
    pub fn boards(&self) -> BoardReader {
        BoardReader::new(self.sync.clone(), self.paths.coord_home.clone())
    }

    /// The id of the Overlord's board, found by its role (`boards.json`, else the legacy folder). Empty when
    /// the home has no such board, so no key matches it.
    #[must_use]
    pub fn overlord(&self) -> String {
        self.boards().overlord().unwrap_or_default()
    }

    /// Keeps a bus listener attached for as long as the services live.
    pub(crate) fn keep(&self, link: Subscription) {
        self.links
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(link);
    }
}
