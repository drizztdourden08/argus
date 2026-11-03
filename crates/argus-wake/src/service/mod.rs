//! The wake service: the engine, the registry, the board presence and the two reminder producers, wired
//! together the way the Node `createWake` wired them. The owner calls `tick` when `next_due_ms` has
//! passed (see [`driver`]); everything else is a method call.

pub mod driver;
mod periodic;
mod remind;
mod schedule;
mod stall;

use std::sync::Arc;

use tokio::sync::Notify;
use tracing::warn;

use crate::activity::{ActivityFaults, ActivityLine, ActivitySink, Level};
use crate::agent_wakes::AgentWakes;
use crate::board::{BoardView, EntryView};
use crate::config::WakeConfig;
use crate::dep_wakes::DepWakes;
use crate::docs::DocSink;
use crate::engine::{
    AttachOptions, Attached, EngineConfig, ListenerId, ListenerInfo, ListenerSink, WakeEngine,
};
use crate::faults::{Fault, FaultStore};
use crate::item_wakes::ItemWakes;
use crate::mirror::{WakeDoc, docs_of};
use crate::notices::{NoticeDigest, digest};
use crate::policy::{ApplyCtx, RunView, coordinator_in, failed_run_line, wake_for_run, wakes_for};
use crate::presence::{Presence, PresenceRow, PresenceToken};
use crate::reasons::{IMMEDIATE_REASONS, MAIN_SCOPE, PORTFOLIO};
use crate::registry::{Env, Registry, RegistryDeps, RegistryRow};
use crate::reminders::{NoticeSource, OwnerAlerts, OwnerSink, Resend};
use crate::source::{AgentLive, BoardSource, SyncSource};
use crate::stalls::Stalls;
use crate::store::{RowsQuery, StoreResult, WakeStore};
use crate::sync_wakes::{SyncDue, SyncWakes};
use crate::wake::WakeInput;
use argus_core::clock::Clock;
use schedule::Schedule;

const DEFAULT_LIST_LIMIT: usize = 50;
const MAX_LIST_LIMIT: usize = 200;

/// Everything the service reads and writes.
pub struct WakeDeps {
    /// The wakes table.
    pub store: Box<dyn WakeStore>,
    /// The faults table.
    pub faults: Box<dyn FaultStore>,
    /// Where the page documents go.
    pub docs: Arc<dyn DocSink>,
    /// The project boards.
    pub boards: Arc<dyn BoardSource>,
    /// The live Claude tasks of the sub-agents.
    pub live: Arc<dyn AgentLive>,
    /// The last remote sync of each project.
    pub syncs: Arc<dyn SyncSource>,
    /// The activity log, when there is one.
    pub activity: Option<Arc<dyn ActivitySink>>,
    /// The clock.
    pub clock: Arc<dyn Clock>,
}

/// One owner post the importer applied.
#[derive(Debug, Clone, Default)]
pub struct AppliedPost {
    /// Project key, or `_portfolio`.
    pub scope: String,
    /// The entry as the board stored it.
    pub entry: Option<EntryView>,
    /// The post result.
    pub status: Option<String>,
    /// The board after the post.
    pub board: Option<Arc<BoardView>>,
    /// The board as the last export left it.
    pub before: Option<Arc<BoardView>>,
}

/// The wake service.
pub struct WakeService {
    engine: WakeEngine,
    registry: Registry,
    presence: Presence,
    track_presence: bool,
    agents: AgentWakes,
    items: ItemWakes,
    sync_wakes: SyncWakes,
    dep_wakes: DepWakes,
    stalls: Stalls,
    resend: Resend,
    owner_alerts: OwnerAlerts,
    owner: Option<Arc<dyn OwnerSink>>,
    notice_sources: Vec<Arc<dyn NoticeSource>>,
    boards: Arc<dyn BoardSource>,
    live: Arc<dyn AgentLive>,
    syncs: Arc<dyn SyncSource>,
    activity: Option<Arc<dyn ActivitySink>>,
    clock: Arc<dyn Clock>,
    config: WakeConfig,
    schedule: Schedule,
    notify: Arc<Notify>,
}

impl WakeService {
    /// A service over its dependencies. `track_presence` switches the board-open check on.
    #[must_use]
    pub fn new(deps: WakeDeps, config: &WakeConfig, track_presence: bool) -> Self {
        let now = deps.clock.now_ms();
        let sync_due: Arc<dyn NoticeSource> = Arc::new(SyncDue::new(
            Arc::clone(&deps.boards),
            Arc::clone(&deps.syncs),
            config.syncs.due_ms,
            now,
        ));
        let engine_config = EngineConfig {
            window_ms: config.window_ms,
            replay_ms: EngineConfig::default().replay_ms,
            immediate: IMMEDIATE_REASONS.iter().map(|r| (*r).to_owned()).collect(),
            policy: config.policy,
        };
        let engine = WakeEngine::new(
            deps.store,
            Arc::clone(&deps.docs),
            Arc::clone(&deps.clock),
            engine_config,
        );
        let faults: Box<dyn FaultStore> = match &deps.activity {
            Some(sink) => Box::new(ActivityFaults::new(deps.faults, Arc::clone(sink))),
            None => deps.faults,
        };
        let stalls = Stalls::new(config.stall.clone(), Arc::clone(&deps.docs), now);
        let registry_deps = RegistryDeps {
            boards: Arc::clone(&deps.boards),
            faults,
            docs: deps.docs,
            clock: Arc::clone(&deps.clock),
        };
        Self {
            engine,
            registry: Registry::new(registry_deps, config.registry.clone()),
            presence: Presence::new(),
            track_presence,
            agents: AgentWakes::new(config.agents.clone(), now),
            items: ItemWakes::new(config.items.clone()),
            sync_wakes: SyncWakes::new(config.syncs.clone(), now),
            dep_wakes: DepWakes::new(config.deps.clone()),
            stalls,
            resend: Resend::new(config.reminders.resend_ms),
            owner_alerts: OwnerAlerts::new(config.reminders.owner_ms),
            owner: None,
            notice_sources: vec![sync_due],
            boards: deps.boards,
            live: deps.live,
            syncs: deps.syncs,
            activity: deps.activity,
            clock: deps.clock,
            config: config.clone(),
            schedule: Schedule::starting(now, config),
            notify: Arc::new(Notify::new()),
        }
    }

    fn with_env<R>(&mut self, run: impl FnOnce(&mut Registry, &mut Env<'_>) -> R) -> R {
        let mut env = Env {
            engine: &mut self.engine,
            presence: self.track_presence.then_some(&self.presence),
        };
        run(&mut self.registry, &mut env)
    }

    fn pump(&mut self) {
        self.with_env(Registry::pump);
    }

    /// The wake that signals the driver to look at the deadlines again.
    #[must_use]
    pub fn notifier(&self) -> Arc<Notify> {
        Arc::clone(&self.notify)
    }

    /// The Overlord's thread, or `coordinator`.
    fn main_thread(&self) -> String {
        coordinator_in(self.boards.board(MAIN_SCOPE).as_deref())
            .unwrap_or_else(|| "coordinator".to_owned())
    }

    /// Hook the registry to the engine, close the windows a stop left open, and arm the timers.
    pub fn start(&mut self) {
        self.with_env(Registry::start);
        if let Err(err) = self.engine.recover() {
            warn!(%err, "wake recovery failed");
        }
        self.pump();
        self.schedule = Schedule::starting(self.clock.now_ms(), &self.config);
    }

    /// Close the open windows and stop observing.
    pub fn stop(&mut self) {
        self.engine.stop();
        self.pump();
        self.with_env(Registry::stop);
    }

    /// Record wakes and open or join their windows.
    ///
    /// # Errors
    /// When a wake cannot be written to the store.
    pub fn submit(&mut self, wakes: &[WakeInput]) -> StoreResult<usize> {
        let result = self.engine.submit(wakes);
        self.pump();
        self.notify.notify_one();
        result
    }

    fn submit_logged(&mut self, wakes: &[WakeInput], what: &str) {
        if wakes.is_empty() {
            return;
        }
        if let Err(err) = self.submit(wakes) {
            warn!(%err, "{what} not sent");
        }
    }

    /// One applied owner post: ask the policy which threads it wakes. Never fails: the post is already done.
    pub fn on_applied(&mut self, post: &AppliedPost) {
        let boards = Arc::clone(&self.boards);
        let coordinator = move |key: &str| {
            Some(
                coordinator_in(boards.board(key).as_deref())
                    .unwrap_or_else(|| "coordinator".to_owned()),
            )
        };
        let ctx = ApplyCtx {
            scope: &post.scope,
            entry: post.entry.as_ref(),
            status: post.status.as_deref(),
            board: post.board.as_deref(),
            before: post.before.as_deref(),
            coordinator_of: &coordinator,
        };
        let wakes = wakes_for(&ctx);
        self.submit_logged(&wakes, "owner wake");
    }

    /// A run ended: a failure wakes the lead of its project and leaves a line in the activity log.
    pub fn on_run_end(&mut self, run: &RunView) {
        if let Some(line) = failed_run_line(run) {
            if let Some(sink) = &self.activity {
                sink.add(ActivityLine {
                    kind: "run-failed".to_owned(),
                    level: Level::Error,
                    text: format!("{}: {line}", run.scope),
                    project: Some(run.scope.clone()),
                    thread: None,
                });
            }
        }
        let lead = coordinator_in(self.boards.board(&run.scope).as_deref());
        if let Some(wake) = wake_for_run(run, lead.as_deref()) {
            self.submit_logged(&[wake], "run wake");
        }
    }

    /// A board was exported again: a heartbeat, a new thread, an acked inbox.
    pub fn on_board(&mut self, scope: &str) {
        self.with_env(|registry, env| registry.on_change(env, scope));
    }

    /// A hard resync wiped the docs of a scope.
    pub fn on_reset(&mut self, scope: &str) {
        self.engine.remirror(scope);
        if scope == PORTFOLIO {
            self.stalls.forget_docs();
        }
        self.with_env(|registry, env| registry.on_reset(env, scope));
    }

    /// A listener connects (see [`WakeEngine::attach`]).
    pub fn attach(
        &mut self,
        scope: &str,
        thread: &str,
        sink: Box<dyn ListenerSink>,
        options: AttachOptions,
    ) -> Attached {
        let attached = self.engine.attach(scope, thread, sink, options);
        self.pump();
        attached
    }

    /// A listener's connection closed.
    pub fn detach(&mut self, id: &ListenerId) {
        self.engine.detach(id);
        self.pump();
    }

    /// A board page opened with `?thread=` holds a stream: register the tab.
    pub fn presence_attach(&mut self, scope: &str, thread: &str, tab: &str) -> PresenceToken {
        let token = self
            .presence
            .attach(scope, thread, tab, self.clock.now_ms());
        self.on_board(scope);
        token
    }

    /// That stream closed.
    pub fn presence_detach(&mut self, token: &PresenceToken) {
        let scope = token.scope().to_owned();
        self.presence.detach(token);
        self.on_board(&scope);
    }

    /// The threads with an open board.
    #[must_use]
    pub fn presence_list(&self) -> Vec<PresenceRow> {
        self.presence.list()
    }

    /// The registry rows of a project.
    #[must_use]
    pub fn registry_rows(&mut self, scope: &str) -> Vec<RegistryRow> {
        self.with_env(|registry, env| registry.rows(env, scope))
    }

    /// The wake faults.
    #[must_use]
    pub fn faults(&self, all: bool) -> Vec<Fault> {
        self.registry.faults(all)
    }

    /// Clear a fault; the caller checked who asks.
    pub fn clear_fault(
        &mut self,
        project: &str,
        thread: &str,
        by: &str,
        note: &str,
    ) -> Option<Fault> {
        self.with_env(|registry, env| registry.clear_fault(env, project, thread, by, note))
    }

    /// The page-facing rows of a project, newest first.
    ///
    /// # Errors
    /// When the rows cannot be read.
    pub fn list(
        &self,
        scope: &str,
        limit: Option<usize>,
        thread: Option<&str>,
    ) -> StoreResult<Vec<WakeDoc>> {
        let query = RowsQuery {
            limit: limit.unwrap_or(DEFAULT_LIST_LIMIT).clamp(1, MAX_LIST_LIMIT),
            thread: thread.map(str::to_owned),
        };
        let rows = self.engine.store().rows(scope, &query)?;
        docs_of(self.engine.store(), &rows)
    }

    /// The passive notices of a thread (`*`: every thread) made after `since_ms`, read from the
    /// newest rows of the project: what the prompt hook's digest line says.
    ///
    /// # Errors
    /// When the rows cannot be read.
    pub fn notices(&self, scope: &str, thread: &str, since_ms: i64) -> StoreResult<NoticeDigest> {
        let query = RowsQuery {
            limit: MAX_LIST_LIMIT,
            thread: Some(thread.to_owned()),
        };
        let rows = self.engine.store().rows(scope, &query)?;
        Ok(digest(&rows, since_ms))
    }

    /// The open streams of a project.
    #[must_use]
    pub fn listeners(&self, scope: &str) -> Vec<ListenerInfo> {
        self.engine.listeners_of(scope)
    }

    /// The engine, for the numbers the routes need (cap, window, has-listener).
    #[must_use]
    pub fn engine(&self) -> &WakeEngine {
        &self.engine
    }

    /// The registry's pending wakes and timeouts.
    #[must_use]
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}
