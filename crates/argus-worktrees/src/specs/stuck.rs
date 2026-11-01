//! KEEP: K5.
//! A request never stays `running` on one target: a removal that does not come back within the target limit (here a
//! git that hangs past its own timeout) fails with a sentence, and the request goes on with the next target. While a
//! target runs, the record names it.

// test-tag: e2e

use crate::board::Caller;
use crate::requests::{Current, RequestInput};
use crate::sync::locked;
use crate::testkit::{Fixture, Gate, Harness, RemoveMode, rule_for};
use std::sync::{Arc, Mutex};

#[test]
#[ignore = "e2e"]
fn a_git_that_hangs_past_its_timeout_fails_the_target_and_the_request_goes_on() {
    let fx = Fixture::new();
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |o| o.target_limit_ms = 3000);
    h.scan();
    h.svc.start();
    let (merged, dirty) = (h.fact("old-merged").id, h.fact("dirty-old").id);
    *locked(&h.exec.mode) = RemoveMode::Untouched;
    let seen: Arc<Mutex<Option<Current>>> = Arc::default();
    let gate = Arc::new(Gate::default());
    let (svc, held, hold) = (h.svc.clone(), seen.clone(), gate.clone());
    *locked(&h.exec.on_remove) = Some(Box::new(move |_| {
        *locked(&held) = svc.request_list("alpha")[0].current.clone();
        hold.wait();
    }));
    let mut input = RequestInput::new("alpha", Caller::Thread("lead".into()));
    input.targets = vec![merged, dirty];
    h.svc.request(&input).unwrap();
    h.svc.requests_idle();
    let list = h.svc.request_list("alpha");
    let r = &list[0];
    assert!(!r.status.active(), "the request ended: {:?}", r.status);
    assert_eq!(r.results.len(), 2, "both targets have an outcome");
    assert_eq!(r.results[0].code.as_deref(), Some("timed_out"));
    assert!(
        r.results[0]
            .text
            .contains("the removal did not finish within 3 s, so the request went on without it"),
        "{}",
        r.results[0].text
    );
    assert!(!r.results[1].ok, "the dirty one is never removed");
    assert!(r.current.is_none(), "a finished request is on no target");
    let current = locked(&seen).clone().expect("the record named its target");
    assert_eq!(current.id, r.results[0].id, "the target it was on");
    assert!(current.label.ends_with("/old-merged"), "{}", current.label);
    let step = current.step.expect("the record named the removal's step");
    assert_eq!(step.name, "git worktree remove");
    assert!(
        r.results[0]
            .text
            .contains("(it was at: git worktree remove, for "),
        "{}",
        r.results[0].text
    );
    gate.open();
}
