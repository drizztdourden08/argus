//! Step 4, the record Claude has nothing for: no transcript and no task output. In a session
//! Claude has a folder for, a record older than `orphanWakeMs` and untouched for as long is an
//! orphan: its thread is told once, and `orphanClearMs` later, still unchanged, it is closed
//! `stopped` with `lost: true`. Any update of the record starts the clock over.

use super::pass::{IDLE_KEYS, ORPHAN_KEYS, Pass};
use super::report::Class;
use crate::evidence::session::Session;
use crate::iso::format_iso;
use crate::record::{Patch, Record, id_of, is_set, ms_of, text};

impl Pass<'_> {
    pub(super) fn missing(&mut self, agent: &Record, current: &Record, session: &Session) {
        self.clear_flags(agent, &IDLE_KEYS);
        if !session.known {
            self.counts.unknown += 1;
            let detail = if session.has_session_id() {
                "Claude has no folder for the thread session: nothing to compare with"
            } else {
                "the thread has no session id: nothing to compare with"
            };
            self.finding(agent, Class::Unknown, detail.to_owned());
            return;
        }
        let touched_since =
            is_set(current, "orphanSince") && current.get("rev") != current.get("orphanRev");
        if touched_since {
            self.clear_flags(agent, &ORPHAN_KEYS);
        }
        let flagged = is_set(current, "orphanSince") && !touched_since;
        if !flagged && !self.old_enough(current) {
            self.counts.pending += 1;
            let detail = "no Claude task found yet, but the record is recent";
            self.finding(agent, Class::Pending, detail.to_owned());
            return;
        }
        self.counts.orphan += 1;
        self.orphan(agent, current, flagged);
    }

    fn old_enough(&self, current: &Record) -> bool {
        let wake_ms = self.env.opts.orphan_wake_ms;
        let began = ms_of(current, "startedAt");
        let touched = ms_of(current, "updatedAt").filter(|m| *m != 0).or(began);
        let aged = |moment: Option<i64>| moment.is_some_and(|m| self.at - m >= wake_ms);
        aged(began) && aged(touched)
    }

    fn orphan(&mut self, agent: &Record, current: &Record, flagged: bool) {
        let mut woke_at =
            ms_of(current, "orphanWokeAt").filter(|_| flagged && is_set(current, "orphanWokeAt"));
        if !flagged {
            self.patch(agent, Patch::new().with("orphanSince", format_iso(self.at)));
        }
        if woke_at.is_none() && self.can_wake {
            let sentence = format!(
                "{}) is on the board but Claude has no sub-agent or background task for it: finished or dead? close it (coord agent done {}) or tell me it is running.",
                Self::who(agent),
                id_of(agent),
            );
            if self.wake(agent, "agent-orphan", sentence) {
                woke_at = Some(self.at);
                self.patch(
                    agent,
                    Patch::new().with("orphanWokeAt", format_iso(self.at)),
                );
            }
        }
        let since = if flagged {
            ms_of(current, "orphanSince")
        } else {
            Some(self.at)
        };
        let fallback = if self.can_wake { None } else { since };
        let due = woke_at.or(fallback).is_some_and(|base| {
            self.at - base.max(self.env.boot_at) >= self.env.opts.orphan_clear_ms
        });
        if due {
            self.close_lost(agent);
            return;
        }
        let since_text = text(current, "orphanSince")
            .filter(|_| flagged)
            .map_or_else(|| format_iso(self.at), str::to_owned);
        let told = if woke_at.is_some() {
            ", the thread was told"
        } else {
            ""
        };
        let detail = format!("no Claude task found since {since_text}{told}");
        self.finding(agent, Class::Orphan, detail);
    }

    fn close_lost(&mut self, agent: &Record) {
        let closed = Patch::new()
            .with("status", "stopped")
            .with("finishedAt", format_iso(self.at))
            .with("note", "no Claude task found, closed automatically")
            .with("lost", true)
            .with("reaped", true)
            .with("closedBy", "reconciler")
            .without_all(&ORPHAN_KEYS);
        self.patch(agent, closed);
        self.counts.closed += 1;
        let name = text(agent, "name").unwrap_or("");
        self.lost
            .push(format!("{} {name}", id_of(agent)).trim().to_owned());
        let detail = "no Claude task found and unchanged: closed as lost";
        self.finding(agent, Class::Closed, detail.to_owned());
    }
}
