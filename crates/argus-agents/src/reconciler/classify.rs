//! Step 4: every live record against its transcript, else its task output. A transcript that
//! grew lately is alive. One that is quiet with its turn not ended is left to the reaper and the
//! stale wake. One whose turn ended is idle: the thread is told once, and the record is closed as
//! finished only after a full clear period with no growth.

use super::pass::{IDLE_KEYS, ORPHAN_KEYS, Pass};
use super::report::Class;
use super::session::Session;
use super::state::{Observed, task_state, transcript_state};
use crate::config::MINUTE_MS;
use crate::iso::format_iso;
use crate::record::{Patch, Record, claude_id_of, filled, is_set, ms_of, text};
use crate::transcript::last_turn;
use serde_json::Value;
use std::path::Path;

const RESULT_MAX: usize = 300;

/// Milliseconds as whole units, halves rounding up as `Math.round` does.
pub(super) fn rounded(ms: i64, unit: i64) -> i64 {
    (ms + unit / 2).div_euclid(unit)
}

impl<'a> Pass<'a> {
    pub(super) fn classify(&mut self) {
        let live: Vec<&'a Record> = self
            .all
            .iter()
            .copied()
            .filter(|a| Self::is_open(a) && !self.closed_here.contains(crate::record::id_of(a)))
            .collect();
        for agent in live {
            self.classify_one(agent);
        }
    }

    fn classify_one(&mut self, agent: &Record) {
        let current = self.cur(agent);
        let session = self.session(agent);
        let id = claude_id_of(&current);
        match self.observe(&current, &session, id.as_deref()) {
            Some((state, via)) if !state.ended => self.running(agent, &state, via),
            Some((state, _)) => self.idle(agent, &current, &state, id.as_deref()),
            None => self.missing(agent, &current, &session),
        }
    }

    fn observe(
        &self,
        current: &Record,
        session: &Session,
        id: Option<&str>,
    ) -> Option<(Observed, &'static str)> {
        let env = self.env;
        let own = filled(current, "transcriptFile")
            .and_then(|f| transcript_state(env, Path::new(f), self.at));
        if let Some(state) = own {
            return Some((state, "transcript"));
        }
        let id = id?;
        let found = session
            .find_file(id)
            .and_then(|f| transcript_state(env, &f, self.at));
        if let Some(state) = found {
            return Some((state, "transcript"));
        }
        let task = session
            .task_file(id)
            .and_then(|f| task_state(env, &f, self.at));
        task.map(|state| (state, "task"))
    }

    fn running(&mut self, agent: &Record, state: &Observed, via: &str) {
        let (class, detail) = if state.alive {
            self.counts.alive += 1;
            (
                Class::Alive,
                format!("{via} grew {}s ago", rounded(state.quiet_ms, 1000)),
            )
        } else {
            self.counts.quiet += 1;
            let minutes = rounded(state.quiet_ms, MINUTE_MS);
            (
                Class::Quiet,
                format!("{via} quiet for {minutes} min (the reaper and the stale wake cover it)"),
            )
        };
        self.clear_flags(agent, &[&ORPHAN_KEYS[..], &IDLE_KEYS[..]].concat());
        self.finding(agent, class, detail);
    }

    fn idle(&mut self, agent: &Record, current: &Record, state: &Observed, id: Option<&str>) {
        self.counts.idle += 1;
        self.clear_flags(agent, &ORPHAN_KEYS);
        let episode = format_iso(state.ended_at);
        let same_episode = text(current, "idleSince") == Some(episode.as_str());
        let mut woke_at = if same_episode && is_set(current, "idleWokeAt") {
            ms_of(current, "idleWokeAt")
        } else {
            None
        };
        if !same_episode {
            let started = Patch::new()
                .with("idleSince", episode)
                .without("idleWokeAt");
            self.patch(agent, started);
        }
        let idle_ms = self.at - state.ended_at;
        let opts = &self.env.opts;
        if woke_at.is_none() && self.can_wake && idle_ms >= opts.idle_wake_ms {
            let sentence = format!(
                "{}, id {}) ended its turn: resume it by id (SendMessage to the agent id) or close the record (coord agent done {}).",
                Self::who(agent),
                id.unwrap_or("unknown"),
                crate::record::id_of(agent),
            );
            if self.wake(agent, "agent-idle", sentence) {
                woke_at = Some(self.at);
                self.patch(agent, Patch::new().with("idleWokeAt", format_iso(self.at)));
            }
        }
        let fallback = (!self.can_wake).then(|| state.ended_at + opts.idle_wake_ms);
        let due = woke_at.or(fallback).is_some_and(|base| {
            self.at - base.max(self.env.boot_at) >= self.env.opts.orphan_clear_ms
        });
        if due {
            self.close_finished(agent, current, state, idle_ms);
        } else {
            let told = if woke_at.is_some() {
                ", the thread was told"
            } else {
                ""
            };
            let minutes = rounded(idle_ms, MINUTE_MS);
            self.finding(
                agent,
                Class::Idle,
                format!("turn ended {minutes} min ago{told}"),
            );
        }
    }

    fn close_finished(&mut self, agent: &Record, current: &Record, state: &Observed, idle_ms: i64) {
        let last = last_turn(self.env.fs, &state.file, state.size);
        let stopped = text(agent, "status") == Some("stop-requested") || state.stopped;
        let mut closed = Patch::new()
            .with("status", if stopped { "stopped" } else { "done" })
            .with("finishedAt", format_iso(self.at))
            .with("note", "finished, closed automatically")
            .with("reaped", true)
            .with("closedBy", "reconciler");
        if let Some(turn) = last.filter(|t| !t.text.is_empty())
            && !is_set(current, "result")
        {
            let result: String = turn.text.chars().take(RESULT_MAX).collect();
            closed = closed.with("result", Value::from(result));
        }
        self.patch(agent, closed.without_all(&IDLE_KEYS));
        self.counts.closed += 1;
        let minutes = rounded(idle_ms, MINUTE_MS);
        let detail =
            format!("turn ended {minutes} min ago and nothing grew since: closed as finished");
        self.finding(agent, Class::Closed, detail);
    }
}
