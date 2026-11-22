//! What the Overlord is doing now, for the owner's header: the session status and whether a tool
//! call is in flight, in one word.

use serde::Serialize;

use super::state::Status;

/// The four words of the header mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Activity {
    /// No child runs.
    Off,
    /// A child runs and no turn does.
    Idle,
    /// A turn runs and no tool call is in flight: the model writes.
    Thinking,
    /// A turn runs with a tool call in flight, or waits on an approval.
    Working,
}

/// The activity of a session in `status`, with `tool_in_flight` from the supervisor's link.
#[must_use]
pub fn activity(status: Status, tool_in_flight: bool) -> Activity {
    match status {
        Status::Stopped | Status::Crashed => Activity::Off,
        Status::WaitingApproval => Activity::Working,
        Status::Turn if tool_in_flight => Activity::Working,
        Status::Turn => Activity::Thinking,
        Status::Starting | Status::Idle | Status::PausedBudget => Activity::Idle,
    }
}

// test-tag: critical
#[cfg(test)]
mod specs {
    use super::*;

    #[test]
    fn no_child_is_off() {
        assert_eq!(activity(Status::Stopped, false), Activity::Off);
        assert_eq!(activity(Status::Crashed, true), Activity::Off);
    }

    #[test]
    fn a_turn_thinks_until_a_tool_runs() {
        assert_eq!(activity(Status::Turn, false), Activity::Thinking);
        assert_eq!(activity(Status::Turn, true), Activity::Working);
    }

    #[test]
    fn an_approval_wait_is_working_and_the_rest_is_idle() {
        assert_eq!(activity(Status::WaitingApproval, false), Activity::Working);
        assert_eq!(activity(Status::Idle, false), Activity::Idle);
        assert_eq!(activity(Status::Starting, false), Activity::Idle);
        assert_eq!(activity(Status::PausedBudget, false), Activity::Idle);
    }

    #[test]
    fn the_word_is_lowercase_json() {
        let word = serde_json::to_value(Activity::Thinking).expect("json");
        assert_eq!(word, "thinking");
    }
}
