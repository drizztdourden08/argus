//! KEEP: K5.
//! The turn of scans and removals is a project's own (the live bug of 2026-11-01): Argus's request R-3, three empty
//! leftover folders, stayed on "waiting for its turn" for minutes while the one turn of every project went from a
//! `node_modules` clear in Brock to one in Tessera to one in Archipelia, and its deletion job would then have queued
//! behind theirs. A wait for the project's own turn past a while names the holder.

// test-tag: e2e

use crate::board::Caller;
use crate::progress::WAITING;
use crate::requests::{RequestInput, RequestStatus};
use crate::sync::locked;
use crate::testkit::{Gate, Harness, backdate, exists_itself, put, started};
use std::sync::Arc;

fn until(done: impl Fn() -> bool) {
    while !done() {
        std::thread::yield_now();
    }
}

#[test]
#[ignore = "e2e"]
fn a_turn_and_a_deletion_job_held_by_one_project_do_not_hold_up_another_projects_leftover_request()
{
    let (fx, h) = started(|fx| {
        let dir = fx.path("left-beta");
        put(&dir, "a.txt", "left over\n");
        backdate(&dir, 3.0);
    });
    h.svc.start();
    let gate = Arc::new(Gate::default());
    let shut = gate.clone();
    h.svc.queue.of("alpha").push(Box::new(move || shut.wait()));
    let turn = h
        .svc
        .one_at_a_time("alpha", "the removal of a big worktree for lead", None);
    locked(&h.store.projects).push("beta".to_string());
    let mut input = RequestInput::new("beta", Caller::Thread("lead".into()));
    input.targets = vec![fx.path("left-beta")];
    h.svc.request(&input).unwrap();
    until(|| {
        h.svc
            .request_list("beta")
            .first()
            .is_some_and(|r| r.status == RequestStatus::Done)
    });
    let beta = h.svc.request_list("beta").remove(0);
    assert!(
        beta.results[0].text.starts_with("deleted leftover folder "),
        "{:?}",
        beta.results
    );
    assert!(!exists_itself(&fx.path("left-beta")));
    drop(turn);
    gate.open();
    h.idle();
    assert!(locked(&h.notes.problems).is_empty());
}

#[test]
#[ignore = "e2e"]
fn a_removal_that_waits_for_its_projects_turn_past_a_while_names_the_holder_and_its_step() {
    let (_fx, h) = started(|_| {});
    let merged = h.fact("old-merged").id;
    let holder = h.fact("dirty-old").id;
    h.svc.step(&holder, "node_modules clear");
    h.svc.step_done(&holder, 674);
    let what = format!("the removal of {holder} for lead");
    let turn = h.svc.one_at_a_time("alpha", &what, Some(&holder));
    let svc = h.svc.clone();
    let waiting = merged.clone();
    let removal = std::thread::spawn(move || {
        let _ = svc.remove(&Harness::input(&waiting));
    });
    until(|| {
        h.svc
            .step_of("alpha", &merged)
            .is_some_and(|s| s.name == WAITING)
    });
    h.clock.advance(31_000);
    let step = h.svc.step_of("alpha", &merged).unwrap();
    assert!(
        step.name
            .starts_with(&format!("waiting for its turn: {what} has held it since ")),
        "{}",
        step.name
    );
    assert!(
        step.name
            .ends_with(" (31 s), at node_modules clear, 674 entries removed"),
        "{}",
        step.name
    );
    drop(turn);
    removal.join().unwrap();
}
