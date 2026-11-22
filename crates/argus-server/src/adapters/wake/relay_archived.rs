//! A wake whose session the owner archived: the relay neither posts a note nor resumes it, because the
//! desktop refuses an archived session. Once a day per thread it logs one activity line and tells the owner.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use argus_accounts::session_map::{is_archived, local_ids};
use argus_store::{ActivityLevel, NewActivity};
use argus_wake::relay::Ward;

use super::relay::RelayHost;

/// The archived notice goes out at most this often per thread.
const NOTICE_EVERY: Duration = Duration::from_secs(86_400);

/// Remembers when the owner was last told about each archived thread, keyed by project and thread.
#[derive(Debug, Default)]
pub struct ArchivedGate(Mutex<HashMap<(String, String), Instant>>);

impl ArchivedGate {
    /// Is a notice for `target` due at `now`? When it is, the gate counts it as sent.
    pub fn due(&self, target: &Ward, now: Instant) -> bool {
        let mut seen = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = (target.key.project.clone(), target.key.thread.clone());
        if seen
            .get(&key)
            .is_some_and(|at| now.saturating_duration_since(*at) < NOTICE_EVERY)
        {
            return false;
        }
        seen.insert(key, now);
        true
    }
}

impl RelayHost {
    /// True when the session of `target` is archived. Then, once a day, it logs the skip and tells the owner.
    pub(super) fn skip_archived(&self, app: &Path, target: &Ward) -> bool {
        let Some(session) = target.session_id.as_ref() else {
            return false;
        };
        let wanted = [session.clone()];
        let archived = local_ids(app, &wanted)
            .get(session)
            .is_some_and(|local| is_archived(app, local));
        if archived && self.archived.due(target, Instant::now()) {
            self.log_skip(target);
            self.tell_archived(target);
        }
        archived
    }

    fn log_skip(&self, target: &Ward) {
        let Some(activity) = &self.activity else {
            return;
        };
        let text = format!(
            "wake of {} {} skipped: session archived",
            target.key.project, target.key.thread
        );
        let line = NewActivity::new("headless-wake", ActivityLevel::Info, text)
            .project(Some(target.key.project.clone()))
            .thread(Some(target.key.thread.clone()));
        activity.add(line);
    }

    fn tell_archived(&self, target: &Ward) {
        let text = format!(
            "{} {} holds a wake but its session is archived, so Argus cannot wake it. Unarchive it or end the thread.",
            target.key.project, target.key.thread
        );
        self.ping(target, format!("{} is archived", target.key.thread), text);
    }
}
