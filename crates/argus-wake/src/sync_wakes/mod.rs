//! A wake about the daily remote sync: the owner's standard is that each lead runs the full
//! sync (push, pull requests, packages, releases, deploys) and then `coord remote synced`, which
//! records `syncs.<project> = { at, by, note }` in `remote.json`.
//!
//! * `remote-sync-due`: a project whose lead thread is registered and not ended has gone
//!   `due_ms` (24 h) without a record, counted from its last record or, when it never had one,
//!   from the day the lead registered. The lead gets ONE wake, and the same again every
//!   `repeat_ms` (24 h) while the sync is still due. A new record ends it.
//! * Each new record leaves one line in the activity log (kind `remote-sync`), and each due wake
//!   one line (kind `remote-sync-due`).
//!
//! The last wake is asked of the wakes table, so a restart does not send it again at once.
//! The Overlord's own scope has no repository to sync and is left out.

mod due;
mod text;

pub use due::SyncDue;
pub use text::{ago_text, due_text};

use argus_core::lead::LEAD_ROLE;
use std::collections::HashMap;

use argus_core::remote::SyncRecord;
use serde_json::Value;

use crate::activity::{ActivityLine, Level};
use crate::board::BoardView;
use crate::config::section_ms;
use crate::item_wakes::LastSent;
use crate::reasons::PORTFOLIO;
use crate::source::{BoardSource, SyncSource};
use crate::time::parse_ms;
use crate::wake::WakeInput;

/// The reason of the wake.
pub const REASON: &str = "remote-sync-due";

/// The kind of the activity line of a new sync record.
pub const SYNC_KIND: &str = "remote-sync";

const DAY: i64 = 24 * 3_600_000;

/// The timing of the remote sync wake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncWakeConfig {
    /// How often the owner calls `tick`.
    pub tick_ms: i64,
    /// A sync is due this long after the last one.
    pub due_ms: i64,
    /// A lead is woken again after this long while the sync is still due.
    pub repeat_ms: i64,
}

impl Default for SyncWakeConfig {
    fn default() -> Self {
        Self {
            tick_ms: 60_000,
            due_ms: DAY,
            repeat_ms: DAY,
        }
    }
}

impl SyncWakeConfig {
    /// The settings from the `remoteSync` section of config.json; a value that is not a positive
    /// number keeps its default.
    #[must_use]
    pub fn from_json(section: &Value) -> Self {
        let d = Self::default();
        Self {
            tick_ms: section_ms(section, "tickMs", d.tick_ms),
            due_ms: section_ms(section, "dueMs", d.due_ms),
            repeat_ms: section_ms(section, "repeatMs", d.repeat_ms),
        }
    }
}

/// What a tick reads.
pub struct SyncEnv<'a> {
    /// The project boards.
    pub boards: &'a dyn BoardSource,
    /// The sync records.
    pub syncs: &'a dyn SyncSource,
    /// The last wake the wakes table knows, so a restart does not send it again at once.
    pub last_sent: LastSent<'a>,
}

/// What one tick produced.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncTick {
    /// The wakes to submit.
    pub wakes: Vec<WakeInput>,
    /// The activity lines.
    pub activity: Vec<ActivityLine>,
}

struct Lead {
    id: String,
    registered_ms: Option<i64>,
}

/// The first lead thread that is registered and has not ended.
fn lead_of(board: &BoardView) -> Option<Lead> {
    let (key, thread) = board
        .threads
        .iter()
        .find(|(_, t)| t.role.is(LEAD_ROLE) && t.is_live())?;
    Some(Lead {
        id: thread.id.some().map_or_else(|| key.clone(), str::to_owned),
        registered_ms: thread.registered_at.some().and_then(parse_ms),
    })
}

fn tracked(scope: &str, overlord: &str) -> bool {
    scope != PORTFOLIO && scope != overlord
}

fn sync_line(project: &str, record: &SyncRecord) -> ActivityLine {
    let by = record
        .by
        .as_deref()
        .filter(|b| !b.is_empty())
        .map_or_else(String::new, |b| format!(" by {b}"));
    let note = if record.note.is_empty() {
        String::new()
    } else {
        format!(": {}", record.note)
    };
    ActivityLine {
        kind: SYNC_KIND.to_owned(),
        level: Level::Info,
        text: format!("{project}: remote sync recorded{by}{note}"),
        project: Some(project.to_owned()),
        thread: None,
    }
}

/// The remote sync wake.
pub struct SyncWakes {
    config: SyncWakeConfig,
    started_at: i64,
    seen: HashMap<String, i64>,
    sent: HashMap<String, i64>,
}

impl SyncWakes {
    /// The wake, started at `started_at`: a record older than that is not news.
    #[must_use]
    pub fn new(config: SyncWakeConfig, started_at: i64) -> Self {
        Self {
            config,
            started_at,
            seen: HashMap::new(),
            sent: HashMap::new(),
        }
    }

    /// The timing.
    #[must_use]
    pub fn config(&self) -> &SyncWakeConfig {
        &self.config
    }

    /// One line for each record newer than the one seen before.
    fn log_new(&mut self, records: &HashMap<String, SyncRecord>, out: &mut SyncTick) {
        let mut keys: Vec<&String> = records.keys().collect();
        keys.sort();
        for key in keys {
            let record = &records[key];
            let Some(at) = record.at_ms() else {
                continue;
            };
            let before = self.seen.insert(key.clone(), at);
            if at > before.unwrap_or(self.started_at) {
                out.activity.push(sync_line(key, record));
            }
        }
    }

    fn last_of(&self, env: &SyncEnv<'_>, scope: &str, lead: &str) -> i64 {
        let stored = (env.last_sent)(scope, lead, REASON, scope);
        self.sent
            .get(&format!("{scope}/{lead}"))
            .copied()
            .unwrap_or(0)
            .max(stored)
    }

    fn check(
        &mut self,
        now: i64,
        scope: &str,
        records: &HashMap<String, SyncRecord>,
        env: &SyncEnv<'_>,
        out: &mut SyncTick,
    ) {
        let Some(lead) = env.boards.board(scope).as_deref().and_then(lead_of) else {
            return;
        };
        let synced = records.get(scope).and_then(SyncRecord::at_ms);
        let since = synced.or(lead.registered_ms).unwrap_or(self.started_at);
        let waiting = now - since >= self.config.due_ms;
        if !waiting || now - self.last_of(env, scope, &lead.id) < self.config.repeat_ms {
            return;
        }
        let ago = ago_text(synced.map(|at| now - at));
        out.activity.push(ActivityLine {
            kind: REASON.to_owned(),
            level: Level::Info,
            text: format!("{scope}/{}: remote sync due, last {ago}", lead.id),
            project: Some(scope.to_owned()),
            thread: Some(lead.id.clone()),
        });
        out.wakes.push(WakeInput::about(
            scope,
            &lead.id,
            REASON,
            scope,
            due_text(&ago),
        ));
        self.sent.insert(format!("{scope}/{}", lead.id), now);
    }

    /// The wakes and the activity lines due now.
    pub fn tick(&mut self, now: i64, env: &SyncEnv<'_>) -> SyncTick {
        let records: HashMap<String, SyncRecord> = env.syncs.syncs().into_iter().collect();
        let mut out = SyncTick::default();
        self.log_new(&records, &mut out);
        let mut scopes = env.boards.scopes();
        scopes.sort();
        let overlord = env.boards.overlord();
        for scope in scopes.iter().filter(|s| tracked(s, &overlord)) {
            self.check(now, scope, &records, env, &mut out);
        }
        out
    }
}
