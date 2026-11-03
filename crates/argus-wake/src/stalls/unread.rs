//! Rule 2, the acknowledgement ladder of a wake that asks a thread to act. From the age of the wake row:
//!
//! | age | step |
//! |---|---|
//! | `nudge_ms` (5 min) | the same wake again to the same thread, `ack-nudge`, text `2nd: ...` |
//! | `unread_ms` (10 min) | an `unread` stall for the Overlord (never about the Overlord itself) |
//! | `owner_ms` (30 min) | a notice for the owner |
//!
//! A `coord lead` note runs the first two steps sooner: `lead_nudge_ms` (3 min) and `lead_unread_ms` (5 min).
//!
//! Only wakes made at or after `since`, the start of the stall watch, are counted: the history from before
//! it has no read stamps and is never unread.
//!
//! Each wake row is read at most once a tick and nothing here writes: the caller sends what comes back.

use std::collections::HashSet;

use super::ackable::{is_ackable, is_lead_note};
use super::acked::Acks;
use super::config::StallConfig;
use super::env::StallEnv;
use super::sources::StallKv;
use crate::board::BoardView;
use crate::reasons::{MAIN_SCOPE, PORTFOLIO};
use crate::reminders::{OwnerNotice, TOPIC_STALL};
use crate::stall_rules::{Found, Kind};
use crate::time::{iso, ms_or_zero};
use crate::wake::{WakeInput, WakeRow};

const NUDGE_REASON: &str = "ack-nudge";
const MIN_MS: i64 = 60_000;
const NOTE_CLIP: usize = 80;

/// What the ladder found at one moment.
#[derive(Default)]
pub(super) struct Unread {
    /// The wakes sent again.
    pub(super) nudges: Vec<WakeInput>,
    /// The stalls for the Overlord.
    pub(super) stalls: Vec<Found>,
    /// The notices for the owner, every one that is due (the caller sends each once).
    pub(super) owner: Vec<OwnerNotice>,
    /// The dependencies an `unread` stall covers, so that rule 3 stays quiet about them.
    pub(super) deps: HashSet<String>,
}

struct Scan<'a> {
    now: i64,
    since: i64,
    env: &'a StallEnv<'a>,
    config: &'a StallConfig,
    acks: Acks<'a>,
}

/// The ladder over the wake rows of every project, the Overlord's included, made at or after `since`
/// (epoch ms).
pub(super) fn scan(
    now: i64,
    since: i64,
    env: &StallEnv<'_>,
    config: &StallConfig,
    kv: &dyn StallKv,
) -> Unread {
    let ledger = env.boards.board(PORTFOLIO);
    let scan = Scan {
        now,
        since,
        env,
        config,
        acks: Acks {
            kv,
            ledger: ledger.as_deref(),
        },
    };
    let mut out = Unread::default();
    for scope in env.boards.scopes().into_iter().filter(|s| s != PORTFOLIO) {
        if let Some(board) = env.boards.board(&scope) {
            scan.project(&scope, &board, &mut out);
        }
    }
    out
}

impl Scan<'_> {
    fn project(&self, scope: &str, board: &BoardView, out: &mut Unread) {
        for row in self.env.rows(scope) {
            let at = ms_or_zero(Some(row.at.as_str()));
            if self.waiting(board, &row, at) {
                self.ladder(board, &row, at, out);
            }
        }
    }

    /// An ackable wake old enough for the first step, young enough to matter, that nobody acknowledged.
    fn waiting(&self, board: &BoardView, row: &WakeRow, at: i64) -> bool {
        let age = self.now - at;
        let (nudge_ms, _) = self.config.unread_steps(&row.reason);
        is_ackable(&row.reason)
            && at >= self.since
            && (nudge_ms..=self.config.horizon_ms).contains(&age)
            && !self.acks.covers(board, row, at)
    }

    fn ladder(&self, board: &BoardView, row: &WakeRow, at: i64, out: &mut Unread) {
        let age = self.now - at;
        let in_turn = board
            .threads
            .get(&row.thread)
            .is_some_and(|t| t.in_turn(self.now));
        let line = line(row, age, in_turn);
        self.nudge(row, out);
        if age >= self.config.owner_ms {
            out.owner.push(notice(row, age, at, &line));
        }
        let (_, unread_ms) = self.config.unread_steps(&row.reason);
        if age >= unread_ms && row.scope != MAIN_SCOPE {
            out.deps.extend(dep_of(row));
            out.stalls.push(found(row, at, line));
        }
    }

    fn nudge(&self, row: &WakeRow, out: &mut Unread) {
        if self
            .env
            .last_sent(&row.scope, &row.thread, NUDGE_REASON, &row.id)
            == 0
        {
            out.nudges.push(WakeInput::about(
                &row.scope,
                &row.thread,
                NUDGE_REASON,
                &row.id,
                format!("2nd: {}", row.text),
            ));
        }
    }
}

fn found(row: &WakeRow, since: i64, text: String) -> Found {
    Found {
        kind: Kind::Unread,
        project: row.scope.clone(),
        lead: row.thread.clone(),
        subject: row.id.clone(),
        since,
        text,
    }
}

fn notice(row: &WakeRow, age: i64, at: i64, line: &str) -> OwnerNotice {
    OwnerNotice {
        title: format!("Unread for {} min", age / MIN_MS),
        text: line.to_owned(),
        project: row.scope.clone(),
        thread: row.thread.clone(),
        since: iso(at),
        topic: TOPIC_STALL.to_owned(),
    }
}

/// The dependency a `dep-request` row is about (its subject is `D-n#seq`).
fn dep_of(row: &WakeRow) -> Option<String> {
    if row.reason != "dep-request" {
        return None;
    }
    let subject = row.subject.as_deref()?;
    Some(subject.split('#').next().unwrap_or(subject).to_owned())
}

/// What the wake was about: `D-11` for a dependency request, the note for a `coord lead` note, else
/// `<reason> <subject>`.
fn label(row: &WakeRow) -> String {
    if let Some(dep) = dep_of(row) {
        return dep;
    }
    if is_lead_note(&row.reason) {
        let note: String = row.text.chars().take(NOTE_CLIP).collect();
        return format!("the lead note \"{note}\"");
    }
    match row.subject.as_deref() {
        Some(subject) => format!("{} {subject}", row.reason),
        None => row.reason.clone(),
    }
}

/// `stall: rotp did not read D-11 for 10 min (it is in a turn)`
fn line(row: &WakeRow, age: i64, in_turn: bool) -> String {
    let tail = if in_turn {
        "it is in a turn"
    } else {
        "it is idle"
    };
    format!(
        "stall: {} did not read {} for {} min ({tail})",
        row.scope,
        label(row),
        age / MIN_MS
    )
}
