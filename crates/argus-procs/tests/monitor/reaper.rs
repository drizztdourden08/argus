//! The reaper over a fake machine: the sample's time is the clock and the rows carry the CPU
//! times. Orphaned and long searches, idle and capped test or build runs, what it never takes,
//! shadow mode, the activity lines and the debounced wake.

use super::harness::{Harness, MIN, Setup, Spec, T0, proc};
use argus_procs::{Level, ProcRow, WakeRequest};
use serde_json::{Value, json};

const FIND: &str = "C:\\Program Files\\Git\\usr\\bin\\find.exe";
const NODE: &str = "C:\\nvm4w\\nodejs\\node.exe";
const VITEST: &str = "node x:/alpha/node_modules/vitest/vitest.mjs run";
const WORKER: &str = "node x:/alpha/node_modules/vitest/dist/workers/forks.js";
const CARGO: &str = "C:\\Users\\me\\.cargo\\bin\\cargo.exe";

fn harness(reaper: Value, self_pid: u32) -> Harness {
    let h = Harness::new(Setup {
        self_pid: Some(self_pid),
        reaper: Some(reaper),
        ..Setup::default()
    });
    h.fake
        .set_folders(&[("alpha", &["X:\\alpha"]), ("beta", &["X:\\beta"])]);
    h
}

/// A search: `ppid` 4242 is a dead parent, 100 the live explorer.
fn search(pid: u32, ppid: u32, start: i64, cwd: &'static str) -> ProcRow {
    proc(Spec {
        ppid,
        name: "find.exe",
        cmd: "find . -name tessera",
        path: FIND,
        start,
        cwd: Some(cwd),
        ..Spec::new(pid)
    })
}

fn run(pid: u32, ppid: u32, (cmd, start, cpu): (&'static str, i64, i64)) -> ProcRow {
    proc(Spec {
        ppid,
        cmd,
        start,
        cpu,
        cwd: Some("X:\\alpha"),
        ..Spec::new(pid)
    })
}

fn cargo(pid: u32, ppid: u32, cmd: &'static str) -> ProcRow {
    proc(Spec {
        ppid,
        name: "cargo.exe",
        cmd,
        path: CARGO,
        start: T0 - 40 * MIN,
        cwd: Some("X:\\alpha"),
        ..Spec::new(pid)
    })
}

fn reap_lines(h: &Harness) -> Vec<(Level, String)> {
    h.fake
        .acts()
        .into_iter()
        .filter(|a| a.kind == "proc-reap")
        .map(|a| (a.level, a.text))
        .collect()
}

fn wakes(h: &Harness) -> Vec<WakeRequest> {
    h.fake
        .sent()
        .into_iter()
        .filter(|w| w.reason == "process-reaped")
        .collect()
}

#[test]
fn an_orphaned_search_is_reaped_with_its_tree_one_line_and_one_wake() {
    let h = harness(json!({}), 1);
    h.at(T0, vec![search(7001, 4242, T0 - MIN, "X:\\alpha")]);
    assert_eq!(h.fake.trees(), [(7001, true, vec![])]);
    let line = format!("{FIND} 7001: an orphaned search (whatever started it is gone)");
    assert_eq!(
        reap_lines(&h),
        [(Level::Warn, format!("reaped {line} (project alpha)"))]
    );
    let sent = wakes(&h);
    assert_eq!(sent.len(), 1);
    let wake = &sent[0];
    assert_eq!(
        (
            wake.scope.as_str(),
            wake.thread.as_str(),
            wake.subject.as_str()
        ),
        ("alpha", "alpha-lead", "procs")
    );
    assert!(
        wake.text.starts_with(&format!(
            "Argus reaped a process tree of this project that hung: {line}. "
        )),
        "{}",
        wake.text
    );
}

#[test]
fn a_search_is_reaped_past_ten_minutes_and_never_outside_a_project_folder() {
    let h = harness(json!({}), 1);
    h.at(
        T0,
        vec![
            search(7002, 100, T0 - 11 * MIN, "X:\\alpha"),
            search(7003, 100, T0 - 5 * MIN, "X:\\alpha"),
            search(7004, 4242, T0 - 20 * MIN, "C:\\Users\\me"),
        ],
    );
    assert_eq!(h.fake.killed(), [7002]);
    let (_, line) = &reap_lines(&h)[0];
    assert!(
        line.ends_with("7002: a search running for 11m (project alpha)"),
        "{line}"
    );
}

/// A vitest run and two workers, each with `cpu` ms of CPU time.
fn vitest_tree(cpu: i64) -> Vec<ProcRow> {
    let start = T0 - MIN;
    vec![
        run(7101, 100, (VITEST, start, cpu)),
        run(7102, 7101, (WORKER, start, cpu)),
        run(7103, 7101, (WORKER, start, cpu)),
    ]
}

#[test]
fn a_run_whose_tree_gains_no_cpu_time_for_five_minutes_is_reaped_whole() {
    let h = harness(json!({}), 1);
    h.at(T0, vitest_tree(1000));
    h.at(T0 + 3 * MIN, vitest_tree(1500));
    assert!(h.fake.killed().is_empty());
    h.at(T0 + 6 * MIN, vitest_tree(1500));
    assert_eq!(h.fake.trees(), [(7101, true, vec![7102, 7103])]);
    let (level, line) = &reap_lines(&h)[0];
    assert_eq!(*level, Level::Warn);
    assert_eq!(
        line,
        &format!(
            "reaped {NODE} 7101 and its tree (2 more): a vitest run with no CPU time for 6m (project alpha)"
        )
    );
}

#[test]
fn a_busy_run_is_reaped_only_past_the_cap() {
    let h = harness(json!({}), 1);
    let busy = |cpu| vec![run(7111, 100, (VITEST, T0 - 29 * MIN, cpu))];
    h.at(T0, busy(0));
    h.at(T0 + MIN, busy(30_000));
    assert!(h.fake.killed().is_empty());
    h.at(T0 + 2 * MIN, busy(60_000));
    assert_eq!(h.fake.killed(), [7111]);
    let (_, line) = &reap_lines(&h)[0];
    assert!(
        line.ends_with("7111: a vitest run older than the 30m cap (project alpha)"),
        "{line}"
    );
}

#[test]
fn the_cap_follows_the_setting() {
    let h = harness(json!({ "capMs": 10 * MIN }), 1);
    h.at(T0, vec![run(7112, 100, (VITEST, T0 - 11 * MIN, 0))]);
    assert_eq!(h.fake.killed(), [7112]);
}

#[test]
fn cargo_is_reaped_with_rustc_under_it_but_a_program_it_runs_and_a_dev_servers_esbuild_are_not() {
    let h = harness(json!({}), 1);
    let rustc = proc(Spec {
        ppid: 7201,
        name: "rustc.exe",
        cmd: "rustc --crate-name argus_procs",
        cwd: Some("X:\\alpha"),
        start: T0 - 40 * MIN,
        ..Spec::new(7202)
    });
    let vite = run(
        7204,
        100,
        (
            "node x:/alpha/node_modules/vite/bin/vite.js",
            T0 - 40 * MIN,
            0,
        ),
    );
    let esbuild = proc(Spec {
        ppid: 7204,
        name: "esbuild.exe",
        cmd: "x:/alpha/node_modules/@esbuild/win32-x64/esbuild.exe --service=0.21.5 --ping",
        cwd: Some("X:\\alpha"),
        start: T0 - 40 * MIN,
        ..Spec::new(7205)
    });
    h.at(
        T0,
        vec![
            cargo(
                7201,
                100,
                "C:\\Users\\me\\.cargo\\bin\\cargo.exe test -p argus-procs",
            ),
            rustc,
            cargo(7203, 100, "cargo +stable run -p game"),
            vite,
            esbuild,
        ],
    );
    assert_eq!(h.fake.trees(), [(7201, true, vec![7202])]);
}

#[test]
fn a_tree_of_the_dashboard_or_holding_its_process_or_another_projects_is_never_reaped() {
    let h = harness(json!({}), 50);
    let dashboard = proc(Spec {
        name: "argus.exe",
        cmd: "X:\\argus\\argus.exe serve",
        path: "X:\\argus\\argus.exe",
        start: T0 - 60 * MIN,
        ..Spec::new(50)
    });
    let serve_child = proc(Spec {
        ppid: 7302,
        name: "argus.exe",
        cmd: "X:\\alpha\\target\\debug\\argus.exe serve --port 9",
        path: "X:\\alpha\\target\\debug\\argus.exe",
        cwd: Some("X:\\alpha"),
        ..Spec::new(7303)
    });
    let beta_child = proc(Spec {
        ppid: 7304,
        cwd: Some("X:\\beta"),
        cmd: "node build.js",
        ..Spec::new(7305)
    });
    h.at(
        T0,
        vec![
            dashboard,
            cargo(7301, 50, "cargo test"),
            cargo(7302, 100, "cargo test"),
            serve_child,
            cargo(7304, 100, "cargo build"),
            beta_child,
        ],
    );
    assert!(h.fake.killed().is_empty());
    assert!(reap_lines(&h).is_empty() && wakes(&h).is_empty());
}

#[test]
fn shadow_mode_writes_what_it_would_reap_once_and_kills_nothing() {
    let h = harness(json!({ "shadow": true }), 1);
    h.at(T0, vec![search(7001, 4242, T0 - MIN, "X:\\alpha")]);
    h.at(T0 + MIN, vec![search(7001, 4242, T0 - MIN, "X:\\alpha")]);
    assert!(h.fake.killed().is_empty() && wakes(&h).is_empty());
    assert_eq!(
        reap_lines(&h),
        [(
            Level::Info,
            format!(
                "would reap {FIND} 7001: an orphaned search (whatever started it is gone) (project alpha; shadow mode, nothing killed)"
            )
        )]
    );
}

#[test]
fn a_disabled_reaper_does_nothing() {
    let h = harness(json!({ "enabled": false }), 1);
    h.at(T0, vec![search(7001, 4242, T0 - 30 * MIN, "X:\\alpha")]);
    h.at(T0 + 10 * MIN, vitest_tree(0));
    assert!(h.fake.killed().is_empty() && reap_lines(&h).is_empty());
}

#[test]
fn a_failed_reap_writes_one_error_line_and_wakes_nobody() {
    let h = harness(json!({}), 1);
    h.fake.fail_kills();
    h.at(T0, vec![search(7001, 4242, T0 - MIN, "X:\\alpha")]);
    h.at(T0 + MIN, vec![search(7001, 4242, T0 - MIN, "X:\\alpha")]);
    let lines = reap_lines(&h);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].0, Level::Error);
    assert!(
        lines[0]
            .1
            .starts_with(&format!("could not reap {FIND} 7001: "))
    );
    assert!(lines[0].1.ends_with("(project alpha): access is denied"));
    assert!(wakes(&h).is_empty());
}

#[test]
fn the_wake_is_one_per_thread_per_batch_and_debounced() {
    let h = harness(json!({}), 1);
    h.at(
        T0,
        vec![
            search(7401, 4242, T0 - MIN, "X:\\alpha"),
            search(7402, 4242, T0 - MIN, "X:\\alpha\\src"),
            search(7404, 4242, T0 - MIN, "X:\\beta"),
        ],
    );
    let first = wakes(&h);
    let routes: Vec<(&str, &str)> = first
        .iter()
        .map(|w| (w.scope.as_str(), w.thread.as_str()))
        .collect();
    assert_eq!(routes, [("alpha", "alpha-lead"), ("beta", "beta-lead")]);
    assert!(
        first[0]
            .text
            .starts_with("Argus reaped 2 process trees of this project that hung: ")
    );
    assert!(first[0].text.contains(" 7401: ") && first[0].text.contains(" 7402: "));
    h.at(T0 + MIN, vec![search(7403, 4242, T0, "X:\\alpha")]);
    h.at(T0 + 5 * MIN, vec![]);
    assert_eq!(h.fake.killed(), [7401, 7402, 7404, 7403]);
    assert_eq!(wakes(&h).len(), 2);
    h.at(T0 + 11 * MIN, vec![]);
    let all = wakes(&h);
    assert_eq!(all.len(), 3);
    assert_eq!(all[2].thread, "alpha-lead");
    assert!(all[2].text.contains(" 7403: ") && !all[2].text.contains(" 7401: "));
}
