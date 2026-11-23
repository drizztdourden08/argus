use argus_core::clock::iso_from_ms;
use serde_json::json;

use super::*;

const NOW: i64 = 1_800_000_000_000;
const MIN: i64 = 60_000;

fn input(every_min: i64) -> RuleInput {
    RuleInput {
        title: "Sweep".to_owned(),
        prompt: "look".to_owned(),
        every_min,
        enabled: true,
    }
}

fn rule(every_min: i64) -> Rule {
    create("OR-9", &input(every_min), By::Owner, NOW).unwrap()
}

// test-tag: critical
#[test]
fn a_new_rule_is_due_one_period_later() {
    let made = rule(15);
    assert_eq!(made.next_run_at, iso_from_ms(NOW + 15 * MIN));
    assert_eq!(made.created_at, iso_from_ms(NOW));
    assert_eq!(made.last_run_at, None);
}

#[test]
fn the_period_is_five_to_1440_minutes() {
    assert!(create("OR-1", &input(4), By::Owner, NOW).is_err());
    assert!(create("OR-1", &input(5), By::Owner, NOW).is_ok());
    assert!(create("OR-1", &input(1440), By::Owner, NOW).is_ok());
    assert!(create("OR-1", &input(1441), By::Owner, NOW).is_err());
    assert!(create("OR-1", &input(-5), By::Owner, NOW).is_err());
}

#[test]
fn the_title_and_prompt_have_limits() {
    let mut bad = input(15);
    bad.title = "  ".to_owned();
    assert!(create("OR-1", &bad, By::Owner, NOW).is_err());
    bad.title = "t".repeat(81);
    assert!(create("OR-1", &bad, By::Owner, NOW).is_err());
    bad.title = "t".repeat(80);
    bad.prompt = "p".repeat(4001);
    assert!(create("OR-1", &bad, By::Owner, NOW).is_err());
    bad.prompt = "p".repeat(4000);
    assert!(create("OR-1", &bad, By::Owner, NOW).is_ok());
}

#[test]
fn a_refusal_is_a_plain_sentence() {
    let text = create("OR-1", &input(1), By::Owner, NOW).unwrap_err();
    assert_eq!(text, "everyMin must be 5 to 1440 minutes");
}

#[test]
fn only_an_enabled_rule_past_its_time_is_due() {
    let mut late = rule(15);
    late.next_run_at = iso_from_ms(NOW - MIN);
    let mut off = late.clone();
    off.id = "OR-10".to_owned();
    off.enabled = false;
    let mut early = late.clone();
    early.id = "OR-11".to_owned();
    early.next_run_at = iso_from_ms(NOW + MIN);
    assert_eq!(due(&[late, off, early], NOW), vec!["OR-9".to_owned()]);
}

#[test]
fn a_rule_is_due_at_its_exact_time() {
    let mut exact = rule(15);
    exact.next_run_at = iso_from_ms(NOW);
    assert_eq!(due(&[exact], NOW).len(), 1);
}

#[test]
fn a_run_moves_the_next_time_a_period_on() {
    let ran = after_run(&rule(30), NOW + 90 * MIN);
    assert_eq!(ran.last_run_at, Some(iso_from_ms(NOW + 90 * MIN)));
    assert_eq!(ran.next_run_at, iso_from_ms(NOW + 120 * MIN));
    assert_eq!(ran.title, "Sweep");
}

#[test]
fn the_text_names_the_rule_its_period_and_the_prompt() {
    assert_eq!(
        prompt_text(&rule(15)),
        "Rule OR-9 \"Sweep\" (every 15 min): look"
    );
}

#[test]
fn a_patch_keeps_what_it_does_not_name() {
    let change = RulePatch {
        title: Some("Other".to_owned()),
        ..RulePatch::default()
    };
    let next = patch(&rule(15), &change, NOW + MIN).unwrap();
    assert_eq!(next.title, "Other");
    assert_eq!(next.prompt, "look");
    assert_eq!(next.next_run_at, iso_from_ms(NOW + 15 * MIN));
    assert_eq!(next.updated_at, iso_from_ms(NOW + MIN));
}

#[test]
fn a_new_period_or_switching_on_reschedules() {
    let change = RulePatch {
        every_min: Some(60),
        ..RulePatch::default()
    };
    let next = patch(&rule(15), &change, NOW + MIN).unwrap();
    assert_eq!(next.next_run_at, iso_from_ms(NOW + 61 * MIN));
    let mut off = rule(15);
    off.enabled = false;
    let on = RulePatch {
        enabled: Some(true),
        ..RulePatch::default()
    };
    let back = patch(&off, &on, NOW + 5 * MIN).unwrap();
    assert_eq!(back.next_run_at, iso_from_ms(NOW + 20 * MIN));
}

#[test]
fn a_bad_patch_is_refused() {
    let change = RulePatch {
        every_min: Some(1),
        ..RulePatch::default()
    };
    assert!(patch(&rule(15), &change, NOW).is_err());
}

#[test]
fn the_next_due_time_is_the_soonest_enabled() {
    let mut soon = rule(15);
    soon.next_run_at = iso_from_ms(NOW + MIN);
    let mut later = soon.clone();
    later.next_run_at = iso_from_ms(NOW + 9 * MIN);
    let mut off = soon.clone();
    off.enabled = false;
    off.next_run_at = iso_from_ms(NOW);
    assert_eq!(
        next_due_at(&[later, off, soon]),
        Some(iso_from_ms(NOW + MIN))
    );
    assert_eq!(next_due_at(&[]), None);
}

#[test]
fn the_next_rule_is_the_enabled_one_due_first() {
    let mut soon = rule(15);
    soon.id = "OR-1".to_owned();
    soon.next_run_at = iso_from_ms(NOW + MIN);
    let mut later = soon.clone();
    later.id = "OR-2".to_owned();
    later.next_run_at = iso_from_ms(NOW + 9 * MIN);
    let mut off = soon.clone();
    off.enabled = false;
    off.next_run_at = iso_from_ms(NOW);
    let all = [later, off, soon];
    assert_eq!(next_rule(&all).map(|r| r.id.as_str()), Some("OR-1"));
    assert_eq!(next_rule(&[]), None);
}

#[test]
fn the_seed_is_three_owner_rules() {
    let rules = seed(NOW);
    let shape: Vec<(&str, u32)> = rules
        .iter()
        .map(|r| (r.title.as_str(), r.every_min))
        .collect();
    assert_eq!(
        shape,
        vec![
            ("Board sweep", 15),
            ("Defender watch", 60),
            ("Daily sync window", 1440)
        ]
    );
    assert!(rules.iter().all(|r| r.created_by == By::Owner && r.enabled));
    assert_eq!(rules[2].id, "OR-3");
}

#[test]
fn a_body_parses_and_a_wrong_type_is_named() {
    let body = json!({ "title": "t", "prompt": "p", "everyMin": 10 });
    let parsed = parse_input(&body).unwrap();
    assert_eq!(parsed.every_min, 10);
    assert!(parsed.enabled);
    let missing = parse_input(&json!({ "title": "t" })).unwrap_err();
    assert_eq!(missing, "send the prompt as text");
    let wrong = parse_patch(&json!({ "enabled": "yes" })).unwrap_err();
    assert_eq!(wrong, "enabled must be true or false");
    assert_eq!(parse_patch(&json!({})).unwrap(), RulePatch::default());
}

#[test]
fn a_rule_serializes_in_camel_case() {
    let value = serde_json::to_value(rule(15)).unwrap();
    assert_eq!(value["everyMin"], 15);
    assert_eq!(value["createdBy"], "owner");
    assert!(value["lastRunAt"].is_null());
}
