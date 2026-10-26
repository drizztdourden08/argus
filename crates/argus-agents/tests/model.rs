//! The model rules of a sub-agent record: `--model` and `--why` are both required, the 12 character
//! minimum and 200 character limit, opus needs a category (or "owner asked: ..."), an opus chore
//! or test needs an owner request, and an opus-level model needs a check-in of 15 to 30 minutes.
//! The pure part of `agent.model.test.mjs`; the CLI output cases belong to the coord port.

use argus_agents::model_rules::{
    CHECKIN_HELP, MODEL_STANDARD, RuleError, check_checkin, check_model_why, high_tier,
};

fn refused(model: Option<&str>, why: Option<&str>, kind: &str) -> String {
    check_model_why(model, why, kind).unwrap_err().0
}

fn accepted(model: &str, why: &str, kind: &str) -> (String, String) {
    let ok = check_model_why(Some(model), Some(why), kind).unwrap();
    (ok.model, ok.why)
}

#[test]
fn refuses_a_missing_model_a_missing_why_a_short_or_long_why_and_a_two_word_model() {
    let known = Some("known cause, one module");
    assert!(refused(None, known, "build").contains("--model is required"));
    assert!(refused(Some("sonnet"), None, "build").contains("--why is required"));
    assert!(refused(Some("sonnet"), Some("too short"), "build").contains("minimum is 12"));
    let long = "x".repeat(201);
    assert!(refused(Some("sonnet"), Some(long.as_str()), "build").contains("limit is 200"));
    assert!(refused(Some("two words"), known, "build").contains("one word"));
}

#[test]
fn haiku_and_sonnet_take_any_justification_of_12_chars_trimmed_and_an_unknown_one_word_model_is_accepted()
 {
    let (_, why) = accepted("haiku", "  runs the tests and lint  ", "test");
    assert_eq!(why, "runs the tests and lint");
    assert_eq!(
        accepted("sonnet", "known cause, one-module fix", "build").0,
        "sonnet"
    );
    assert_eq!(
        accepted("newmodel-9", "a newer model for this", "build").0,
        "newmodel-9"
    );
}

#[test]
fn opus_without_a_category_is_refused_with_the_standard_and_an_example() {
    let why = Some("this looks hard to me");
    let message = refused(Some("opus"), why, "build");
    assert!(message.contains("design, architecture, unknown-cause, risky-review, plan"));
    assert!(message.contains(MODEL_STANDARD));
    assert!(message.contains("--why \"design: new document model touches every module\""));
    assert!(
        refused(Some("opus"), Some("explain the thing at length"), "build")
            .contains("one of these categories")
    );
}

#[test]
fn opus_with_a_category_word_is_accepted_for_build_and_research() {
    let cases = [
        ("design: new document model touches every module", "build"),
        ("Unknown-Cause crash only on exit", "research"),
        ("risky-review of the migration", "build"),
        ("plan the rollout across projects", "build"),
        ("architecture: split the store", "build"),
    ];
    for (why, kind) in cases {
        assert_eq!(accepted("opus", why, kind).1, why);
    }
}

#[test]
fn opus_for_a_chore_or_test_is_refused_unless_the_why_starts_with_owner() {
    let chore = refused(
        Some("opus"),
        Some("design: tidy the folder layout"),
        "chore",
    );
    assert!(chore.contains("not for chore agents") && chore.contains("owner asked"));
    assert!(
        refused(Some("opus"), Some("plan: run the whole suite"), "test")
            .contains("not for test agents")
    );
    for (why, kind) in [
        ("owner asked: use opus for the cleanup", "chore"),
        ("owner asked: opus", "test"),
        ("owner asked: no reason given", "build"),
    ] {
        assert_eq!(accepted("opus", why, kind).0, "opus");
    }
}

#[test]
fn a_new_model_for_an_existing_record_needs_a_new_why_with_the_same_rules() {
    assert!(refused(Some("sonnet"), None, "chore").contains("--why is required"));
    assert!(refused(Some("sonnet"), Some("short"), "chore").contains("minimum is 12"));
    assert!(
        refused(Some("opus"), Some("design: nope nope"), "chore").contains("not for chore agents")
    );
    assert_eq!(
        accepted("sonnet", "it turned out to need code changes", "chore").1,
        "it turned out to need code changes"
    );
}

#[test]
fn checkin_is_required_for_an_opus_level_model_whole_minutes_from_15_to_30() {
    let message = check_checkin(Some("opus"), None).unwrap_err().0;
    assert!(message.contains("--checkin is required for opus") && message.contains(CHECKIN_HELP));
    for bad in ["14", "31", "0", "20.5", "soon", "-20", "1e1"] {
        let err: RuleError = check_checkin(Some("opus"), Some(bad)).unwrap_err();
        assert!(
            err.0
                .contains("--checkin must be a whole number of minutes from 15 to 30"),
            "{bad}"
        );
    }
    assert_eq!(check_checkin(Some("opus"), Some("15")), Ok(Some(15)));
    assert_eq!(check_checkin(Some("opus"), Some("30")), Ok(Some(30)));
}

#[test]
fn checkin_is_optional_and_not_kept_for_haiku_and_sonnet_but_a_bad_value_is_still_refused() {
    assert_eq!(check_checkin(Some("sonnet"), None), Ok(None));
    assert_eq!(check_checkin(Some("haiku"), Some("20")), Ok(None));
    assert!(
        check_checkin(Some("sonnet"), Some("99"))
            .unwrap_err()
            .0
            .contains("from 15 to 30")
    );
    assert_eq!(check_checkin(None, None), Ok(None));
    assert_eq!(check_checkin(None, Some("22")), Ok(Some(22)));
}

#[test]
fn a_model_above_haiku_and_sonnet_is_high_tier() {
    assert!(high_tier("opus") && high_tier("fable") && high_tier("claude-opus-5-5"));
    assert!(!high_tier("sonnet") && !high_tier("Haiku") && !high_tier(""));
}
