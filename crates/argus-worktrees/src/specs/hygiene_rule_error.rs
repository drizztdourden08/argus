//! KEEP: K3, K4.
//! A rule the scan cannot read skips only its own worktrees: the hygiene goes on with every other rule of the project,
//! and the error stays on the rule's summary.

// test-tag: standard

use crate::hygiene::HygieneState;
use crate::paths::join_path;
use crate::rule::Rule;
use crate::specs::hygiene_kit::{agent, agents_rule, hygiene_on, kind, through_debounce};
use crate::testkit::{Fixture, Harness};
use std::fs;

#[test]
fn a_rule_that_fails_to_scan_does_not_stop_the_hygiene_of_the_other_rules() {
    let fx = Fixture::fake();
    fx.add("agent-a1");
    let plain = join_path(&fx.root, "not-a-repository");
    fs::create_dir_all(&plain).unwrap();
    let broken = Rule::folder("W-2", &plain, "lead").repo();
    let h = Harness::new(&fx.root, vec![agents_rule(&fx), broken], hygiene_on);
    h.board().agents = vec![agent("1", "lead", "done", h.clock_now() - 1000)];
    h.scan();
    h.svc.start();
    let rules = h.meta().rules;
    let error = |id: &str| rules.iter().find(|r| r.id == id).unwrap().error.clone();
    assert_eq!(error("W-1"), None);
    assert!(error("W-2").is_some(), "the error stays on the rule");
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
    assert!(through_debounce(&h, &fx.path("agent-a1")));
}
