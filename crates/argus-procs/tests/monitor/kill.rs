//! Kill guards and kill-rule, over a fake killer that records the pids it was asked to end.

use super::harness::{Harness, MIN, Setup, Spec, T0, proc};
use argus_procs::{ErrorCode, KillRequest, KillRuleRequest, Level, Source};
use chrono::{DateTime, SecondsFormat};
use serde_json::{Value, json};

fn iso(ms: i64) -> String {
    DateTime::from_timestamp_millis(ms)
        .unwrap()
        .to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn harness(rules: Value) -> Harness {
    Harness::new(Setup {
        rules: Some(rules),
        self_pid: Some(1),
        ..Setup::default()
    })
}

fn request(pid: u32, started_at: Option<String>) -> KillRequest {
    KillRequest {
        pid,
        started_at,
        ..KillRequest::default()
    }
}

fn code_and_message(h: &Harness, req: &KillRequest) -> (ErrorCode, String) {
    let err = h.service.kill(req).unwrap_err();
    (err.code(), err.message().to_string())
}

fn listed_fixture() -> (Harness, i64) {
    let rules = json!([
        {"name": "killme", "match": {"cmd": "procs-kill-test"}, "killable": true, "maxMemMB": 10},
        {"name": "keep", "match": {"cmd": "procs-keep-test"}}
    ]);
    let h = harness(rules);
    let start = T0 - 5 * MIN;
    h.at(
        T0,
        vec![
            proc(Spec {
                cmd: "node procs-kill-test",
                start,
                ..Spec::new(5000)
            }),
            proc(Spec {
                cmd: "node procs-keep-test",
                ..Spec::new(902)
            }),
            proc(Spec {
                name: "svchost.exe",
                cmd: "procs-kill-test",
                path: "C:\\x\\svchost.exe",
                ..Spec::new(903)
            }),
        ],
    );
    (h, start)
}

#[test]
fn a_kill_is_refused_for_an_unlisted_unkillable_protected_or_reused_pid_and_nothing_dies() {
    let (h, start) = listed_fixture();
    let ten = Some(iso(T0 - 10 * MIN));
    assert_eq!(
        h.service
            .kill(&request(4242, Some(iso(start))))
            .unwrap_err()
            .code(),
        ErrorCode::NotFound
    );
    let (code, message) = code_and_message(&h, &request(902, ten.clone()));
    assert_eq!(code, ErrorCode::Forbidden);
    assert!(message.contains("not killable"), "{message}");
    let (code, message) = code_and_message(&h, &request(903, ten));
    assert_eq!(code, ErrorCode::Forbidden);
    assert!(message.contains("protected (system)"), "{message}");
    let (code, message) = code_and_message(&h, &request(5000, Some(iso(start + 1000))));
    assert_eq!(code, ErrorCode::Conflict);
    assert!(message.contains("reused"), "{message}");
    assert_eq!(
        h.service.kill(&request(5000, None)).unwrap_err().code(),
        ErrorCode::BadRequest
    );
    assert_eq!(h.fake.killed(), Vec::<u32>::new());
    let warns = h
        .fake
        .acts()
        .iter()
        .filter(|a| a.kind == "process-kill" && a.level == Level::Warn)
        .count();
    assert_eq!(warns, 4, "the four refusals");
}

#[test]
fn a_listed_unprotected_process_of_a_killable_rule_with_its_creation_time_dies_and_is_logged() {
    let (h, start) = listed_fixture();
    let ask = KillRequest {
        reason: Some("test leftover".to_string()),
        source: Source::Cli {
            actor: Some("boss".to_string()),
        },
        ..request(5000, Some(iso(start)))
    };
    let out = h.service.kill(&ask).unwrap();
    assert_eq!(
        (out.killed, out.pid, out.name.as_str()),
        (true, 5000, "node.exe")
    );
    assert_eq!(h.fake.killed(), [5000]);
    let kills: Vec<_> = h
        .fake
        .acts()
        .into_iter()
        .filter(|a| a.kind == "process-kill")
        .collect();
    assert_eq!(
        kills.last().unwrap().text,
        "killed node 5000 (procs-kill-test, 100 MB) from the CLI (boss): test leftover"
    );
}

#[test]
fn a_tree_kill_is_refused_when_the_tree_holds_a_protected_process() {
    let h =
        harness(json!([{"name": "killme", "match": {"cmd": "procs-kill-test"}, "killable": true}]));
    h.at(
        T0,
        vec![
            proc(Spec {
                cmd: "node procs-kill-test",
                start: T0 - 9 * MIN,
                ..Spec::new(1001)
            }),
            proc(Spec {
                name: "claude.exe",
                cmd: "claude",
                ppid: 1001,
                start: T0 - 8 * MIN,
                ..Spec::new(1002)
            }),
        ],
    );
    let tree = KillRequest {
        tree: true,
        ..request(1001, Some(iso(T0 - 9 * MIN)))
    };
    let (code, message) = code_and_message(&h, &tree);
    assert_eq!(code, ErrorCode::Forbidden);
    assert!(
        message.contains("holds a protected process (claude 1002"),
        "{message}"
    );
    assert_eq!(h.fake.killed(), Vec::<u32>::new());
}

fn killme_idle() -> Value {
    json!([{"name": "killme", "match": {"cmd": "procs-kill-test"}, "killable": true, "idleMin": 1}])
}

fn busy(pid: u32, t: i64) -> argus_procs::ProcRow {
    proc(Spec {
        cmd: "node procs-kill-test b",
        start: T0 - 5 * MIN,
        cpu: t - T0,
        ..Spec::new(pid)
    })
}

#[test]
fn kill_rule_ends_the_old_enough_and_skips_the_rest() {
    let h = harness(killme_idle());
    let procs = |t| {
        vec![
            proc(Spec {
                cmd: "node procs-kill-test a",
                start: T0 - 120 * MIN,
                ..Spec::new(6001)
            }),
            busy(6002, t),
        ]
    };
    h.at(T0, procs(T0));
    h.at(T0 + 90_000, procs(T0 + 90_000));
    let unknown = KillRuleRequest {
        rule: "nope".to_string(),
        ..KillRuleRequest::default()
    };
    assert_eq!(
        h.service.kill_rule(&unknown).unwrap_err().code(),
        ErrorCode::NotFound
    );
    let ask = KillRuleRequest {
        rule: "killme".to_string(),
        older_than_min: Some(60.0),
        ..KillRuleRequest::default()
    };
    let out = h.service.kill_rule(&ask).unwrap();
    assert_eq!(out.killed.iter().map(|k| k.pid).collect::<Vec<_>>(), [6001]);
    assert_eq!(
        out.skipped
            .iter()
            .map(|k| (k.pid, k.why.as_str()))
            .collect::<Vec<_>>(),
        [(6002, "younger than 60 min")]
    );
    let acts = h.fake.acts();
    assert!(acts.iter().any(|a| a.kind == "process-kill"
        && a.text.starts_with("killed node ")
        && a.text.ends_with(" from the page")));
}

#[test]
fn kill_rule_idle_only_ends_the_quiet_one_and_keeps_the_busy_one() {
    let h = harness(killme_idle());
    let procs = |t| {
        vec![
            proc(Spec {
                cmd: "node procs-kill-test c",
                start: T0 - 5 * MIN,
                ..Spec::new(6003)
            }),
            busy(6002, t),
        ]
    };
    h.at(T0, procs(T0));
    h.at(T0 + 90_000, procs(T0 + 90_000));
    let ask = KillRuleRequest {
        rule: "killme".to_string(),
        idle_only: true,
        ..KillRuleRequest::default()
    };
    let out = h.service.kill_rule(&ask).unwrap();
    assert_eq!(out.killed.iter().map(|k| k.pid).collect::<Vec<_>>(), [6003]);
    assert_eq!(
        out.skipped
            .iter()
            .map(|k| (k.pid, k.why.as_str()))
            .collect::<Vec<_>>(),
        [(6002, "not idle for 1 min")]
    );
    assert_eq!(h.fake.killed(), [6003]);
}
