//! K1: The alert wake: two samples in a row, coalesced, biggest first, repeat and recovery.
// test-tag: critical

use super::harness::{Harness, Setup, Spec, T0, proc};
use argus_procs::{ALERT_TAIL, ProcRow};
use serde_json::json;

fn rules() -> serde_json::Value {
    json!([
        {"name": "dev", "match": {"name": "^node"}, "maxMemMB": 1000, "orphan": true, "killable": true},
        {"name": "claude", "match": {"name": "^claude"}, "maxMemMB": 1000, "orphan": true}
    ])
}

fn harness(repeat_ms: Option<u64>) -> Harness {
    Harness::new(Setup {
        rules: Some(rules()),
        repeat_ms,
        ..Setup::default()
    })
}

fn tool(mem: u64) -> ProcRow {
    proc(Spec {
        mem,
        cmd: "node x:/a/tool.js",
        ppid: 9999,
        ..Spec::new(602)
    })
}

fn bad() -> Vec<ProcRow> {
    vec![
        proc(Spec {
            mem: 1200,
            cmd: "node x:/a/node_modules/vite/bin/vite.js",
            ..Spec::new(601)
        }),
        tool(2000),
        proc(Spec {
            mem: 100,
            ppid: 9999,
            cmd: "node x:/a/y.js",
            ..Spec::new(603)
        }),
    ]
}

fn primed() -> Harness {
    let h = harness(Some(3_600_000));
    h.at(T0, bad());
    assert_eq!(
        h.fake.sent().len(),
        0,
        "the first sample after a start never alerts"
    );
    h.at(T0 + 30_000, bad());
    h
}

#[test]
fn a_flag_seen_twice_in_a_row_is_one_coalesced_wake_to_the_overlord_biggest_first() {
    let h = primed();
    let sent = h.fake.sent();
    assert_eq!(sent.len(), 1);
    let wake = &sent[0];
    let route = [
        wake.scope.as_str(),
        wake.thread.as_str(),
        wake.reason.as_str(),
        wake.subject.as_str(),
    ];
    assert_eq!(route, ["portfolio", "boss", "process-alert", "procs"]);
    assert_eq!(
        wake.text,
        format!(
            "3 processes need a look: node 602 (tool) 2 GB hog, orphan 10m, node 601 (vite) 1.2 GB hog, node 603 (y) 100 MB orphan 10m{ALERT_TAIL}"
        )
    );
    let lines: Vec<_> = h
        .fake
        .acts()
        .into_iter()
        .filter(|a| a.kind == "process-alert")
        .collect();
    assert_eq!(lines.len(), 1);
    assert!(!lines[0].text.ends_with(ALERT_TAIL));
    h.at(T0 + 60_000, bad());
    assert_eq!(h.fake.sent().len(), 1, "once per process and flag");
}

#[test]
fn a_recovered_flag_is_forgotten_and_needs_two_samples_to_be_due_again() {
    let h = primed();
    let mut recovered = bad();
    recovered[1] = tool(500);
    h.at(T0 + 90_000, recovered);
    h.at(T0 + 120_000, bad());
    assert_eq!(h.fake.sent().len(), 1);
    h.at(T0 + 150_000, bad());
    assert_eq!(h.fake.sent().len(), 2);
    assert_eq!(
        h.fake.sent()[1].text,
        format!("1 process needs a look: node 602 (tool) 2 GB hog{ALERT_TAIL}")
    );
    h.at(T0 + 30_000 + 3_600_000, bad());
    let sent = h.fake.sent();
    assert_eq!(sent.len(), 3);
    assert_eq!(
        sent[2].text,
        format!(
            "3 processes need a look: node 602 (tool) 2 GB orphan 1h, node 601 (vite) 1.2 GB hog, node 603 (y) 100 MB orphan 1h{ALERT_TAIL}"
        ),
        "the hog of 602 was sent an hour later: not due yet"
    );
}

#[test]
fn the_ones_still_bad_are_due_again_after_repeat_ms() {
    let h = primed();
    h.at(T0 + 90_000, bad());
    assert_eq!(h.fake.sent().len(), 1);
    h.at(T0 + 30_000 + 3_600_000, bad());
    let sent = h.fake.sent();
    assert_eq!(sent.len(), 2);
    assert_eq!(
        sent[1].text,
        format!(
            "3 processes need a look: node 602 (tool) 2 GB hog, orphan 1h, node 601 (vite) 1.2 GB hog, node 603 (y) 100 MB orphan 1h{ALERT_TAIL}"
        )
    );
}

#[test]
fn a_protected_process_alerts_only_for_its_memory_and_a_long_list_is_cut_to_fit() {
    let h = harness(None);
    let claude = |pid, mem| Spec {
        name: "claude.exe",
        cmd: "claude",
        ppid: 9999,
        mem,
        ..Spec::new(pid)
    };
    let pair = || vec![proc(claude(701, 1500)), proc(claude(702, 10))];
    h.at(T0, pair());
    h.at(T0 + 30_000, pair());
    let sent = h.fake.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(
        sent[0].text,
        format!("1 process needs a look: claude 701 1.5 GB hog (protected){ALERT_TAIL}")
    );

    let many = || {
        (0..20u32)
            .map(|i| ProcRow {
                cmd: format!("node x:/a/some-long-tool-name-{i}.js"),
                ..proc(Spec {
                    mem: 1100 + u64::from(i),
                    ..Spec::new(800 + i)
                })
            })
            .collect::<Vec<_>>()
    };
    let n = harness(None);
    n.at(T0, many());
    n.at(T0 + 30_000, many());
    let text = n.fake.sent()[0].text.clone();
    assert!(text.chars().count() <= 300, "{text}");
    assert!(
        text.starts_with("20 processes need a look: node 819 "),
        "{text}"
    );
    assert!(text.contains(" (+") && text.contains(" more)"), "{text}");
    assert!(text.ends_with(ALERT_TAIL), "{text}");
}
