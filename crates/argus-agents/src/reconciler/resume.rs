//! Step 5: a record that was closed (`done` or `stopped`, not a duplicate or a lost one) within a
//! day whose transcript has grown since is reopened as `working`. A `stopped` record needs the
//! longer proof, because pausing a thread stops its records, not its agents.

use super::pass::Pass;
use super::report::Class;
use super::state::transcript_state;
use crate::record::{Patch, Record, claude_id_of, filled, id_of, ms_of, text};
use std::path::{Path, PathBuf};

impl<'a> Pass<'a> {
    pub(super) fn resume(&mut self) {
        let closed: Vec<&'a Record> = self
            .all
            .iter()
            .copied()
            .filter(|a| self.recently_closed(a) && !self.closed_here.contains(id_of(a)))
            .collect();
        for agent in closed {
            self.reopen_if_growing(agent);
        }
    }

    fn transcript_of(&mut self, current: &Record) -> Option<PathBuf> {
        let own = filled(current, "transcriptFile").filter(|f| self.exists(Some(f)));
        if let Some(file) = own {
            return Some(PathBuf::from(file));
        }
        let id = claude_id_of(current)?;
        self.session_of(text(current, "thread")).find_file(&id)
    }

    fn reopen_if_growing(&mut self, agent: &Record) {
        let current = self.cur(agent);
        let Some(file) = self.transcript_of(&current) else {
            return;
        };
        let Some(state) = transcript_state(self.env, Path::new(&file), self.at) else {
            return;
        };
        let gaps = if text(&current, "status") == Some("stopped") {
            3
        } else {
            1
        };
        let finished = ms_of(&current, "finishedAt").unwrap_or(0);
        if !state.alive || state.mtime_ms <= finished + gaps * self.env.opts.reopen_gap_ms {
            return;
        }
        let reopened = Patch::new()
            .with("status", "working")
            .with("finishedAt", serde_json::Value::Null)
            .with("note", "resumed, reopened automatically")
            .without_all(&["reaped", "closedBy", "lost"]);
        self.patch(agent, reopened);
        self.counts.reopened += 1;
        let detail = "its transcript grew after the record was closed";
        self.finding(agent, Class::Reopened, detail.to_owned());
    }
}
