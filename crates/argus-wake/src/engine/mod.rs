//! The wake engine: collects wakes, merges them per (project, thread) in a time window into ONE event,
//! and hands the event to the listeners that are connected.
//!
//! * `submit` records each wake (status queued) and opens or joins its window.
//! * A window closes by `tick` once its deadline passed (or at once for an immediate reason): the first
//!   row is the event, the others fold into it, and every matching listener is sent the event.
//! * With no listener the event stays queued and is replayed once to the first matching v2 listener that
//!   connects within the replay period (24 hours).
//! * The wake policy applies here, for every producer: a passive reason is recorded as a notice and never
//!   reaches a listener, and a thread woken inside the debounce has its next wakes held in one digest.
//!
//! The engine is pure logic over a [`WakeStore`], a [`DocSink`] and a [`Clock`]. Nothing here sleeps:
//! the owner calls `tick` when `next_deadline` has passed.

mod attach;
mod listeners;
mod notes;
mod policy;

pub use attach::{Attached, MAX_LISTENERS_PER_SCOPE};
pub use listeners::{
    AttachOptions, ChannelSink, Delivery, ListenerId, ListenerInfo, ListenerMessage, ListenerSink,
    ListenerStat, channel,
};
pub use notes::{EngineNote, ReplayedEvent};

use std::collections::HashMap;
use std::sync::Arc;

use tracing::warn;

use crate::docs::DocSink;
use crate::mirror::mirror_wakes;
use crate::store::{StoreResult, WakeStore};
use crate::time::iso;
use crate::wake::{FinishedEvent, WakeEvent, WakeInput, WakeRow, WakeStatus};
use crate::wake_policy::{WakePolicy, is_passive};
use argus_core::clock::Clock;
use listeners::ListenerSet;
use policy::Placed;

const MAX_LINES: usize = 8;
const MAX_TEXT: usize = 300;

/// How the engine windows and replays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineConfig {
    /// The coalescing window; 0 closes at once.
    pub window_ms: i64,
    /// How old a queued event may be and still be replayed.
    pub replay_ms: i64,
    /// Reasons that close the window at once.
    pub immediate: Vec<String>,
    /// How often one thread may be woken; off here, `wakePolicy` of `config.json` in the service.
    pub policy: WakePolicy,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window_ms: 15_000,
            replay_ms: 24 * 3_600_000,
            immediate: Vec::new(),
            policy: WakePolicy::off(),
        }
    }
}

type Key = (String, String);

struct Group {
    key: Key,
    rows: Vec<WakeRow>,
    deadline: Option<i64>,
    /// Held by the debounce: the window is a digest, not the short coalescing one.
    held: bool,
}

/// The wake engine.
pub struct WakeEngine {
    store: Box<dyn WakeStore>,
    docs: Arc<dyn DocSink>,
    clock: Arc<dyn Clock>,
    config: EngineConfig,
    groups: Vec<Group>,
    /// When each thread was last woken, for the debounce.
    woken: HashMap<Key, i64>,
    listeners: ListenerSet,
    observed: bool,
    notes: Vec<EngineNote>,
}

/// The first eight texts, then `+N more`.
#[must_use]
pub fn lines_of(texts: &[&str]) -> Vec<String> {
    let mut lines: Vec<String> = texts
        .iter()
        .take(MAX_LINES)
        .map(|t| (*t).to_owned())
        .collect();
    if texts.len() > MAX_LINES {
        lines.push(format!("+{} more", texts.len() - MAX_LINES));
    }
    lines
}

fn clipped(text: &str) -> String {
    if text.chars().count() > MAX_TEXT {
        let head: String = text.chars().take(MAX_TEXT - 1).collect();
        format!("{head}…")
    } else {
        text.to_owned()
    }
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn push_unique<T: PartialEq>(list: &mut Vec<T>, item: T) {
    if !list.contains(&item) {
        list.push(item);
    }
}

impl WakeEngine {
    /// An engine over a store, a document sink and a clock.
    #[must_use]
    pub fn new(
        store: Box<dyn WakeStore>,
        docs: Arc<dyn DocSink>,
        clock: Arc<dyn Clock>,
        config: EngineConfig,
    ) -> Self {
        Self {
            store,
            docs,
            clock,
            config,
            groups: Vec::new(),
            woken: HashMap::new(),
            listeners: ListenerSet::default(),
            observed: false,
            notes: Vec::new(),
        }
    }

    /// Start (or stop) recording [`EngineNote`]s for the registry.
    pub fn observe(&mut self, on: bool) {
        self.observed = on;
        if !on {
            self.notes.clear();
        }
    }

    /// The notes recorded since the last call.
    pub fn take_notes(&mut self) -> Vec<EngineNote> {
        std::mem::take(&mut self.notes)
    }

    fn note(&mut self, note: EngineNote) {
        if self.observed {
            self.notes.push(note);
        }
    }

    /// Write the page mirror of a project again (after a resync wiped its docs).
    pub fn remirror(&self, scope: &str) {
        self.mirror(scope);
    }

    fn mirror(&self, scope: &str) {
        if let Err(err) = mirror_wakes(self.store.as_ref(), self.docs.as_ref(), scope) {
            warn!(scope, %err, "wake mirror failed");
        }
    }

    /// The coalescing window in milliseconds.
    #[must_use]
    pub fn window_ms(&self) -> i64 {
        self.config.window_ms
    }

    /// The wakes table behind the engine.
    #[must_use]
    pub fn store(&self) -> &dyn WakeStore {
        self.store.as_ref()
    }

    /// Record wakes and open or join their windows; returns how many projects were touched.
    /// A passive reason is recorded as a notice only (see [`crate::wake_policy`]).
    ///
    /// # Errors
    /// When a wake cannot be written to the store.
    pub fn submit(&mut self, wakes: &[WakeInput]) -> StoreResult<usize> {
        let at = iso(self.clock.now_ms());
        let mut touched: Vec<String> = Vec::new();
        let mut fresh: Vec<Key> = Vec::new();
        let mut urgent: Vec<Key> = Vec::new();
        for wake in wakes
            .iter()
            .filter(|w| !w.thread.is_empty() && !w.text.is_empty())
        {
            let row = self.store.insert(
                &WakeInput {
                    text: clipped(&wake.text),
                    ..wake.clone()
                },
                &at,
            )?;
            push_unique(&mut touched, wake.scope.clone());
            if is_passive(&wake.reason) {
                self.shelve(&row);
                continue;
            }
            let key = (wake.scope.clone(), wake.thread.clone());
            let immediate = self.config.immediate.contains(&wake.reason);
            let placed = self.place(&key, row, immediate);
            if matches!(placed, Placed::Opened) {
                fresh.push(key.clone());
            }
            let close_now = match placed {
                Placed::Opened | Placed::Joined => immediate,
                Placed::Held { due } => due,
            };
            if close_now {
                push_unique(&mut urgent, key);
            }
        }
        for scope in &touched {
            self.mirror(scope);
        }
        let mut closing = if self.config.window_ms <= 0 {
            fresh
        } else {
            Vec::new()
        };
        for key in urgent {
            push_unique(&mut closing, key);
        }
        for (scope, thread) in &closing {
            self.flush(scope, thread);
        }
        Ok(touched.len())
    }

    /// Close one window now; returns the event, when there was one.
    pub fn flush(&mut self, scope: &str, thread: &str) -> Option<WakeEvent> {
        let at = self
            .groups
            .iter()
            .position(|g| g.key.0 == scope && g.key.1 == thread)?;
        let group = self.groups.remove(at);
        self.flush_rows(scope, thread, &group.rows)
    }

    fn flush_rows(&mut self, scope: &str, thread: &str, rows: &[WakeRow]) -> Option<WakeEvent> {
        let (lead, others) = rows.split_first()?;
        let texts: Vec<&str> = rows.iter().map(|r| r.text.as_str()).collect();
        let lines = lines_of(&texts);
        let event = WakeEvent {
            id: lead.id.clone(),
            scope: scope.to_owned(),
            thread: thread.to_owned(),
            at: lead.at.clone(),
            lines: lines.clone(),
            count: count(rows.len()),
        };
        let (sent, errors) = self.listeners.deliver(scope, thread, &event);
        let done = FinishedEvent {
            lines,
            count: event.count,
            status: if sent > 0 {
                WakeStatus::Delivered
            } else {
                WakeStatus::Queued
            },
            listeners: sent,
            delivered_at: (sent > 0).then(|| iso(self.clock.now_ms())),
        };
        if let Err(err) = self.store.finish(lead, others, &done) {
            warn!(scope, thread, %err, "wake event not recorded");
        }
        self.mark_woken(scope, thread);
        self.mirror(scope);
        self.note(EngineNote::Flushed {
            scope: scope.to_owned(),
            thread: thread.to_owned(),
            id: lead.id.clone(),
            at: lead.at.clone(),
            reason: lead.reason.clone(),
            text: lead.text.clone(),
            sent,
        });
        for err in errors {
            self.note(EngineNote::DeliveryError {
                scope: scope.to_owned(),
                thread: thread.to_owned(),
                id: lead.id.clone(),
                err,
            });
        }
        Some(event)
    }

    /// Close every window whose deadline passed, oldest deadline first; returns how many closed.
    pub fn tick(&mut self) -> usize {
        let now = self.clock.now_ms();
        let mut due: Vec<(i64, Key)> = self
            .groups
            .iter()
            .filter_map(|g| g.deadline.filter(|d| *d <= now).map(|d| (d, g.key.clone())))
            .collect();
        due.sort_by_key(|(deadline, _)| *deadline);
        for (_, (scope, thread)) in &due {
            self.flush(scope, thread);
        }
        due.len()
    }

    /// The earliest moment a window closes, when one is open.
    #[must_use]
    pub fn next_deadline(&self) -> Option<i64> {
        self.groups.iter().filter_map(|g| g.deadline).min()
    }

    /// Close every open window now.
    pub fn flush_all(&mut self) {
        let keys: Vec<Key> = self.groups.iter().map(|g| g.key.clone()).collect();
        for (scope, thread) in &keys {
            self.flush(scope, thread);
        }
    }

    /// Stop: the open windows close, delivering to whoever still listens.
    pub fn stop(&mut self) {
        self.flush_all();
    }

    /// How many wakes wait in open windows.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.groups.iter().map(|g| g.rows.len()).sum()
    }

    /// Boot: windows left open when the server stopped close now, one event per thread; old rows go.
    /// Returns how many windows were closed.
    ///
    /// # Errors
    /// When the store cannot be read.
    pub fn recover(&mut self) -> StoreResult<usize> {
        let mut by_key: Vec<(Key, Vec<WakeRow>)> = Vec::new();
        for row in self.store.unflushed()? {
            let key = (row.scope.clone(), row.thread.clone());
            if self.groups.iter().any(|g| g.key == key) {
                continue;
            }
            if is_passive(&row.reason) {
                self.shelve(&row);
                continue;
            }
            match by_key.iter_mut().find(|(k, _)| *k == key) {
                Some((_, rows)) => rows.push(row),
                None => by_key.push((key, vec![row])),
            }
        }
        for ((scope, thread), rows) in &by_key {
            self.flush_rows(scope, thread, rows);
        }
        self.prune()?;
        Ok(by_key.len())
    }

    /// Delete rows older than a week and refresh the mirrors that lost some.
    ///
    /// # Errors
    /// When the store cannot delete.
    pub fn prune(&mut self) -> StoreResult<()> {
        for scope in self.store.prune(self.clock.now_ms())? {
            self.mirror(&scope);
        }
        Ok(())
    }

    /// Events nobody received since `since_iso` (for the registry after a restart); a passive one
    /// queued before the wake policy is not one.
    #[must_use]
    pub fn queued_since(&self, scope: &str, since_iso: &str) -> Vec<WakeRow> {
        let mut rows = self
            .store
            .queued_events(scope, "*", since_iso)
            .unwrap_or_default();
        rows.retain(|r| !is_passive(&r.reason));
        rows
    }

    /// Every project that has wake rows.
    #[must_use]
    pub fn known_scopes(&self) -> Vec<String> {
        self.store.scopes().unwrap_or_default()
    }

    /// Listeners now: how many per project and thread key.
    #[must_use]
    pub fn listener_stats(&self) -> Vec<ListenerStat> {
        self.listeners.stats()
    }

    /// How many listeners (legacy ones too) a project has.
    #[must_use]
    pub fn listener_total(&self, scope: &str) -> usize {
        self.listeners.total(scope)
    }

    /// Is there a v2 listener for this exact project and thread key? A new one would replace it.
    #[must_use]
    pub fn has_listener(&self, scope: &str, thread: &str) -> bool {
        self.listeners.has_v2(scope, thread)
    }

    /// The connections of one project.
    #[must_use]
    pub fn listeners_of(&self, scope: &str) -> Vec<ListenerInfo> {
        self.listeners.info(scope)
    }
}
