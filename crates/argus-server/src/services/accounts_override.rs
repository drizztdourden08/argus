//! The owner's overrides of the switch process: which step is under way, the words of an
//! override, and its record in the activity log and the step log of the process.

use argus_store::{ActivityLevel, NewActivity};

use super::AccountsService;
use super::accounts_handover::Handovers;
use super::accounts_switch_log::{Line, LineState, Stage};
use crate::Services;

/// The kind of the activity line of an override.
pub(crate) const KIND: &str = "accounts-override";

/// The stage a word of the API names.
pub(crate) fn stage_named(word: &str) -> Option<Stage> {
    match word {
        "detect" => Some(Stage::Detect),
        "countdown" => Some(Stage::Countdown),
        "handover" => Some(Stage::Handover),
        "sync" => Some(Stage::Sync),
        "switch" => Some(Stage::Switch),
        "verify" => Some(Stage::Verify),
        _ => None,
    }
}

/// The stage under way: by the phase of the `switch` doc, and the handover stage too while a
/// handover cycle is open with no switch yet.
pub(crate) fn current_stage(accounts: &AccountsService, handovers: &Handovers) -> Option<Stage> {
    match accounts.switch_phase().as_str() {
        "countdown" => Some(Stage::Countdown),
        "handover" => Some(Stage::Handover),
        "sync" => Some(Stage::Sync),
        "switching" => Some(Stage::Switch),
        "verifying" => Some(Stage::Verify),
        _ => handovers
            .desk()
            .cycle()
            .filter(|c| c.phase.is_open())
            .map(|_| Stage::Handover),
    }
}

/// What the owner's skip of a step did, in a sentence.
pub(crate) fn step_text(stage: Stage, cycle: Option<&str>) -> String {
    let what = match stage {
        Stage::Countdown => "the grace minute ended early",
        Stage::Handover => "the wait for the handovers ended and the rest were cut off",
        Stage::Verify => "the account check was not waited for",
        Stage::Detect | Stage::Sync | Stage::Switch => "the step was skipped",
    };
    match cycle {
        Some(id) => format!("Owner skipped a step ({id}): {what}."),
        None => format!("Owner skipped a step: {what}."),
    }
}

/// What the owner's skip of one lead's wait did, in a sentence: the lead, its handover and the
/// cycle, and whether the handover file it had was used.
pub(crate) fn thread_text(lead: (&str, &str), ids: (&str, &str), used_file: bool) -> String {
    let how = if used_file {
        "its handover file was used"
    } else {
        "cut off"
    };
    format!(
        "Owner skipped the handover wait for {}/{} ({}, {}): {how}.",
        lead.0, lead.1, ids.0, ids.1
    )
}

/// Notes an override in the activity log and the step log of the process. Blocking.
pub(crate) fn record(svc: &Services, accounts: &AccountsService, stage: Stage, text: &str) {
    svc.activity
        .add(NewActivity::new(KIND, ActivityLevel::Info, text));
    accounts.note_switch(&[Line::new(stage, LineState::Skipped, text)]);
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::{Stage, stage_named, step_text, thread_text};

    #[test]
    fn a_stage_is_found_by_its_word() {
        assert_eq!(stage_named("handover"), Some(Stage::Handover));
        assert_eq!(stage_named("verify"), Some(Stage::Verify));
        assert_eq!(stage_named("nope"), None);
    }

    #[test]
    fn the_words_of_an_override_name_the_lead_the_handover_and_the_cycle() {
        assert_eq!(
            thread_text(("rotp", "lead"), ("H-12", "SW-4"), false),
            "Owner skipped the handover wait for rotp/lead (H-12, SW-4): cut off."
        );
        assert!(thread_text(("rotp", "lead"), ("H-12", "SW-4"), true).ends_with("file was used."));
        assert!(step_text(Stage::Countdown, Some("SW-4")).contains("(SW-4)"));
        assert!(step_text(Stage::Verify, None).starts_with("Owner skipped a step: "));
    }
}
