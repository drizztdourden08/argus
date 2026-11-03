//! Board presence: which threads have their board open and live in a browser tab right now.
//!
//! A board page opened with `?thread=<id>` holds one event stream for as long as the tab is loaded;
//! closing the tab or a dead page drops it. A thread is "open" while at least one such stream is
//! connected. A page without `?thread=` (the owner's tab) claims nothing. The registry raises a
//! `board-closed` fault when a live thread has no open board for longer than its grace.

use indexmap::IndexMap;
use serde::Serialize;

use crate::time::iso;

/// How many tabs one thread may hold open; the oldest is dropped beyond it.
pub const MAX_TABS_PER_THREAD: usize = 8;

type Key = (String, String);

struct Tab {
    since: i64,
    seq: u64,
}

/// The claim one open tab holds; give it back to [`Presence::detach`] when the stream closes.
#[derive(Debug, PartialEq, Eq)]
pub struct PresenceToken {
    key: Key,
    tab: String,
    seq: u64,
}

impl PresenceToken {
    /// The project the tab belongs to.
    #[must_use]
    pub fn scope(&self) -> &str {
        &self.key.0
    }
}

/// A thread's open board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenBoard {
    /// The earliest open tab's connect time (epoch ms).
    pub since: i64,
    /// How many tabs are open.
    pub tabs: usize,
}

/// A row of the presence listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PresenceRow {
    /// Project key.
    pub project: String,
    /// The thread.
    pub thread: String,
    /// Open tabs.
    pub tabs: usize,
    /// The earliest open tab's connect time (ISO).
    pub since: String,
}

/// The open board tabs of every thread.
#[derive(Default)]
pub struct Presence {
    tabs: IndexMap<Key, IndexMap<String, Tab>>,
    seq: u64,
}

impl Presence {
    /// No tabs open.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one tab; a tab id that reconnects replaces its own entry.
    pub fn attach(&mut self, scope: &str, thread: &str, tab: &str, now_ms: i64) -> PresenceToken {
        self.seq += 1;
        let key = (scope.to_owned(), thread.to_owned());
        let set = self.tabs.entry(key.clone()).or_default();
        if !set.contains_key(tab) && set.len() >= MAX_TABS_PER_THREAD {
            set.shift_remove_index(0);
        }
        set.insert(
            tab.to_owned(),
            Tab {
                since: now_ms,
                seq: self.seq,
            },
        );
        PresenceToken {
            key,
            tab: tab.to_owned(),
            seq: self.seq,
        }
    }

    /// Whether the tab may register on the thread: it already holds a place there (a renewal
    /// replaces its entry, whichever stream it came from, so it never counts twice), or the thread
    /// has fewer than [`MAX_TABS_PER_THREAD`] tabs.
    #[must_use]
    pub fn admits(&self, scope: &str, thread: &str, tab: &str) -> bool {
        let set = self.tabs.get(&(scope.to_owned(), thread.to_owned()));
        set.is_none_or(|s| s.contains_key(tab) || s.len() < MAX_TABS_PER_THREAD)
    }

    /// A tab's stream closed. A tab that already reconnected keeps its newer entry.
    pub fn detach(&mut self, token: &PresenceToken) {
        let Some(set) = self.tabs.get_mut(&token.key) else {
            return;
        };
        if set.get(&token.tab).is_some_and(|t| t.seq == token.seq) {
            set.shift_remove(&token.tab);
        }
        if set.is_empty() {
            self.tabs.shift_remove(&token.key);
        }
    }

    /// The thread's open board, when it has one.
    #[must_use]
    pub fn open_of(&self, scope: &str, thread: &str) -> Option<OpenBoard> {
        let set = self.tabs.get(&(scope.to_owned(), thread.to_owned()))?;
        let since = set.values().map(|t| t.since).min()?;
        Some(OpenBoard {
            since,
            tabs: set.len(),
        })
    }

    /// Every thread with an open board.
    #[must_use]
    pub fn list(&self) -> Vec<PresenceRow> {
        self.tabs
            .iter()
            .filter_map(|((scope, thread), set)| {
                let since = set.values().map(|t| t.since).min()?;
                Some(PresenceRow {
                    project: scope.clone(),
                    thread: thread.clone(),
                    tabs: set.len(),
                    since: iso(since),
                })
            })
            .collect()
    }
}
