//! The step log of a switch: every step the process takes, from the handover cycle to the end
//! of the switch, as `{ at, text }` lines in the `log` field of the `switch` state doc. Every
//! writer of that doc goes through [`AccountsService::put_switch`], so the log is never lost
//! between writers, and a step noted while a switch is under way rewrites the doc at once.
//!
//! A log belongs to one process. A doc that ends it (done, cancelled, failed or carrying an
//! error) closes the log; the next step or the next live doc starts a new one. Resume lines
//! come after the end and join the finished log instead.

mod lines;

use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_core::Coll;
use argus_core::clock::now_ms;
use serde_json::{Value, json};

pub(crate) use lines::{CycleLines, cycle_lines, sidebar_check_line, step_lines, sync_note};

use super::AccountsService;

const SWITCH_DOC: &str = "switch";
const ENDED: [&str; 4] = ["idle", "done", "cancelled", "failed"];

/// Whether a `switch` doc ends its process.
fn ended(doc: &Value) -> bool {
    let phase = doc["phase"].as_str().unwrap_or("idle");
    ENDED.contains(&phase) || !doc["error"].is_null()
}

/// The log lines and the last doc written.
#[derive(Default)]
pub(crate) struct LogState {
    entries: Vec<Value>,
    closed: bool,
    doc: Option<Value>,
}

impl LogState {
    fn push(&mut self, at: i64, lines: &[String]) {
        self.entries.extend(
            lines
                .iter()
                .map(|text| json!({ "at": at, "text": text })),
        );
    }

    /// Notes steps of the process; a closed log starts over first.
    pub(crate) fn add(&mut self, at: i64, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        if self.closed {
            self.entries.clear();
            self.closed = false;
        }
        self.push(at, lines);
    }

    /// Notes steps that follow the end, such as a resume: they join the last log.
    pub(crate) fn add_after(&mut self, at: i64, lines: &[String]) {
        self.push(at, lines);
    }

    /// Puts the log into `doc` and keeps it as the last doc.
    pub(crate) fn stamp(&mut self, mut doc: Value) -> Value {
        let ends = ended(&doc);
        if self.closed && !ends {
            self.entries.clear();
            self.closed = false;
        }
        doc["log"] = json!(self.entries);
        self.closed = ends;
        self.doc = Some(doc.clone());
        doc
    }

    /// The last doc again with the log as it is now; `None` when there is none, or when it ended
    /// and `even_ended` is false.
    pub(crate) fn again(&mut self, even_ended: bool) -> Option<Value> {
        let doc = self.doc.clone().filter(|d| even_ended || !ended(d))?;
        let closed = self.closed;
        let doc = self.stamp(doc);
        self.closed = closed;
        Some(doc)
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
    pub(crate) fn put_switch(&self, doc: Value, lines: &[String]) -> Result<(), String> {
        let mut log = self.switch_log.held();
        log.add(now_ms(), lines);
        let doc = log.stamp(doc);
        self.docs.put(Coll::AccountsState, SWITCH_DOC, doc)
    }

    /// Notes steps of the process; the doc is written again when a switch is under way. Blocking.
    pub(crate) fn note_switch(&self, lines: &[String]) {
        let mut log = self.switch_log.held();
        log.add(now_ms(), lines);
        let doc = if lines.is_empty() {
            None
        } else {
            log.again(false)
        };
        self.write_again(doc);
    }

    /// Notes resume steps after the end of a switch and writes its doc again. Blocking.
    pub(crate) fn note_after_switch(&self, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        let mut log = self.switch_log.held();
        log.add_after(now_ms(), lines);
        let doc = log.again(true);
        self.write_again(doc);
    }

    fn write_again(&self, doc: Option<Value>) {
        let Some(doc) = doc else { return };
        if let Err(e) = self.docs.put(Coll::AccountsState, SWITCH_DOC, doc) {
            self.log
                .warn("accounts switch log not written", json!({ "error": e }));
        }
    }

    /// Notes the lines a change of the handover cycle makes. Blocking.
    pub(crate) fn note_cycle(&self, lines: CycleLines) {
        self.note_switch(&lines.process);
        self.note_after_switch(&lines.resume);
    }
}

#[cfg(test)]
mod tests;
