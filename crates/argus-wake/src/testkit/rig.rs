//! The engine and the service on in-memory storage and a manual clock.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value;

use super::fixtures::{MIN, MemoryBusy, START, registry_boards};
use super::listen::Tap;
use crate::board::BoardView;
use crate::config::WakeConfig;
use crate::engine::{AttachOptions, EngineConfig, WakeEngine, channel};
use crate::memory::{
    MemoryActivity, MemoryBoards, MemoryDocs, MemoryFaults, MemoryNotices, MemoryOwner,
    MemoryStore, MemorySyncs,
};
use crate::registry::RowState;
use crate::service::{WakeDeps, WakeService};
use crate::source::NoLive;
use crate::stalls::MemoryKv;
use crate::wake::WakeInput;
use argus_core::clock::{Clock, ManualClock};

/// A bare engine: windows, listeners, replay and the mirror.
pub struct EngineRig {
    /// The clock the engine reads.
    pub clock: ManualClock,
    /// The wakes table.
    pub store: MemoryStore,
    /// The page documents.
    pub docs: MemoryDocs,
    /// The engine.
    pub engine: WakeEngine,
}

impl EngineRig {
    /// An engine with this window, starting at `START`.
    #[must_use]
    pub fn new(window_ms: i64) -> Self {
        Self::with(EngineConfig {
            window_ms,
            ..EngineConfig::default()
        })
    }

    /// An engine with this config, starting at `START`.
    #[must_use]
    pub fn with(config: EngineConfig) -> Self {
        let clock = ManualClock::at(START);
        let store = MemoryStore::new();
        let docs = MemoryDocs::new();
        let engine = WakeEngine::new(
            Box::new(store.clone()),
            Arc::new(docs.clone()),
            Arc::new(clock.clone()),
            config,
        );
        Self {
            clock,
            store,
            docs,
            engine,
        }
    }

    /// Attach a channel listener.
    pub fn listen(&mut self, scope: &str, thread: &str, options: AttachOptions) -> Tap {
        let (sink, rx) = channel();
        let attached = self.engine.attach(scope, thread, Box::new(sink), options);
        Tap::new(attached, rx)
    }

    /// Submit wakes.
    ///
    /// # Panics
    /// When the store refuses.
    pub fn submit(&mut self, wakes: &[WakeInput]) {
        self.engine.submit(wakes).expect("submit");
    }

    /// Let time pass; every window whose deadline falls inside closes at its own moment, like a real timer.
    pub fn advance(&mut self, ms: i64) {
        let until = self.clock.now_ms() + ms;
        while let Some(at) = self.engine.next_deadline().filter(|at| *at <= until) {
            self.clock.set(at.max(self.clock.now_ms()));
            self.engine.tick();
        }
        self.clock.set(until);
    }

    /// The `wakes` documents of a project by id.
    #[must_use]
    pub fn mirrored(&self, scope: &str) -> BTreeMap<String, Value> {
        self.docs
            .list(scope, "wakes")
            .into_iter()
            .map(|d| (d.id, d.data))
            .collect()
    }
}

/// The whole service on in-memory storage.
pub struct Rig {
    /// The clock the service reads.
    pub clock: ManualClock,
    /// The wakes table.
    pub store: MemoryStore,
    /// The faults table.
    pub faults: MemoryFaults,
    /// The page documents.
    pub docs: MemoryDocs,
    /// The project boards.
    pub boards: MemoryBoards,
    /// The activity log.
    pub activity: MemoryActivity,
    /// The remote syncs of the projects.
    pub syncs: MemorySyncs,
    /// What the owner was told.
    pub owner: MemoryOwner,
    /// The open notices of the extra source (`worktree-stale` in the server).
    pub notices: MemoryNotices,
    /// The projects with a running process, which the stall watch counts as work.
    pub busy: MemoryBusy,
    /// The inbox reads and the stall acks.
    pub kv: MemoryKv,
    /// The service.
    pub service: WakeService,
}

impl Rig {
    /// A service with no boards, starting at `START`.
    #[must_use]
    pub fn new(config: &WakeConfig, track_presence: bool) -> Self {
        let clock = ManualClock::at(START);
        let store = MemoryStore::new();
        let faults = MemoryFaults::new();
        let docs = MemoryDocs::new();
        let boards = MemoryBoards::new();
        let activity = MemoryActivity::new();
        let syncs = MemorySyncs::new();
        let owner = MemoryOwner::new();
        let notices = MemoryNotices::new();
        let busy = MemoryBusy::new();
        let kv = MemoryKv::new();
        let deps = WakeDeps {
            store: Box::new(store.clone()),
            faults: Box::new(faults.clone()),
            docs: Arc::new(docs.clone()),
            boards: Arc::new(boards.clone()),
            live: Arc::new(NoLive),
            syncs: Arc::new(syncs.clone()),
            activity: Some(Arc::new(activity.clone())),
            clock: Arc::new(clock.clone()),
        };
        let mut service = WakeService::new(deps, config, track_presence);
        service.set_owner_sink(Arc::new(owner.clone()));
        service.add_notice_source(Arc::new(notices.clone()));
        service.set_stall_sources(Arc::new(busy.clone()), Arc::new(kv.clone()));
        Self {
            clock,
            store,
            faults,
            docs,
            boards,
            activity,
            syncs,
            owner,
            notices,
            busy,
            kv,
            service,
        }
    }

    /// The registry tests' setup: `alpha` and `portfolio` boards whose threads were last seen ten minutes ago, started.
    #[must_use]
    pub fn registry(config: &WakeConfig, track_presence: bool) -> Self {
        Self::registry_seen(config, track_presence, 10 * MIN)
    }

    /// The same with every thread last seen `seen_ago_ms` before the start.
    #[must_use]
    pub fn registry_seen(config: &WakeConfig, track_presence: bool, seen_ago_ms: i64) -> Self {
        let mut rig = Self::new(config, track_presence);
        for (scope, board) in registry_boards(seen_ago_ms) {
            rig.boards.set(scope, board);
        }
        rig.service.start();
        rig
    }

    /// Attach a channel listener.
    pub fn listen(&mut self, scope: &str, thread: &str, options: AttachOptions) -> Tap {
        let (sink, rx) = channel();
        let attached = self.service.attach(scope, thread, Box::new(sink), options);
        Tap::new(attached, rx)
    }

    /// Let time pass; every window whose deadline falls inside closes at its own moment, like a real timer.
    pub fn advance(&mut self, ms: i64) {
        let until = self.clock.now_ms() + ms;
        while let Some(at) = self
            .service
            .engine()
            .next_deadline()
            .filter(|at| *at <= until)
        {
            self.clock.set(at.max(self.clock.now_ms()));
            self.service.tick();
        }
        self.clock.set(until);
    }

    /// Time passes in steps and the registry timer ticks after each, the way the real 5 s timer does.
    pub fn pass(&mut self, ms: i64, step: i64) {
        let mut left = ms;
        while left > 0 {
            self.clock.advance(step.min(left));
            self.service.registry_tick();
            left -= step;
        }
    }

    /// The thread shows a sign of life one millisecond from now, and the board is exported.
    pub fn seen(&mut self, scope: &str, thread: &str) {
        let at = crate::time::iso(self.clock.now_ms() + 1);
        self.edit_thread(scope, thread, |t| t.last_seen = at.as_str().into());
        self.service.on_board(scope);
    }

    /// Change one thread of a board in place.
    pub fn edit_thread(
        &self,
        scope: &str,
        thread: &str,
        change: impl FnOnce(&mut crate::board::ThreadView),
    ) {
        self.boards.edit(scope, |b| {
            if let Some(t) = b.threads.get_mut(thread) {
                change(t);
            }
        });
    }

    /// Change a board in place.
    pub fn edit(&self, scope: &str, change: impl FnOnce(&mut BoardView)) {
        self.boards.edit(scope, change);
    }

    /// Where a thread stands in the registry.
    pub fn state(&mut self, scope: &str, thread: &str) -> Option<RowState> {
        self.service
            .registry_rows(scope)
            .iter()
            .find(|r| r.thread == thread)
            .map(|r| r.state)
    }

    /// The `registry` document of a row.
    #[must_use]
    pub fn doc(&self, scope: &str, id: &str) -> Option<Value> {
        self.docs.get(scope, "registry", id)
    }

    /// The wakes made for the Overlord about a fault.
    #[must_use]
    pub fn fault_wakes(&self) -> Vec<crate::wake::WakeRow> {
        self.wakes_of("wake-fault")
    }

    /// Every wake row of a reason, oldest first.
    #[must_use]
    pub fn wakes_of(&self, reason: &str) -> Vec<crate::wake::WakeRow> {
        self.store
            .all()
            .into_iter()
            .filter(|r| r.reason == reason)
            .collect()
    }
}
