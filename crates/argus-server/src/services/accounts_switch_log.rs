//! The step log of a switch: every step the process takes, from the countdown and the handover
//! cycle to the checks after Claude started, as the `steps` of the `switch` state doc
//! (`{ at, stage, state, text, detail? }`, oldest first), with the leads of the handover
//! (`threads`) and what the sync did (`sync`). Every writer of that doc goes through
//! [`AccountsService::put_switch`], so the log is never lost between writers, and a step noted
//! while a switch is under way rewrites the doc at once. `contract` gives the doc its shape.
//!
//! A log belongs to one process. A doc that ends it (done, cancelled, failed or carrying an
//! error) closes the log; the next step or the next live doc starts a new one, and so does a doc
//! of another switch. Resume lines come after the end and join the finished log instead.

mod contract;
mod line;
mod lines;

use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_accounts::handover::Cycle;
use argus_core::Coll;
use argus_core::clock::now_ms;
use serde_json::{Value, json};

pub(crate) use contract::sync_info;
pub(crate) use line::{Line, LineState, Stage};
pub(crate) use lines::{
    CycleLines, cycle_lines, sidebar_check_line, stage_of, step_lines, sync_note, threads_of,
};

use super::AccountsService;

const SWITCH_DOC: &str = "switch";
const ENDED: [&str; 4] = ["idle", "done", "cancelled", "failed"];

/// Whether a `switch` doc ends its process.
fn ended(doc: &Value) -> bool {
    let phase = doc["phase"].as_str().unwrap_or("idle");
    ENDED.contains(&phase) || !doc["error"].is_null()
}

/// The log of the process and the last doc a writer gave.
#[derive(Default)]
pub(crate) struct LogState {
    entries: Vec<Value>,
    threads: Vec<Value>,
    sync: Value,
    closed: bool,
    doc: Option<Value>,
}

impl LogState {
    fn restart(&mut self) {
        self.entries.clear();
        self.threads.clear();
        self.sync = Value::Null;
        self.closed = false;
        self.doc = None;
    }

    fn push(&mut self, at: i64, lines: &[Line]) {
        self.entries
            .extend(lines.iter().map(|line| contract::entry(at, line)));
    }

    /// Notes steps of the process; a closed log starts over first.
    pub(crate) fn add(&mut self, at: i64, lines: &[Line]) {
        if lines.is_empty() {
            return;
        }
        if self.closed {
            self.restart();
        }
        self.push(at, lines);
    }

    /// Notes steps that follow the end, such as a resume: they join the last log.
    pub(crate) fn add_after(&mut self, at: i64, lines: &[Line]) {
        self.push(at, lines);
    }

    /// The leads of the handover cycle of the process.
    pub(crate) fn set_threads(&mut self, threads: Vec<Value>) {
        self.threads = threads;
    }

    /// What the sync of the process wrote ([`sync_info`]).
    pub(crate) fn set_sync(&mut self, info: Value) {
        self.sync = info;
    }

    /// A doc a writer gives: a live doc on a closed log, or a doc of another switch while one
    /// was under way, starts a new log.
    pub(crate) fn begin(&mut self, doc: &Value) {
        let other = !self.closed && self.doc.as_ref().is_some_and(|d| d["id"] != doc["id"]);
        if (self.closed && !ended(doc)) || other {
            self.restart();
        }
    }

    fn shaped(&self, doc: &Value, now: i64) -> Value {
        let parts = contract::Parts {
            entries: &self.entries,
            threads: &self.threads,
            sync: &self.sync,
        };
        contract::shape(doc, &parts, now)
    }

    /// The doc in its shape with the log as it is now; keeps `doc` as the last doc.
    pub(crate) fn stamp(&mut self, doc: Value, now: i64) -> Value {
        self.closed = ended(&doc);
        let out = self.shaped(&doc, now);
        self.doc = Some(doc);
        out
    }

    /// The last doc again with the log as it is now; `None` when there is none, or when it ended
    /// and `even_ended` is false.
    pub(crate) fn again(&self, even_ended: bool, now: i64) -> Option<Value> {
        let doc = self.doc.as_ref().filter(|d| even_ended || !ended(d))?;
        Some(self.shaped(doc, now))
    }

    /// Ends the switch `id` in `phase` (with `error`) after the steps `lines`, when its doc is
    /// still the last one; `None` when another doc came since.
    pub(crate) fn finish(
        &mut self,
        id: &str,
        (phase, error): (&str, Option<&str>),
        at: i64,
        lines: &[Line],
    ) -> Option<Value> {
        let mut doc = self.doc.clone().filter(|d| d["id"] == id)?;
        self.add_after(at, lines);
        doc["phase"] = json!(phase);
        if let Some(e) = error {
            doc["error"] = json!(e);
        }
        Some(self.stamp(doc, at))
    }
}

/// The step log of the accounts service.
#[derive(Default)]
pub(crate) struct SwitchLog {
    state: Mutex<LogState>,
}

impl SwitchLog {
    fn held(&self) -> MutexGuard<'_, LogState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl AccountsService {
    /// Writes the `switch` doc with `lines` added to its step log. Blocking.
    ///
    /// # Errors
    /// When the store refuses the write.
    pub(crate) fn put_switch(&self, doc: Value, lines: &[Line]) -> Result<(), String> {
        let now = now_ms();
        let mut log = self.switch_log.held();
        log.begin(&doc);
        log.add(now, lines);
        let doc = log.stamp(doc, now);
        self.docs.put(Coll::AccountsState, SWITCH_DOC, doc)
    }

    /// Notes steps of the process; the doc is written again when a switch is under way. Blocking.
    pub(crate) fn note_switch(&self, lines: &[Line]) {
        let now = now_ms();
        let mut log = self.switch_log.held();
        log.add(now, lines);
        let doc = if lines.is_empty() {
            None
        } else {
            log.again(false, now)
        };
        self.write_again(doc);
    }

    /// Ends the switch `id` in `phase` (with `error`) after the steps `lines`, as long as no
    /// other doc was written since. Blocking.
    pub(crate) fn finish_switch(&self, id: &str, end: (&str, Option<&str>), lines: &[Line]) {
        let doc = self.switch_log.held().finish(id, end, now_ms(), lines);
        self.write_again(doc);
    }

    /// Keeps what the sync of the switch wrote; the next write of the doc shows it.
    pub(crate) fn note_sync(&self, info: Value) {
        self.switch_log.held().set_sync(info);
    }

    fn write_again(&self, doc: Option<Value>) {
        let Some(doc) = doc else { return };
        if let Err(e) = self.docs.put(Coll::AccountsState, SWITCH_DOC, doc) {
            self.log
                .warn("accounts switch log not written", json!({ "error": e }));
        }
    }

    /// Notes the lines a change of the handover cycle makes, and the leads it has. Blocking.
    pub(crate) fn note_cycle(
        &self,
        lines: &CycleLines,
        cycle: Option<&Cycle>,
        name_of: &dyn Fn(&str) -> Option<String>,
    ) {
        let now = now_ms();
        let mut log = self.switch_log.held();
        log.add(now, &lines.process);
        if let Some(cycle) = cycle {
            log.set_threads(threads_of(cycle, name_of));
        }
        log.add_after(now, &lines.resume);
        let even_ended = !lines.resume.is_empty();
        let doc = if lines.process.is_empty() && !even_ended {
            None
        } else {
            log.again(even_ended, now)
        };
        drop(log);
        self.write_again(doc);
    }
}

#[cfg(test)]
mod tests;
