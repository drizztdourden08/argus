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

/// Adds the project `beta` once. It shares alpha's board, so its folders are alpha's.
pub fn add_beta(h: &Harness) {
    let mut projects = locked(&h.store.projects);
    if !projects.iter().any(|p| p == "beta") {
        projects.push("beta".to_string());
    }
}

/// Adds `beta` and scans it now. A project's first scan adopts every `.argus-deleting-` folder under its rules as a
/// crash leftover and finishes it with a job of its own. Beta's folders are alpha's, so a first scan of beta while alpha
/// is deleting one would start a second job on the same folder. Scanning before alpha deletes anything avoids that.
pub fn add_beta_scanned(h: &Harness) {
    add_beta(h);
    h.svc.scan_now("beta");
}

/// Adds the project `beta` (unless it is there), asks as the lead to remove `target` there by path, and waits until
/// the request is done.
pub fn beta_request_done(h: &Harness, target: String) {
    add_beta(h);
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
