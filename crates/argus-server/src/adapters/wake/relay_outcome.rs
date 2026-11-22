//! What the relay does with the end of a headless resume: one activity line, and one owner ping when the
//! CLI login has expired (at most once an hour).

use std::sync::Mutex;
use std::time::{Duration, Instant};

use argus_notify::{By, Level, PingRequest};
use argus_store::{ActivityLevel, NewActivity};
use argus_wake::relay::Ward;

use super::outcome::Outcome;
use super::relay::RelayHost;

/// The sign-in ping goes out at most this often.
const SIGN_IN_EVERY: Duration = Duration::from_secs(3600);

/// Remembers when the owner was last told that the CLI login expired.
#[derive(Debug, Default)]
pub struct SignInGate(Mutex<Option<Instant>>);

impl SignInGate {
    /// Is a ping due at `now`? When it is, the gate counts it as sent.
    pub fn due(&self, now: Instant) -> bool {
        let mut last = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if last.is_some_and(|at| now.saturating_duration_since(at) < SIGN_IN_EVERY) {
            return false;
        }
        *last = Some(now);
        true
    }
}

impl RelayHost {
    /// Records the outcome of a headless resume of `target`, and pings the owner on an expired login.
    pub(super) fn on_outcome(&self, target: &Ward, outcome: &Outcome) {
        self.log_outcome(target, outcome);
        if *outcome == Outcome::SignIn && self.sign_in.due(Instant::now()) {
            self.tell_sign_in();
        }
    }

    fn log_outcome(&self, target: &Ward, outcome: &Outcome) {
        let Some(activity) = &self.activity else {
            return;
        };
        let (level, what) = match outcome {
            Outcome::Ok => (ActivityLevel::Info, "ok".to_owned()),
            Outcome::SignIn => (ActivityLevel::Warn, "sign-in expired".to_owned()),
            Outcome::Failed(reason) => (ActivityLevel::Warn, format!("failed ({reason})")),
            Outcome::TimedOut => (ActivityLevel::Warn, "timed out".to_owned()),
        };
        let text = format!(
            "headless wake of {} {}: {what}",
            target.key.project, target.key.thread
        );
        let line = NewActivity::new("headless-wake", level, text)
            .project(Some(target.key.project.clone()))
            .thread(Some(target.key.thread.clone()));
        activity.add(line);
    }

    fn tell_sign_in(&self) {
        let Some(notify) = &self.notify else {
            return;
        };
        drop(notify.send(PingRequest {
            title: Some("Headless wakes cannot sign in".to_owned()),
            text: "The Claude CLI login that Argus uses for headless wakes has expired, so sleeping threads cannot be woken. Run: claude auth login".to_owned(),
            level: Level::Ping,
            project: None,
            thread: None,
            by: By::Auto,
            url: None,
        }));
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sign_in_ping_is_due_once_an_hour() {
        let gate = SignInGate::default();
        let t0 = Instant::now();
        assert!(gate.due(t0));
        assert!(!gate.due(t0 + Duration::from_secs(3599)));
        assert!(gate.due(t0 + SIGN_IN_EVERY));
    }
}
