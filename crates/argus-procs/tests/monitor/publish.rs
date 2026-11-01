//! The store is written only when a reader would see a change, and at a slow heartbeat.

use super::harness::{Harness, Setup, Spec, T0, proc};
use argus_procs::{HEARTBEAT_MS, ProcRow};
use serde_json::json;

const STEP: i64 = 5_000;

fn harness() -> Harness {
    Harness::new(Setup {
        rules: Some(json!([
            {"name": "dev", "match": {"name": "^node"}, "killable": true}
        ])),
        ..Setup::default()
    })
}

fn one(cpu: i64, mem: u64) -> Vec<ProcRow> {
    vec![proc(Spec {
        cpu,
        mem,
        ..Spec::new(701)
    })]
}

fn two() -> Vec<ProcRow> {
    let mut rows = one(0, 100);
    rows.push(proc(Spec::new(702)));
    rows
}

#[test]
fn a_sample_that_changed_nothing_writes_nothing() {
    let h = harness();
    h.at(T0, one(0, 100));
    assert_eq!(h.fake.writes(), 1, "the first sample is always written");
    h.at(T0 + STEP, one(0, 100));
    h.at(T0 + 2 * STEP, one(0, 100));
    assert_eq!(h.fake.writes(), 1);
}

#[test]
fn a_row_added_or_gone_writes_at_once() {
    let h = harness();
    h.at(T0, one(0, 100));
    h.at(T0 + STEP, two());
    assert_eq!(h.fake.writes(), 2);
    assert_eq!(h.fake.published().unwrap().0.len(), 2);
    h.at(T0 + 2 * STEP, one(0, 100));
    assert_eq!(h.fake.writes(), 3);
    assert_eq!(h.fake.published().unwrap().0.len(), 1);
}

#[test]
fn a_changed_memory_waits_for_the_heartbeat() {
    let h = harness();
    h.at(T0, one(0, 100));
    h.at(T0 + STEP, one(0, 140));
    assert_eq!(h.fake.writes(), 1);
    h.at(T0 + HEARTBEAT_MS, one(0, 150));
    assert_eq!(h.fake.writes(), 2);
    assert_eq!(h.fake.published().unwrap().0[0].mem_mb, 150);
}

#[test]
fn only_the_cpu_changing_writes_nothing_until_the_heartbeat() {
    let h = harness();
    h.at(T0, one(0, 100));
    h.at(T0 + STEP, one(0, 100));
    h.at(T0 + 2 * STEP, one(4_000, 100));
    assert!(h.doc(701).cpu_pct.is_some_and(|pct| pct > 0.0));
    assert_eq!(h.fake.writes(), 1);
    let stale = h.fake.published().unwrap().0[0].cpu_pct;
    assert_ne!(stale, h.doc(701).cpu_pct);

    h.at(T0 + HEARTBEAT_MS - STEP, one(8_000, 100));
    assert_eq!(h.fake.writes(), 1);
    h.at(T0 + HEARTBEAT_MS, one(12_000, 100));
    assert_eq!(h.fake.writes(), 2);
    let (docs, meta) = h.fake.published().unwrap();
    assert_eq!(docs[0].cpu_pct, h.doc(701).cpu_pct);
    assert_eq!(meta, h.service.meta());
}

#[test]
fn a_failed_sample_writes_once_per_new_error() {
    let h = harness();
    h.at(T0, one(0, 100));
    h.fake.fail("powershell.exe failed: boom");
    let _ = h.service.sample_now();
    assert_eq!(h.fake.writes(), 2);
    let _ = h.service.sample_now();
    assert_eq!(h.fake.writes(), 2);
}

#[test]
fn republish_writes_again_though_nothing_changed() {
    let h = harness();
    h.at(T0, one(0, 100));
    h.fake.wipe();
    h.service.republish();
    assert_eq!(h.fake.writes(), 2);
    assert_eq!(h.fake.published().unwrap().0.len(), 1);
}
