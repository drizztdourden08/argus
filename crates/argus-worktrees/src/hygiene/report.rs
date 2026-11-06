//! The report to the Overlord: a lead that did not answer the approval ask within `report_after_ms`, while the
//! worktree stayed unattached, hands the worktree to the Overlord (the lead of the `portfolio` project) for immediate
//! deletion. ONE note per project and step with the worktree, what is left, the backup path and the one delete
//! command, and one wake. Each worktree is reported once.

use super::memory::{HygieneState, Mark};
use super::text::{REPORT_REASON, Reported, report_note, report_wake};
use crate::ports::WakeEvent;
use crate::service::Service;
use argus_core::ids::OVERLORD_PROJECT;

impl Service {
    /// The Overlord's thread: the live lead of the `portfolio` board.
    pub(crate) fn overlord(&self) -> Option<String> {
        self.deps.store.board(OVERLORD_PROJECT)?.coordinator()
    }

    /// Report the asks left unanswered. With no Overlord to tell, the next step tries again. Says how many reports
    /// went (0 or 1).
    pub(super) fn report_overlord(&self, project: &str) -> usize {
        let now = self.now();
        let after = self.opts.report_after_ms;
        let due: Vec<(String, Mark)> = self.with_hygiene(project, |memory| {
            memory
                .marks
                .iter()
                .filter(|(_, m)| m.state == HygieneState::AwaitingLead && m.reported.is_none())
                .filter(|(_, m)| m.asked.is_some_and(|t| now - t >= after))
                .map(|(id, m)| (id.clone(), m.clone()))
                .collect()
        });
        if due.is_empty() {
            return 0;
        }
        let Some(overlord) = self.overlord() else {
            return 0;
        };
        let facts = self.facts_of(project);
        let list: Vec<Reported<'_>> = due
            .iter()
            .filter_map(|(id, mark)| {
                let fact = facts.iter().find(|f| f.id == *id)?;
                Some(Reported {
                    project,
                    id,
                    mark,
                    backup: self.backup_prefix(project, fact),
                })
            })
            .collect();
        if list.is_empty() || !self.tell_overlord(project, &overlord, &list) {
            return 0;
        }
        self.with_hygiene(project, |memory| {
            for r in &list {
                if let Some(mark) = memory.marks.get_mut(r.id) {
                    mark.reported = Some(now);
                    mark.state = HygieneState::Reported;
                }
            }
        });
        1
    }

    /// The note on the Overlord's board, then the wake. True when either reached him.
    fn tell_overlord(&self, project: &str, overlord: &str, list: &[Reported<'_>]) -> bool {
        let note = report_note(list, self.opts.report_after_ms);
        let posted = matches!(
            self.deps.store.post_inbox(OVERLORD_PROJECT, overlord, &note),
            Ok(true)
        );
        let event = WakeEvent {
            scope: OVERLORD_PROJECT.to_string(),
            thread: overlord.to_string(),
            reason: REPORT_REASON,
            subject: overlord.to_string(),
            text: report_wake(list),
        };
        let woke = self.deps.notifier.wake(std::slice::from_ref(&event)).is_ok();
        if woke {
            self.log_wakes(project, &[event]);
        }
        posted || woke
    }
}
