//! The state of one probe run: the fact lines, the raw log and the child to kill.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::redact::Redactor;
use crate::lifecycle::spawn::kill_tree;

/// What a probe run has seen and written so far.
pub struct Probe {
    redactor: Redactor,
    facts: Vec<String>,
    raw: File,
    /// Unix seconds of the start, shared by the file names of the run.
    pub stamp: u64,
    /// The step in progress, named in a timeout report.
    pub step: &'static str,
    /// The pid of the child while it runs.
    pub pid: Option<u32>,
}

impl Probe {
    /// Creates the report folder and the raw log `probe-<unix seconds>.jsonl`.
    pub fn open(dir: &Path, key: &str) -> Result<Self, String> {
        fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let path = dir.join(format!("probe-{stamp}.jsonl"));
        let raw =
            File::create(&path).map_err(|e| format!("cannot create {}: {e}", path.display()))?;
        Ok(Self {
            redactor: Redactor::new(key),
            facts: Vec::new(),
            raw,
            stamp,
            step: "start",
            pid: None,
        })
    }

    /// A copy of the redactor, for a task that writes its own file.
    pub fn redactor(&self) -> Redactor {
        self.redactor.clone()
    }

    /// Prints and keeps one line `fact <name>: <value>`.
    pub fn fact(&mut self, name: &str, value: &str) {
        let line = self.redactor.apply(&format!("fact {name}: {value}"));
        println!("{line}");
        self.facts.push(line);
    }

    /// Appends one raw child line, redacted, to the jsonl log.
    pub fn log_raw(&mut self, line: &str) {
        let clean = self.redactor.apply(line);
        let _ignored = writeln!(self.raw, "{clean}");
    }

    /// Writes the fact lines to `probe-facts.txt`.
    pub fn save_facts(&self, dir: &Path) -> Result<(), String> {
        let path = dir.join("probe-facts.txt");
        let text = format!("{}\n", self.facts.join("\n"));
        fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
    }

    /// Tree kills the child by pid when one is still known.
    pub fn kill_child(&mut self) {
        if let Some(pid) = self.pid.take() {
            kill_tree(pid);
        }
    }
}
