//! Helpers shared by the request and turn specs: waiting for another thread, and the request of a second project.

use crate::board::Caller;
use crate::requests::{RequestInput, RequestStatus};
use crate::sync::locked;
use crate::testkit::Harness;

/// Yields until `done`: another thread sets the state, so no clock is read and nothing sleeps.
pub fn until(done: impl Fn() -> bool) {
    while !done() {
        std::thread::yield_now();
    }
}

/// Adds the project `beta` (never scanned), asks as the lead to remove `target` there by path, and waits until the
/// request is done.
pub fn beta_request_done(h: &Harness, target: String) {
    locked(&h.store.projects).push("beta".to_string());
    let mut input = RequestInput::new("beta", Caller::Thread("lead".into()));
    input.targets = vec![target];
    h.svc.request(&input).unwrap();
    until(|| {
        h.svc
            .request_list("beta")
            .first()
            .is_some_and(|r| r.status == RequestStatus::Done)
    });
}
