//! Telling the thread how its request ended: one note in its inbox, one wake, one activity line.

use super::record::RequestStatus;
use super::text::summary_text;
use crate::ports::{ActivityLine, WakeEvent};
use crate::service::Service;

/// The wake reason of a note that lands in a thread's inbox.
const REASON: &str = "thread-message";
const WAKE_MAX: usize = 300;

impl Service {
    /// An activity line of kind `worktree-request` for this project.
    pub(crate) fn note_request(
        &self,
        project: &str,
        level: &'static str,
        text: &str,
        thread: Option<&str>,
    ) {
        self.deps.notifier.activity(ActivityLine {
            kind: "worktree-request",
            level,
            text: format!("{project}: {text}"),
            project: project.to_string(),
            thread: thread.map(ToString::to_string),
        });
    }

    /// Tell the thread, once, on the project's board or on the board the request names (the Overlord's portfolio).
    /// The wake goes only when the note reached an inbox.
    pub(crate) fn deliver_request(&self, project: &str, id: &str) {
        let Some(rec) = self.request_of(project, id).filter(|r| !r.notified) else {
            return;
        };
        let text = summary_text(&rec);
        let board = rec.inbox.as_deref().unwrap_or(project);
        let posted = matches!(self.deps.store.post_inbox(board, &rec.by, &text), Ok(true));
        if posted {
            let wake = WakeEvent {
                scope: board.to_string(),
                thread: rec.by.clone(),
                reason: REASON,
                subject: rec.by.clone(),
                text: format!("Argus: {text}").chars().take(WAKE_MAX).collect(),
            };
            let _ = self.deps.notifier.wake(&[wake]);
        }
        self.update_request(project, id, |r| r.notified = true);
        self.save_requests(project);
        let level = if rec.status == RequestStatus::Refused {
            "warn"
        } else {
            "info"
        };
        self.note_request(
            project,
            level,
            &format!("{text} (asked by {})", rec.by),
            Some(&rec.by),
        );
    }
}
