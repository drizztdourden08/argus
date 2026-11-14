//! The state machine of `state.json`: pure transitions on events, and the restart backoff.
//! An event that does not fit the current status leaves the state as it is.

use std::time::Duration;

use serde::{Deserialize, Serialize};

const BACKOFF_START_SECS: u64 = 1;
const BACKOFF_CAP_SECS: u64 = 30;

/// Where the session is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    #[default]
    Stopped,
    Starting,
    Idle,
    Turn,
    PausedBudget,
    WaitingApproval,
    Crashed,
}

impl Status {
    fn is_live(self) -> bool {
        !matches!(self, Status::Stopped | Status::Crashed)
    }
}

/// The content of `state.json`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub session_id: Option<String>,
    pub pid: Option<u32>,
    pub started_at: Option<String>,
    pub status: Status,
    pub process_seq: u64,
    pub last_cost_usd: f64,
    pub resume_from: Option<String>,
}

/// What happened to the session.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The child process exists.
    Spawned {
        pid: u32,
        started_at: String,
        resume_from: Option<String>,
    },
    /// The init event passed the billing guard.
    InitOk {
        session_id: String,
    },
    /// The init event failed the guard: the child is stopped and not restarted.
    InitGuardFailed,
    /// A message was written to stdin.
    TurnStarted,
    /// A `result` event arrived; `cost_usd` is its cumulative cost.
    ResultReceived {
        cost_usd: f64,
    },
    BudgetPaused,
    BudgetResumed,
    ApprovalRequested,
    ApprovalResolved,
    /// The process is gone; `requested` is true when Argus asked it to stop.
    Exited {
        requested: bool,
    },
    /// A restart is allowed after a crash.
    RestartAllowed,
    /// No restart is left: the session stays crashed.
    RestartExhausted,
}

/// What to do after a crash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restart {
    After(Duration),
    Exhausted,
}

/// The next state. Total: an event that does not fit returns the state unchanged.
#[must_use]
pub fn apply(state: &State, event: &Event) -> State {
    let mut next = state.clone();
    match (state.status, event) {
        (
            Status::Stopped | Status::Crashed | Status::Starting,
            Event::Spawned {
                pid,
                started_at,
                resume_from,
            },
        ) => {
            next.status = Status::Starting;
            next.pid = Some(*pid);
            next.started_at = Some(started_at.clone());
            next.resume_from.clone_from(resume_from);
            next.session_id = None;
            next.process_seq += 1;
        }
        (Status::Starting, Event::InitOk { session_id }) => {
            next.status = Status::Idle;
            next.session_id = Some(session_id.clone());
        }
        (Status::Starting, Event::InitGuardFailed) => stop(&mut next, Status::Stopped),
        (Status::Idle, Event::TurnStarted) | (Status::WaitingApproval, Event::ApprovalResolved) => {
            next.status = Status::Turn;
        }
        (Status::Turn | Status::WaitingApproval, Event::ResultReceived { cost_usd }) => {
            next.status = Status::Idle;
            next.last_cost_usd = *cost_usd;
        }
        (Status::Idle | Status::Turn, Event::BudgetPaused) => next.status = Status::PausedBudget,
        (Status::PausedBudget, Event::BudgetResumed) => next.status = Status::Idle,
        (Status::Turn, Event::ApprovalRequested) => next.status = Status::WaitingApproval,
        (status, Event::Exited { requested }) if status.is_live() => {
            let after = if *requested {
                Status::Stopped
            } else {
                Status::Crashed
            };
            stop(&mut next, after);
        }
        (Status::Crashed, Event::RestartAllowed) => next.status = Status::Starting,
        _ => {}
    }
    next
}

fn stop(state: &mut State, status: Status) {
    state.status = status;
    state.pid = None;
}

/// The wait before restart number `attempt` (1 for the first): 1 s, doubling, capped at 30 s.
#[must_use]
pub fn backoff(attempt: u32) -> Duration {
    let doublings = attempt.saturating_sub(1).min(16);
    let secs = BACKOFF_START_SECS
        .saturating_mul(1 << doublings)
        .min(BACKOFF_CAP_SECS);
    Duration::from_secs(secs)
}

/// After `crashes` crashes in a row: wait and restart, or give up past `restart_max`.
#[must_use]
pub fn restart_decision(crashes: u32, restart_max: u32) -> Restart {
    if crashes > restart_max {
        Restart::Exhausted
    } else {
        Restart::After(backoff(crashes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawned() -> Event {
        Event::Spawned {
            pid: 77,
            started_at: "2026-11-14T10:00:00Z".to_owned(),
            resume_from: Some("old".to_owned()),
        }
    }

    fn idle() -> State {
        let started = apply(&State::default(), &spawned());
        apply(
            &started,
            &Event::InitOk {
                session_id: "s1".to_owned(),
            },
        )
    }

    #[test]
    fn a_start_runs_from_spawn_to_idle() {
        let started = apply(&State::default(), &spawned());
        assert_eq!(started.status, Status::Starting);
        assert_eq!(started.pid, Some(77));
        assert_eq!(started.process_seq, 1);
        let ready = idle();
        assert_eq!(ready.status, Status::Idle);
        assert_eq!(ready.session_id.as_deref(), Some("s1"));
        assert_eq!(ready.resume_from.as_deref(), Some("old"));
    }

    #[test]
    fn a_turn_ends_with_the_result_and_its_cost() {
        let turn = apply(&idle(), &Event::TurnStarted);
        assert_eq!(turn.status, Status::Turn);
        let done = apply(&turn, &Event::ResultReceived { cost_usd: 0.25 });
        assert_eq!(done.status, Status::Idle);
        assert!((done.last_cost_usd - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn an_approval_pauses_the_turn_and_returns_to_it() {
        let turn = apply(&idle(), &Event::TurnStarted);
        let waiting = apply(&turn, &Event::ApprovalRequested);
        assert_eq!(waiting.status, Status::WaitingApproval);
        assert_eq!(
            apply(&waiting, &Event::ApprovalResolved).status,
            Status::Turn
        );
    }

    #[test]
    fn the_budget_pauses_and_resumes() {
        let paused = apply(&idle(), &Event::BudgetPaused);
        assert_eq!(paused.status, Status::PausedBudget);
        assert_eq!(apply(&paused, &Event::BudgetResumed).status, Status::Idle);
    }

    #[test]
    fn a_failed_guard_stops_without_a_restart() {
        let started = apply(&State::default(), &spawned());
        let stopped = apply(&started, &Event::InitGuardFailed);
        assert_eq!(stopped.status, Status::Stopped);
        assert_eq!(stopped.pid, None);
    }

    #[test]
    fn an_asked_exit_is_stopped_and_a_surprise_exit_is_crashed() {
        let asked = apply(&idle(), &Event::Exited { requested: true });
        assert_eq!(asked.status, Status::Stopped);
        let crashed = apply(&idle(), &Event::Exited { requested: false });
        assert_eq!(crashed.status, Status::Crashed);
        assert_eq!(crashed.pid, None);
    }

    #[test]
    fn a_restart_counts_a_new_process() {
        let crashed = apply(&idle(), &Event::Exited { requested: false });
        let again = apply(&crashed, &Event::RestartAllowed);
        assert_eq!(again.status, Status::Starting);
        let respawned = apply(&again, &spawned());
        assert_eq!(respawned.process_seq, 2);
    }

    #[test]
    fn exhausted_restarts_stay_crashed() {
        let crashed = apply(&idle(), &Event::Exited { requested: false });
        assert_eq!(apply(&crashed, &Event::RestartExhausted), crashed);
    }

    #[test]
    fn a_misfit_event_changes_nothing() {
        let ready = idle();
        assert_eq!(apply(&ready, &Event::ApprovalResolved), ready);
        assert_eq!(
            apply(&State::default(), &Event::TurnStarted),
            State::default()
        );
        assert_eq!(
            apply(&State::default(), &Event::Exited { requested: false }),
            State::default()
        );
    }

    #[test]
    fn the_backoff_doubles_and_caps() {
        let secs: Vec<u64> = (1..=7).map(|n| backoff(n).as_secs()).collect();
        assert_eq!(secs, vec![1, 2, 4, 8, 16, 30, 30]);
        assert_eq!(backoff(0).as_secs(), 1);
        assert_eq!(backoff(u32::MAX).as_secs(), 30);
    }

    #[test]
    fn restarts_stop_past_the_maximum() {
        assert_eq!(
            restart_decision(1, 3),
            Restart::After(Duration::from_secs(1))
        );
        assert_eq!(
            restart_decision(3, 3),
            Restart::After(Duration::from_secs(4))
        );
        assert_eq!(restart_decision(4, 3), Restart::Exhausted);
        assert_eq!(restart_decision(1, 0), Restart::Exhausted);
    }

    #[test]
    fn the_json_shape_is_the_state_file() {
        let value = serde_json::to_value(idle()).unwrap();
        assert_eq!(value["status"], "idle");
        assert_eq!(value["sessionId"], "s1");
        assert_eq!(value["processSeq"], 1);
        assert_eq!(value["resumeFrom"], "old");
        let back: State = serde_json::from_value(value).unwrap();
        assert_eq!(back, idle());
        let paused: Status = serde_json::from_str("\"paused-budget\"").unwrap();
        assert_eq!(paused, Status::PausedBudget);
    }
}
