//! A batch of leftover deletions is only queued in memory and a folder is renamed when its job gets its turn, so a
//! restart used to lose the queued ones for good (the live bug of 2026-11-02: 26 leftovers of Tessera stayed "deleting
//! (queued)" and none went until the lead asked again). The batch is written down when queued, and the next server
//! queues what is left again.

// test-tag: standard

use crate::board::Caller;
use crate::intents::intents_key;
use crate::jobs::JobState;
use crate::sync::locked;
use crate::testkit::{Fixture, Gate, Harness, RemoveMode, backdate, exists_itself, put, rule_for};
use std::fs;
use std::sync::Arc;

const NAMES: [&str; 3] = ["old-1", "old-2", "old-3"];

fn world() -> (Fixture, Harness) {
    let fx = Fixture::fake();
    for name in NAMES {
        put(&fx.wts, &format!("{name}/a.txt"), "hello\n");
        backdate(&fx.path(name), 8.0);
    }
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    (fx, h)
}

/// Keeps the deletion queue of `alpha` shut, as a long job in front of it would.
fn shut_queue(h: &Harness) -> Arc<Gate> {
    let gate = Arc::new(Gate::default());
    let held = gate.clone();
    h.svc.queue.of("alpha").push(Box::new(move || held.wait()));
    gate
}

fn queue_batch(h: &Harness) {
    let out = h.batch(Caller::Thread("worker".into()), |_| {}).unwrap();
    assert_eq!(out.started, NAMES.len(), "{}", out.text);
}

fn restarted(h: &Harness, fx: &Fixture, budget_ms: Option<i64>) -> Harness {
    Harness::on_store(h.store.clone(), &fx.root, RemoveMode::Real, |o| {
        if let Some(ms) = budget_ms {
            o.budget_ms = ms;
        }
    })
}

fn written(h: &Harness) -> String {
    locked(&h.store.kv)
        .get(&intents_key("alpha"))
        .cloned()
        .unwrap_or_default()
}

fn nothing_written(h: &Harness) -> bool {
    written(h).replace(' ', "") == r#"{"pending":[]}"#
}

#[test]
fn a_queued_batch_a_restart_cut_off_is_queued_again_and_every_folder_goes() {
    let (fx, h) = world();
    let gate = shut_queue(&h);
    queue_batch(&h);
    assert!(
        NAMES.iter().all(|n| exists_itself(&fx.path(n))),
        "queued, none renamed or deleted"
    );
    assert!(
        h.svc
            .jobs("alpha")
            .iter()
            .all(|j| j.state == JobState::Queued)
    );
    for name in NAMES {
        assert!(written(&h).contains(&h.id_of(&fx.path(name))), "{name}");
    }

    let again = restarted(&h, &fx, None);
    assert!(
        again.svc.jobs("alpha").is_empty(),
        "the new server has no job"
    );
    again.scan();
    again.idle();

    for name in NAMES {
        assert!(!exists_itself(&fx.path(name)), "{name}");
    }
    let jobs = again.svc.jobs("alpha");
    assert_eq!(jobs.len(), NAMES.len());
    assert!(jobs.iter().all(|j| j.state == JobState::Done));
    assert!(nothing_written(&again), "{}", written(&again));
    gate.open();
    h.idle();
}

#[test]
fn a_folder_not_measured_at_the_restart_waits_for_a_later_scan_and_a_gone_one_is_forgotten() {
    let (fx, h) = world();
    let gate = shut_queue(&h);
    queue_batch(&h);
    fs::remove_dir_all(fx.path("old-2")).unwrap();

    let blind = restarted(&h, &fx, Some(0));
    blind.scan();
    blind.idle();
    assert!(blind.svc.jobs("alpha").is_empty());
    assert!(exists_itself(&fx.path("old-1")) && exists_itself(&fx.path("old-3")));
    assert!(!written(&blind).contains(&blind.id_of(&fx.path("old-2"))));
    assert!(written(&blind).contains(&blind.id_of(&fx.path("old-1"))));

    let seeing = restarted(&h, &fx, None);
    seeing.scan();
    seeing.idle();
    assert!(!exists_itself(&fx.path("old-1")) && !exists_itself(&fx.path("old-3")));
    assert!(nothing_written(&seeing), "{}", written(&seeing));
    gate.open();
    h.idle();
}

#[test]
fn a_batch_that_ran_to_its_end_leaves_nothing_to_resume() {
    let (fx, h) = world();
    queue_batch(&h);
    h.idle();
    assert!(NAMES.iter().all(|n| !exists_itself(&fx.path(n))));
    assert!(nothing_written(&h), "{}", written(&h));
    let again = restarted(&h, &fx, None);
    again.scan();
    again.idle();
    assert!(again.svc.jobs("alpha").is_empty());
}
