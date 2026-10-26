//! Step 5: a record that was closed (`done` or `stopped`, not a duplicate or a lost one) within a
//! day whose agent is working again is reopened as `working`. Two proofs: the transcript has grown
//! since the close (a `stopped` record needs the longer proof, because pausing a thread stops its
//! records, not its agents), or the record was closed by the idle reaper while the agent was in a
//! long tool call, and its Claude task output or worktree shows the work now.

use super::classify::rounded;
use super::pass::Pass;
use super::report::Class;
use crate::evidence::session::Session;
use crate::evidence::sign::{Look, last_sign};
use crate::evidence::state::{Observed, transcript_state};
use crate::record::{Patch, Record, claude_id_of, filled, id_of, is_set, ms_of, text};
use std::path::{Path, PathBuf};

/// Why a closed record comes back.
struct Reopen {
    note: String,
    detail: String,
}

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

    fn transcript_of(
        &self,
        current: &Record,
        session: &Session,
        id: Option<&str>,
    ) -> Option<PathBuf> {
        let own = filled(current, "transcriptFile").filter(|f| self.exists(Some(f)));
        if let Some(file) = own {
            return Some(PathBuf::from(file));
        }
        session.find_file(id?)
    }

    fn reopen_if_growing(&mut self, agent: &Record) {
        let current = self.cur(agent);
        let id = claude_id_of(&current);
        let session = self.session(&current);
        let file = self.transcript_of(&current, &session, id.as_deref());
        let state = file.and_then(|f| transcript_state(self.env, Path::new(&f), self.at));
        let gaps = if text(&current, "status") == Some("stopped") {
            3
        } else {
            1
        };
        let finished = ms_of(&current, "finishedAt").unwrap_or(0);
        let after = finished + gaps * self.env.opts.reopen_gap_ms;
        let growing = state
            .as_ref()
            .is_some_and(|s| s.alive && s.mtime_ms > after);
        let why = if growing {
            Some(Reopen {
                note: "resumed, reopened automatically".to_owned(),
                detail: "its transcript grew after the record was closed".to_owned(),
            })
        } else {
            self.still_running(&current, &session, state.as_ref(), after)
        };
        let Some(why) = why else {
            return;
        };
        let reopened = Patch::new()
            .with("status", "working")
            .with("finishedAt", serde_json::Value::Null)
            .with("note", why.note)
            .without_all(&["reaped", "closedBy", "lost"]);
        self.patch(agent, reopened);
        self.counts.reopened += 1;
        self.finding(agent, Class::Reopened, why.detail);
    }

    /// A record the idle reaper (or an earlier reconcile) closed for silence while the agent was in a long tool call: the
    /// transcript stays quiet then, but the task output or the worktree shows the work. Seen alive again, it is reopened,
    /// and says why.
    fn still_running(
        &self,
        current: &Record,
        session: &Session,
        state: Option<&Observed>,
        after: i64,
    ) -> Option<Reopen> {
        let reaped = is_set(current, "reaped") && text(current, "status") == Some("done");
        if !reaped || state.is_some_and(|s| s.ended) {
            return None;
        }
        let stall = self.env.opts.stall_ms;
        let look = Look {
            need_beyond: Some(stall),
            claude_id: None,
        };
        let sign = last_sign(self.env, session, self.board, current, self.at, look);
        let latest = sign.signs.iter().rev().max_by_key(|s| s.mtime_ms)?;
        if sign.ended || latest.mtime_ms <= after || self.at - latest.mtime_ms > stall {
            return None;
        }
        let ago = rounded(self.at - latest.mtime_ms, 1000);
        let by = filled(current, "closedBy").map_or_else(String::new, |b| format!(" by the {b}"));
        Some(Reopen {
            note: format!(
                "reopened automatically: its Claude task is still running ({} active {ago}s ago) although the record was closed as idle",
                latest.kind
            ),
            detail: format!(
                "{} active {ago}s ago, after the record was closed{by}",
                latest.kind
            ),
        })
    }
}
