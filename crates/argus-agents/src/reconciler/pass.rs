//! One evaluation of one board at one moment: the shared state of the five steps.

use super::Env;
use super::report::{Class, Counts, Finding};
use super::session::Session;
use crate::board::Board;
use crate::ports::WakeEvent;
use crate::record::{
    LIVE_STATUSES, Patch, Patches, Record, claude_id_of, id_of, is_set, ms_of, text,
};
use crate::text::clip;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;

pub(super) const ORPHAN_KEYS: [&str; 3] = ["orphanSince", "orphanWokeAt", "orphanRev"];
pub(super) const IDLE_KEYS: [&str; 2] = ["idleSince", "idleWokeAt"];

/// What an evaluation decided.
pub(super) struct Outcome {
    pub counts: Counts,
    pub findings: Vec<Finding>,
    pub patches: Patches,
    pub wakes: Vec<WakeEvent>,
    pub lost: Vec<String>,
}

/// The state the steps share. The board is only read; the decisions are patches.
pub(super) struct Pass<'a> {
    pub env: &'a Env<'a>,
    pub project: &'a str,
    pub board: &'a Board,
    pub at: i64,
    pub can_wake: bool,
    pub all: Vec<&'a Record>,
    pub counts: Counts,
    pub patches: Patches,
    pub wakes: Vec<WakeEvent>,
    pub lost: Vec<String>,
    pub closed_here: HashSet<String>,
    findings: Vec<Finding>,
    sessions: HashMap<String, Rc<Session<'a>>>,
}

impl<'a> Pass<'a> {
    pub(super) fn new(
        env: &'a Env<'a>,
        project: &'a str,
        board: &'a Board,
        at: i64,
        can_wake: bool,
    ) -> Self {
        let all = board
            .agents
            .iter()
            .filter(|a| !is_set(a, "archivedAt") && !is_set(a, "deletedAt"))
            .collect();
        Self {
            env,
            project,
            board,
            at,
            can_wake,
            all,
            counts: Counts::default(),
            patches: Patches::default(),
            wakes: Vec::new(),
            lost: Vec::new(),
            closed_here: HashSet::new(),
            findings: Vec::new(),
            sessions: HashMap::new(),
        }
    }

    /// Runs the five steps in order.
    pub(super) fn run(&mut self) {
        self.verify_ids();
        self.link();
        self.dedupe();
        self.classify();
        self.resume();
    }

    pub(super) fn finish(self) -> Outcome {
        Outcome {
            counts: self.counts,
            findings: self.findings,
            patches: self.patches,
            wakes: self.wakes,
            lost: self.lost,
        }
    }

    /// The record as it will be once the decisions so far are written.
    pub(super) fn cur(&self, agent: &Record) -> Record {
        self.patches.view(agent)
    }

    pub(super) fn patch(&mut self, agent: &Record, patch: Patch) {
        self.patches.add(id_of(agent), patch);
    }

    pub(super) fn clear_flags(&mut self, agent: &Record, keys: &[&str]) {
        let current = self.cur(agent);
        let gone: Vec<&str> = keys
            .iter()
            .copied()
            .filter(|k| current.contains_key(*k))
            .collect();
        if !gone.is_empty() {
            self.patch(agent, Patch::new().without_all(&gone));
        }
    }

    pub(super) fn is_open(agent: &Record) -> bool {
        text(agent, "status").is_some_and(|s| LIVE_STATUSES.contains(&s))
    }

    pub(super) fn recently_closed(&self, agent: &Record) -> bool {
        let closed = matches!(text(agent, "status"), Some("done" | "stopped"));
        let finished = ms_of(agent, "finishedAt").unwrap_or(0);
        closed
            && !is_set(agent, "duplicateOf")
            && !is_set(agent, "lost")
            && self.at - finished <= self.env.opts.reopen_ms
    }

    pub(super) fn exists(&self, path: Option<&str>) -> bool {
        path.is_some_and(|p| self.env.fs.stat(Path::new(p)).is_some())
    }

    pub(super) fn finding(&mut self, agent: &Record, class: Class, detail: String) {
        if class == Class::Alive || self.findings.len() >= self.env.opts.max_findings {
            return;
        }
        self.findings.push(Finding {
            project: self.project.to_owned(),
            id: id_of(agent).to_owned(),
            name: text(agent, "name").map(str::to_owned),
            thread: text(agent, "thread").map(str::to_owned),
            class,
            claude_id: claude_id_of(&self.cur(agent)),
            detail,
        });
    }

    pub(super) fn session(&mut self, agent: &Record) -> Rc<Session<'a>> {
        self.session_of(text(agent, "thread"))
    }

    pub(super) fn session_of(&mut self, thread: Option<&str>) -> Rc<Session<'a>> {
        let key = thread.unwrap_or("").to_owned();
        if let Some(found) = self.sessions.get(&key) {
            return Rc::clone(found);
        }
        let session = Rc::new(Session::open(self.env, self.board, thread));
        self.sessions.insert(key, Rc::clone(&session));
        session
    }

    /// Queues a wake for the thread that owns the record. False when nobody can be told.
    pub(super) fn wake(&mut self, agent: &Record, reason: &str, text: String) -> bool {
        let Some(thread) = self.board.wake_target(agent) else {
            return false;
        };
        self.wakes.push(WakeEvent {
            scope: self.project.to_owned(),
            thread,
            reason: reason.to_owned(),
            subject: id_of(agent).to_owned(),
            text,
        });
        self.counts.woken += 1;
        true
    }

    /// `A-44 controllers-opus (opus`, the start of every wake sentence.
    pub(super) fn who(agent: &Record) -> String {
        let name = text(agent, "name")
            .filter(|n| !n.is_empty())
            .map_or_else(String::new, |n| format!("{} ", clip(n, 40)));
        let model = text(agent, "modelUsed")
            .or_else(|| text(agent, "model"))
            .unwrap_or("model unknown");
        format!("{} {name}({model}", id_of(agent))
    }
}
