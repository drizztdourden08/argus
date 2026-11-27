//! The board records of a handover cycle: one `SW-n` cycle per switch with one `H-n` handover
//! per lead that was working when the cycle started. Both serialize as camelCase; the cycle is
//! kept whole in the kv store and shown as the `cycle` doc plus one `Handovers` doc per lead.

use argus_core::lead::canonical;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, json};

use crate::types::AccountKey;

/// How a cycle reaches its handover request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CycleMode {
    /// Estimates first; the ask point is worked out from them.
    Auto,
    /// The ask point is a fixed percent: no estimate step.
    Fixed,
    /// The Switch now button: every lead is asked at once.
    Now,
}

/// Where a cycle stands. The countdown, the switch and the resume belong to the rules, the
/// switch and the resumer; this record only says when their part may begin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CyclePhase {
    /// The leads were asked for an estimate.
    Estimating,
    /// The leads were asked for their handover.
    Asked,
    /// Every asked lead is ready or was cut off: the countdown may start.
    Ready,
    /// A switch was made from the cycle, or every handover is resumed or not resumed. A done
    /// cycle never counts as ready again.
    Done,
    /// The owner or a rule called the cycle off before the switch.
    Cancelled,
}

impl CyclePhase {
    /// Whether the cycle still waits on its leads.
    #[must_use]
    pub fn is_open(self) -> bool {
        matches!(self, CyclePhase::Estimating | CyclePhase::Asked)
    }
}

/// Where one lead's handover stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoverState {
    /// The handover-estimate wake went out.
    EstimateAsked,
    /// The lead answered with an estimate.
    Estimated,
    /// The handover-request wake went out.
    Asked,
    /// The lead read the request (`coord handover status`) and is bringing its agents to a stop.
    Writing,
    /// The lead wrote its handover file and ran `coord handover write`.
    Ready,
    /// The switch came first; the lead resumes from its chat and branches.
    CutOff,
    /// The lead started again on the other account.
    Resumed,
    /// The lead did not start again after the switch.
    NotResumed,
}

impl HandoverState {
    /// Whether the lead still owes its handover.
    #[must_use]
    pub fn is_pending(self) -> bool {
        matches!(
            self,
            HandoverState::EstimateAsked
                | HandoverState::Estimated
                | HandoverState::Asked
                | HandoverState::Writing
        )
    }

    /// Whether the handover waits for the resume after the switch.
    #[must_use]
    pub fn awaits_resume(self) -> bool {
        matches!(self, HandoverState::Ready | HandoverState::CutOff)
    }
}

/// A lead's own estimate of what its handover costs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    /// Points of the limit the handover will use.
    pub percent: f64,
    /// Minutes until the handover file could be written.
    pub minutes: f64,
    /// How many agents the lead runs.
    pub agents: u32,
    /// A word from the lead, when it gave one.
    #[serde(default)]
    pub note: Option<String>,
    /// When it was given, ISO 8601.
    pub at: String,
}

/// One agent of a lead and where it stopped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStop {
    /// The agent record id (`A-12`).
    pub id: String,
    /// The branch its work is committed to.
    #[serde(default)]
    pub branch: Option<String>,
    /// Where it stopped, in the lead's words.
    #[serde(default)]
    pub stop: Option<String>,
}

/// A lead that is working now: its thread is inside a turn or it has a live agent record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lead {
    /// The board key (`portfolio` for the Overlord).
    pub board: String,
    /// The lead's thread id on that board.
    pub thread: String,
    /// The Claude Code session id of the thread.
    pub session_id: Option<String>,
    /// The Desktop chat link of the thread (`claude://.../local_<uuid>`).
    pub link: Option<String>,
    /// The lead's live agents, with the branch each works on.
    pub agents: Vec<AgentStop>,
}

/// A thread id read from a record; the old lead name reads as the lead.
fn lead_thread<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let raw = String::deserialize(deserializer)?;
    Ok(canonical(&raw).into_owned())
}

/// One lead's handover in a cycle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Handover {
    /// `H-n`.
    pub id: String,
    /// The `SW-n` it belongs to.
    pub cycle: String,
    /// The board key.
    pub board: String,
    /// The lead's thread id.
    #[serde(deserialize_with = "lead_thread")]
    pub thread: String,
    /// The Claude Code session id of the thread.
    #[serde(default)]
    pub session_id: Option<String>,
    /// The Desktop chat link, for the resume message.
    #[serde(default)]
    pub link: Option<String>,
    /// Where it stands.
    pub state: HandoverState,
    /// The lead's last estimate.
    #[serde(default)]
    pub estimate: Option<Estimate>,
    /// The handover file the lead wrote.
    #[serde(default)]
    pub file: Option<String>,
    /// The lead's agents: the live ones when the cycle started, then what the lead reported.
    #[serde(default)]
    pub agents: Vec<AgentStop>,
    /// When the handover-estimate wake last went out, ISO 8601.
    #[serde(default)]
    pub estimate_asked_at: Option<String>,
    /// When the handover-request wake went out, ISO 8601.
    #[serde(default)]
    pub asked_at: Option<String>,
    /// When the lead marked it ready, ISO 8601.
    #[serde(default)]
    pub ready_at: Option<String>,
}

impl Handover {
    /// Whether this is the handover of `board`/`thread`.
    #[must_use]
    pub fn is_of(&self, board: &str, thread: &str) -> bool {
        self.board == board && self.thread == thread
    }

    /// The `Handovers` doc.
    #[must_use]
    pub fn to_doc(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

/// One handover cycle, `SW-n`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cycle {
    /// `SW-n`.
    pub id: String,
    /// The account being left.
    pub account: AccountKey,
    /// How the request is reached.
    pub mode: CycleMode,
    /// Where it stands.
    pub phase: CyclePhase,
    /// When it started, ISO 8601.
    pub started_at: String,
    /// The percent at which the leads are asked (Auto and Fixed), when known.
    #[serde(default)]
    pub ask_at: Option<f64>,
    /// The calibration factor the ask point used.
    pub k: f64,
    /// The use of the limit when the leads were asked.
    #[serde(default)]
    pub use_at_request: Option<f64>,
    /// The use of the limit when the last lead was ready, or when the rest were cut off.
    #[serde(default)]
    pub use_at_ready: Option<f64>,
    /// One handover per lead that was working.
    pub leads: Vec<Handover>,
}

impl Cycle {
    /// The `cycle` doc of `AccountsState`: the handovers by id; each one is its own doc.
    #[must_use]
    pub fn to_doc(&self) -> Value {
        json!({
            "id": self.id,
            "account": self.account,
            "mode": self.mode,
            "phase": self.phase,
            "startedAt": self.started_at,
            "askAt": self.ask_at,
            "k": self.k,
            "useAtRequest": self.use_at_request,
            "useAtReady": self.use_at_ready,
            "leads": self.leads.iter().map(|h| h.id.as_str()).collect::<Vec<_>>(),
        })
    }

    /// The handover of `board`/`thread`.
    #[must_use]
    pub fn handover_of(&self, board: &str, thread: &str) -> Option<&Handover> {
        self.leads.iter().find(|h| h.is_of(board, thread))
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Cycle, CycleMode, CyclePhase, Handover, HandoverState};

    fn handover() -> Handover {
        Handover {
            id: "H-1".into(),
            cycle: "SW-1".into(),
            board: "rotp".into(),
            thread: "lead".into(),
            session_id: None,
            link: None,
            state: HandoverState::EstimateAsked,
            estimate: None,
            file: None,
            agents: Vec::new(),
            estimate_asked_at: None,
            asked_at: None,
            ready_at: None,
        }
    }

    #[test]
    fn the_cycle_doc_lists_its_handovers_by_id() {
        let cycle = Cycle {
            id: "SW-1".into(),
            account: "second".into(),
            mode: CycleMode::Auto,
            phase: CyclePhase::Estimating,
            started_at: "2026-11-11T10:00:00.000Z".into(),
            ask_at: None,
            k: 1.0,
            use_at_request: None,
            use_at_ready: None,
            leads: vec![handover()],
        };
        let doc = cycle.to_doc();
        assert_eq!(doc["leads"], json!(["H-1"]));
        assert_eq!(doc["mode"], json!("auto"));
        assert_eq!(doc["phase"], json!("estimating"));
    }

    #[test]
    fn a_handover_doc_uses_the_board_state_names() {
        let mut h = handover();
        h.state = HandoverState::CutOff;
        let doc = h.to_doc();
        assert_eq!(doc["state"], json!("cut_off"));
        assert_eq!(doc["sessionId"], json!(null));
    }

    #[test]
    fn a_handover_written_under_the_old_lead_name_reads_as_the_lead() {
        let mut doc = handover().to_doc();
        doc["thread"] = json!("coordinator");
        let read: Handover = serde_json::from_value(doc).unwrap();
        assert_eq!(read.thread, "lead");
    }

    #[test]
    fn a_cycle_survives_the_kv_round_trip() {
        let cycle = Cycle {
            id: "SW-2".into(),
            account: "hotmail".into(),
            mode: CycleMode::Now,
            phase: CyclePhase::Asked,
            started_at: "2026-11-11T10:00:00.000Z".into(),
            ask_at: Some(95.7),
            k: 0.9,
            use_at_request: Some(95.8),
            use_at_ready: None,
            leads: vec![handover()],
        };
        let text = serde_json::to_string(&cycle).unwrap();
        assert_eq!(serde_json::from_str::<Cycle>(&text).unwrap(), cycle);
    }
}
