//! The lead's approval ask: a worktree nothing is attached to still holds work, so its owning lead is asked ONCE to
//! approve the removal (full backup first) or to ask the Overlord for an exception. One inbox note with the commands
//! and one short wake per lead.

use super::memory::{HygieneState, Mark};
use super::text::{APPROVE_REASON, ask_note, ask_wake};
use crate::ports::WakeEvent;
use crate::service::Service;
use std::collections::BTreeMap;

/// The worktrees to ask about, by lead (none when the board has no lead to ask).
type ByLead = BTreeMap<Option<String>, Vec<(String, Mark)>>;

impl Service {
    /// Ask each lead once about its worktrees that hold work, the debounce after nothing was attached. A worktree with
    /// no lead to ask counts as asked: the Overlord hears of it in turn. Says how many leads were asked.
    pub(super) fn ask_leads(&self, project: &str) -> usize {
        let now = self.now();
        let debounce = self.opts.detach_debounce_ms;
        let due: ByLead = self.with_hygiene(project, |memory| {
            let mut out = ByLead::new();
            for (id, mark) in &memory.marks {
                let ready = mark.state == HygieneState::AwaitingLead
                    && mark.asked.is_none()
                    && now - mark.since >= debounce;
                if ready {
                    out.entry(mark.thread.clone())
                        .or_default()
                        .push((id.clone(), mark.clone()));
                }
            }
            out
        });
        let mut asked = 0;
        for (lead, list) in &due {
            if let Some(lead) = lead {
                asked += usize::from(self.ask_lead(project, lead, list));
            }
        }
        self.with_hygiene(project, |memory| {
            for id in due.values().flatten().map(|(id, _)| id) {
                if let Some(mark) = memory.marks.get_mut(id) {
                    mark.asked = Some(now);
                }
            }
        });
        asked
    }

    /// The note with every command, then the wake. True when either reached the lead.
    fn ask_lead(&self, project: &str, lead: &str, list: &[(String, Mark)]) -> bool {
        let refs: Vec<(&String, &Mark)> = list.iter().map(|(id, m)| (id, m)).collect();
        let note = ask_note(project, &refs, self.opts.report_after_ms);
        let posted = matches!(self.deps.store.post_inbox(project, lead, &note), Ok(true));
        let event = WakeEvent {
            scope: project.to_string(),
            thread: lead.to_string(),
            reason: APPROVE_REASON,
            subject: lead.to_string(),
            text: ask_wake(&refs),
        };
        let woke = self.deps.notifier.wake(std::slice::from_ref(&event)).is_ok();
        if woke {
            self.log_wakes(project, &[event]);
        }
        posted || woke
    }
}
