//! What a lead reads: the two wakes (each at most [`WAKE_MAX`] characters, the wake engine's
//! limit), the notice to the Overlord, and what Argus expects next of a handover, which
//! `coord handover status` prints in full.

use super::records::{Handover, HandoverState};
use crate::config::EstimateDefaults;

/// The longest text a wake carries.
pub const WAKE_MAX: usize = 300;

/// The path of a board's handover file, as a lead types it.
#[must_use]
pub fn handover_path(board: &str) -> String {
    format!("~/.coord/{board}/handover.md")
}

fn rubric(d: &EstimateDefaults) -> String {
    format!(
        "P = limit points your handover costs: opus agent {}, sonnet {}, haiku {}, yours {}, x{} over 100k context; M = minutes to a written handover",
        d.opus, d.sonnet, d.haiku, d.lead, d.long_context_factor
    )
}

/// The handover-estimate wake: one command to answer, then carry on.
#[must_use]
pub fn estimate_wake(defaults: &EstimateDefaults) -> String {
    format!(
        "Account switch ahead. Reply once and keep working: coord handover estimate --percent P --minutes M --agents N ({}).",
        rubric(defaults)
    )
}

/// The handover-request wake: agents to a safe point, the file, then `coord handover write`.
#[must_use]
pub fn request_wake(board: &str) -> String {
    format!(
        "Handover now, the account switches soon. Bring each running agent to a safe point (WIP committed to its branch) or stop it. Write {}, a section per agent (branch, stop point, next step), then run coord handover write. Steps: coord handover status",
        handover_path(board)
    )
}

/// The notice the Overlord gets when a lead is ready.
#[must_use]
pub fn ready_notice(h: &Handover) -> String {
    format!(
        "{}/{} handover ready ({}, {}): {}",
        h.board,
        h.thread,
        h.id,
        h.cycle,
        h.file.as_deref().unwrap_or("")
    )
}

/// What Argus expects next of a lead: a short word for scripts and the sentence for the lead.
#[must_use]
pub fn expects(h: &Handover, defaults: &EstimateDefaults) -> (&'static str, String) {
    let file = h.file.clone().unwrap_or_else(|| handover_path(&h.board));
    match h.state {
        HandoverState::EstimateAsked => ("estimate", format!(
            "Send your estimate once, then keep working: coord handover estimate --percent P --minutes M --agents N [--note \"<why>\"] ({}).",
            rubric(defaults)
        )),
        HandoverState::Estimated => ("wait", "Keep working. Argus asks for the handover at the ask point; send a new estimate if your agent count changes.".to_owned()),
        HandoverState::Asked | HandoverState::Writing => ("write", format!(
            "1. Start nothing new. 2. Tell each running agent to reach a safe point (commit its work in progress to its branch and report where it stopped); stop one that cannot within your estimate. 3. Write {file}: what is done and committed, one section per agent (id, branch, stop point, next step), what waits for the owner, the next steps in order. 4. Run coord handover write, with --agent \"<id>|<branch>|<stop point>\" once per agent. Then end your turn."
        )),
        HandoverState::Ready => ("stop", format!(
            "Your handover is recorded. Start nothing new and end your turn; after the switch a resume message points you at {file}."
        )),
        HandoverState::CutOff => ("resume", "The switch came before your handover. After the switch, resume from your chat and your agents' branches.".to_owned()),
        HandoverState::Resumed => ("resume", format!("Resumed after the switch: carry on from {file}.")),
        HandoverState::NotResumed => ("resume", format!(
            "Not resumed after the switch: read {file}, run coord inbox, check your agents by real id, carry on."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{WAKE_MAX, estimate_wake, request_wake};
    use crate::config::EstimateDefaults;

    #[test]
    fn the_estimate_wake_fits_and_carries_the_defaults() {
        let text = estimate_wake(&EstimateDefaults::default());
        assert!(text.chars().count() <= WAKE_MAX, "{}", text.chars().count());
        assert!(text.contains("coord handover estimate --percent P --minutes M --agents N"));
        assert!(text.contains("opus agent 0.3") && text.contains("haiku 0.03"));
    }

    #[test]
    fn the_request_wake_fits_for_a_long_board_key() {
        let text = request_wake("a-board-key-of-twenty");
        assert!(text.chars().count() <= WAKE_MAX, "{}", text.chars().count());
        assert!(text.contains("~/.coord/a-board-key-of-twenty/handover.md"));
        assert!(text.contains("coord handover write"));
    }
}
