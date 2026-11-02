//! The stall watch: a lead is idle only when its board is empty, and a wake nobody read is not done.
//!
//! One tick per `stall.tickMs` derives every stall again from the boards and the wakes table (rule 2 in
//! `unread`, rules 3 to 5 in [`crate::stall_rules`]), diffs them against the last tick in the book, and
//! returns what to do: the wakes (a second wake to a thread that has not read, the stall line for the
//! Overlord), the activity lines, the notices for the owner. The page documents are written here.
//!
//! Only the inbox reads and the acks are stored, in a kv the server plugs in; the open set is never stored.

mod ackable;
mod acked;
mod book;
mod config;
mod docs;
mod env;
mod key;
mod raise;
mod sources;
mod unread;

use std::collections::HashSet;
use std::sync::Arc;

use tracing::warn;

pub use ackable::{ACKABLE, is_ackable};
pub use config::StallConfig;
pub use docs::StallInfo;
pub use env::StallEnv;
pub use sources::{BusySource, MemoryKv, NoBusy, StallKv};

use crate::activity::ActivityLine;
use crate::docs::{Doc, DocSink};
use crate::reasons::PORTFOLIO;
use crate::reminders::{OpenNotice, OwnerNotice};
use crate::source::BoardSource;
use crate::stall_rules::{self, Found, IdleWatch, Kind, RuleInput};
use crate::time::iso;
use crate::wake::WakeInput;
use book::{Ack, StallBook};

/// The collection of `_portfolio` the page reads the stalls from.
pub const COLLECTION: &str = "stalls";

/// What one tick asks the service to do.
#[derive(Debug, Default)]
pub struct StallTick {
    /// The wakes to send: a thread's second wake, the Overlord's stall lines.
    pub wakes: Vec<WakeInput>,
    /// The lines for the activity log.
    pub activity: Vec<ActivityLine>,
    /// The notices for the owner, each due once.
    pub owner: Vec<OwnerNotice>,
}

type Told = (String, String, String);

/// The stall watch.
pub struct Stalls {
    config: StallConfig,
    docs: Arc<dyn DocSink>,
    busy: Arc<dyn BusySource>,
    kv: Arc<dyn StallKv>,
    idle: IdleWatch,
    book: StallBook,
    told: HashSet<Told>,
    written: Option<Vec<Doc>>,
}

impl Stalls {
    /// A watch that sees no running process and keeps its kv in memory until the server plugs its own in.
    #[must_use]
    pub fn new(config: StallConfig, docs: Arc<dyn DocSink>) -> Self {
        Self {
            config,
            docs,
            busy: Arc::new(NoBusy),
            kv: Arc::new(MemoryKv::new()),
            idle: IdleWatch::new(),
            book: StallBook::default(),
            told: HashSet::new(),
            written: None,
        }
    }

    /// The settings.
    #[must_use]
    pub fn config(&self) -> &StallConfig {
        &self.config
    }

    /// Plug in the running processes and the kv.
    pub fn set_sources(&mut self, busy: Arc<dyn BusySource>, kv: Arc<dyn StallKv>) {
        self.busy = busy;
        self.kv = kv;
    }

    /// A thread read its own inbox at `now` (epoch ms): the wakes it was sent before are acknowledged.
    pub fn inbox_read(&self, scope: &str, thread: &str, now: i64) {
        self.kv.set(&key::read_key(scope, thread), &now.to_string());
    }

    /// The page documents were wiped by a hard resync: write them again on the next tick.
    pub fn forget_docs(&mut self) {
        self.written = None;
    }

    /// One pass: find the stalls, diff them against the last pass, write the page documents, say what to send.
    pub fn tick(&mut self, now: i64, env: &StallEnv<'_>) -> StallTick {
        if !self.config.enabled {
            return StallTick::default();
        }
        let ladder = unread::scan(now, env, &self.config, self.kv.as_ref());
        let mut out = StallTick {
            wakes: ladder.nudges,
            owner: self.owner_once(ladder.owner),
            ..StallTick::default()
        };
        let found = self.found(now, env.boards, ladder.stalls, &ladder.deps);
        let gone = self.book.update(now, found, self.kv.as_ref());
        raise::due(now, env, &self.config, &self.book, &mut out);
        raise::cleared(now, &gone, &mut out);
        self.write_docs();
        out
    }

    /// The open stalls, of one project or of all.
    #[must_use]
    pub fn open(&self, project: Option<&str>) -> Vec<StallInfo> {
        self.book
            .all()
            .filter(|open| project.is_none_or(|p| open.found.project == p))
            .map(StallInfo::of)
            .collect()
    }

    /// The Overlord acknowledges every open stall of a project; the page documents change at once.
    /// Returns the stalls acknowledged.
    pub fn ack(&mut self, project: &str, by: &str, note: &str, now: i64) -> Vec<StallInfo> {
        let ack = Ack {
            at: iso(now),
            by: by.to_owned(),
            note: note.to_owned(),
        };
        let acked = self.book.ack_project(project, &ack, self.kv.as_ref());
        self.write_docs();
        acked.iter().map(StallInfo::of).collect()
    }

    /// The unacknowledged stalls as notices, for the Overlord's coalesced line.
    #[must_use]
    pub fn open_notices(&self) -> Vec<OpenNotice> {
        self.book
            .unacked()
            .map(|open| OpenNotice {
                reason: raise::REASON.to_owned(),
                project: open.found.project.clone(),
                subject: open.key.clone(),
            })
            .collect()
    }

    /// The stalls of rule 2 and of rules 3 to 5. A dependency an `unread` stall covers has no `unstarted` one.
    fn found(
        &self,
        now: i64,
        boards: &dyn BoardSource,
        mut found: Vec<Found>,
        covered: &HashSet<String>,
    ) -> Vec<Found> {
        let busy = |project: &str| self.busy.busy(project);
        let input = RuleInput {
            boards,
            busy: &busy,
            idle: &self.idle,
            limits: self.config.limits(),
        };
        found.extend(
            stall_rules::find(now, &input)
                .into_iter()
                .filter(|f| !(f.kind == Kind::Unstarted && covered.contains(&f.subject))),
        );
        found
    }

    /// The notices not told yet; one told long ago is forgotten once it is no longer due.
    fn owner_once(&mut self, due: Vec<OwnerNotice>) -> Vec<OwnerNotice> {
        let now_due: HashSet<Told> = due.iter().map(told_of).collect();
        self.told.retain(|key| now_due.contains(key));
        due.into_iter()
            .filter(|notice| self.told.insert(told_of(notice)))
            .collect()
    }

    fn write_docs(&mut self) {
        let docs: Vec<Doc> = self
            .book
            .all()
            .map(|open| StallInfo::of(open).to_doc())
            .collect();
        if self.written.as_ref() == Some(&docs) {
            return;
        }
        match self.docs.replace(PORTFOLIO, COLLECTION, docs.clone()) {
            Ok(()) => self.written = Some(docs),
            Err(err) => warn!(%err, "stall documents not written"),
        }
    }
}

fn told_of(notice: &OwnerNotice) -> Told {
    (
        notice.project.clone(),
        notice.thread.clone(),
        notice.since.clone(),
    )
}
