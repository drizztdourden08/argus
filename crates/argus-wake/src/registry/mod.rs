//! The registry: which threads are hooked on this dashboard, whether it can wake them, and what
//! happened when it tried.
//!
//! Per project, every live board thread (not done, not archived, not a placeholder) has a row:
//!
//! * `listening`: a v2 wake listener for that thread, or the lead's `*` listener, and nothing pending
//! * `waking`: a wake for the thread is queued or delivered, it has not reacted yet
//! * `fault`: a wake failed and nobody cleared it yet (only the Overlord clears)
//! * `busy`: no listener, but the thread is in a turn right now
//! * `idle`: no listener and nothing pending ("not listening")
//!
//! A thread reacted to a wake when its heartbeat `lastSeen` is later than the moment the wake was
//! delivered, or when it acknowledged the inbox messages that were waiting at that moment.
//!
//! A wake fails when it stays queued for more than `no_listener_ms` (nobody was listening), when it was
//! delivered and the thread shows no sign of life for more than `reaction_ms`, or when writing it to a
//! listener threw. A failure opens a fault (one open per project and thread), mirrors it to the row, and
//! wakes the Overlord with reason `wake-fault`. The fault of the Overlord itself shows on the page only.
//! The registry never clears a fault, except the two that clear themselves: `not-listening` and
//! `board-closed`.

mod evaluate;
mod faults;
mod mirror;
mod pending;
mod rows;
mod watch;

pub use pending::Pending;
pub use rows::{BoardFlags, FaultSummary, ListenFlags, PendingWake, RegistryRow, RowState};

use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use tracing::warn;

use crate::docs::DocSink;
use crate::engine::{EngineNote, WakeEngine};
use crate::faults::{ClearFault, Fault, FaultStore};
use crate::presence::Presence;
use crate::reasons::{MAIN_SCOPE, PORTFOLIO};
use crate::source::BoardSource;
use crate::time::iso;
use argus_core::clock::Clock;

/// How long a wake may stay unreacted after it was delivered.
pub const DEFAULT_REACTION_MS: i64 = 600_000;
/// How long a wake may stay queued with nobody listening.
pub const DEFAULT_NO_LISTENER_MS: i64 = 60_000;
/// How often the timeouts are checked.
pub const DEFAULT_TICK_MS: i64 = 5_000;
/// How long a live thread may have no open board tab.
pub const DEFAULT_BOARD_GRACE_MS: i64 = 30_000;
/// How recent a heartbeat makes a session alive.
pub const DEFAULT_ALIVE_MS: i64 = 600_000;
/// The shortest note a fault clear needs.
pub const NOTE_MIN: usize = 10;
/// The longest note a fault clear takes.
pub const NOTE_MAX: usize = 300;

const REBUILD_MS: i64 = 15 * 60 * 1000;

type Key = (String, String);

/// The timeouts of the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryConfig {
    /// A delivered wake with no sign of life for this long is a fault.
    pub reaction_ms: i64,
    /// A queued wake with nobody listening for this long is a fault.
    pub no_listener_ms: i64,
    /// How often the owner calls `tick`.
    pub tick_ms: i64,
    /// A required board closed for this long is a fault.
    pub board_grace_ms: i64,
    /// A heartbeat this recent makes a session alive.
    pub alive_ms: i64,
    /// An active session with no wake-stream for this long is a fault.
    pub deaf_grace_ms: i64,
    /// After a start, nothing is raised for this long.
    pub restart_grace_ms: i64,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            reaction_ms: DEFAULT_REACTION_MS,
            no_listener_ms: DEFAULT_NO_LISTENER_MS,
            tick_ms: DEFAULT_TICK_MS,
            board_grace_ms: DEFAULT_BOARD_GRACE_MS,
            alive_ms: DEFAULT_ALIVE_MS,
            deaf_grace_ms: DEFAULT_REACTION_MS,
            restart_grace_ms: 0,
        }
    }
}

/// What the registry works on besides its own state.
pub struct Env<'a> {
    /// The wake engine (listeners, wakes to the Overlord).
    pub engine: &'a mut WakeEngine,
    /// The open board tabs; `None` switches the board-open check off.
    pub presence: Option<&'a Presence>,
}

/// The registry.
pub struct Registry {
    config: RegistryConfig,
    boards: Arc<dyn BoardSource>,
    fault_store: Box<dyn FaultStore>,
    docs: Arc<dyn DocSink>,
    clock: Arc<dyn Clock>,
    pending: IndexMap<Key, Pending>,
    last_mirror: HashMap<String, String>,
    missing: HashMap<Key, i64>,
    deaf: HashMap<Key, i64>,
    started_at: i64,
}

/// What a registry reads and writes besides the engine.
pub struct RegistryDeps {
    /// The project boards.
    pub boards: Arc<dyn BoardSource>,
    /// The faults table.
    pub faults: Box<dyn FaultStore>,
    /// Where the `registry` documents go.
    pub docs: Arc<dyn DocSink>,
    /// The clock.
    pub clock: Arc<dyn Clock>,
}

impl Registry {
    /// A registry over its dependencies.
    #[must_use]
    pub fn new(deps: RegistryDeps, config: RegistryConfig) -> Self {
        let started_at = deps.clock.now_ms();
        Self {
            config,
            boards: deps.boards,
            fault_store: deps.faults,
            docs: deps.docs,
            clock: deps.clock,
            pending: IndexMap::new(),
            last_mirror: HashMap::new(),
            missing: HashMap::new(),
            deaf: HashMap::new(),
            started_at,
        }
    }

    /// The timeouts.
    #[must_use]
    pub fn config(&self) -> &RegistryConfig {
        &self.config
    }

    fn now(&self) -> i64 {
        self.clock.now_ms()
    }

    /// The project keys the registry watches (every board except the portfolio page scope).
    fn scopes(&self) -> Vec<String> {
        self.boards
            .scopes()
            .into_iter()
            .filter(|k| k != PORTFOLIO)
            .collect()
    }

    /// The Overlord's thread: the coordinator of project `portfolio`.
    #[must_use]
    pub fn main_thread(&self) -> String {
        self.boards
            .board(MAIN_SCOPE)
            .and_then(|b| b.coordinator())
            .unwrap_or_else(|| "coordinator".to_owned())
    }

    /// Is this the Overlord's own thread?
    #[must_use]
    pub fn is_main(&self, scope: &str, thread: &str) -> bool {
        scope == MAIN_SCOPE && thread == self.main_thread()
    }

    /// Hook into the engine, take up the young events still queued from before a restart, and write the docs.
    pub fn start(&mut self, env: &mut Env<'_>) {
        self.started_at = self.now();
        env.engine.observe(true);
        let since = iso(self.started_at - REBUILD_MS);
        let open = self.open_faults();
        for scope in env.engine.known_scopes() {
            for row in env.engine.queued_since(&scope, &since) {
                let key = (scope.clone(), row.thread.clone());
                if !self.pending.contains_key(&key) && !open.contains_key(&key) {
                    let inbox = self.inbox_ids(&scope, &row.thread);
                    let pending = Pending::rebuilt(&row, self.started_at, inbox);
                    self.pending.insert(key, pending);
                }
            }
        }
        self.refresh(env, None);
    }

    /// Stop observing the engine.
    pub fn stop(&mut self, env: &mut Env<'_>) {
        env.engine.observe(false);
    }

    /// Take up what the engine reported since the last call, until it has nothing more.
    pub fn pump(&mut self, env: &mut Env<'_>) {
        loop {
            let notes = env.engine.take_notes();
            if notes.is_empty() {
                break;
            }
            for note in notes {
                self.handle(env, note);
            }
        }
    }

    fn handle(&mut self, env: &mut Env<'_>, note: EngineNote) {
        match note {
            EngineNote::Flushed {
                scope,
                thread,
                id,
                at,
                reason,
                text,
                sent,
            } => {
                let event = pending::TrackedEvent {
                    id,
                    at,
                    reason: Some(reason),
                    text: Some(text),
                };
                self.track(&scope, &thread, event, sent > 0);
                self.refresh(env, Some(&scope));
            }
            EngineNote::Replayed { scope, events } => {
                for ev in events {
                    let event = pending::TrackedEvent {
                        id: ev.id,
                        at: ev.at,
                        reason: Some(ev.reason),
                        text: Some(ev.text),
                    };
                    self.track(&scope, &ev.thread, event, true);
                }
                self.refresh(env, Some(&scope));
            }
            EngineNote::Attached { scope } => {
                self.evaluate_scope(env, Some(&scope));
                self.refresh(env, Some(&scope));
            }
            EngineNote::Detached { scope } => self.refresh(env, Some(&scope)),
            EngineNote::DeliveryError {
                scope,
                thread,
                id,
                err,
            } => {
                self.raise_delivery_error(env, &scope, &thread, &id, &err);
                self.refresh(env, Some(&scope));
            }
        }
    }

    /// The board of a project changed (a heartbeat, a new thread, an acked inbox) or a board tab opened or closed.
    pub fn on_change(&mut self, env: &mut Env<'_>, scope: &str) {
        self.evaluate_scope(env, Some(scope));
        self.refresh(env, Some(scope));
        self.pump(env);
    }

    /// A hard resync wiped the docs of a scope.
    pub fn on_reset(&mut self, env: &mut Env<'_>, scope: &str) {
        self.last_mirror.remove(scope);
        self.last_mirror.remove(PORTFOLIO);
        self.refresh(
            env,
            if scope == PORTFOLIO {
                None
            } else {
                Some(scope)
            },
        );
        self.pump(env);
    }

    /// The periodic check of timeouts and boards.
    pub fn tick(&mut self, env: &mut Env<'_>) {
        self.evaluate_scope(env, None);
        self.refresh(env, None);
        self.pump(env);
    }

    /// The faults, for the CLI and the route.
    #[must_use]
    pub fn faults(&self, all: bool) -> Vec<Fault> {
        self.fault_store.list(all).unwrap_or_else(|err| {
            warn!(%err, "wake faults not read");
            Vec::new()
        })
    }

    /// Clear a fault; only the route (after it checked who asks) calls this.
    pub fn clear_fault(
        &mut self,
        env: &mut Env<'_>,
        project: &str,
        thread: &str,
        by: &str,
        note: &str,
    ) -> Option<Fault> {
        let request = ClearFault {
            scope: project.to_owned(),
            thread: thread.to_owned(),
            at: iso(self.now()),
            by: by.to_owned(),
            note: note.to_owned(),
        };
        let cleared = match self.fault_store.clear(&request) {
            Ok(cleared) => cleared?,
            Err(err) => {
                warn!(project, thread, %err, "wake fault not cleared");
                return None;
            }
        };
        self.pending
            .shift_remove(&(project.to_owned(), thread.to_owned()));
        self.refresh(env, Some(project));
        self.pump(env);
        Some(cleared)
    }

    /// The wakes waiting for a reaction.
    #[must_use]
    pub fn pending(&self) -> Vec<(String, String, Pending)> {
        self.pending
            .iter()
            .map(|((s, t), p)| (s.clone(), t.clone(), p.clone()))
            .collect()
    }
}
