//! A thread's kill of its own process, over a fake process list and a fake killer: what it
//! ends, each refusal, and the activity line of each.

use super::harness::{Harness, MIN, Setup, Spec, T0, proc};
use argus_procs::{ErrorCode, Level, ProcRow, SelfKillRequest};
use serde_json::json;

const MINE: &str = "node x:/alpha/node_modules/vitest/vitest.mjs run";

fn harness(self_pid: u32, tray: Option<u32>) -> Harness {
    let h = Harness::new(Setup {
        self_pid: Some(self_pid),
        tray,
        reaper: Some(json!({ "enabled": false })),
        ..Setup::default()
    });
    h.fake.set_folders(&[
        ("alpha", &["X:\\alpha", "X:\\wt\\alpha-1", "D:\\ci\\alpha"]),
        ("beta", &["X:\\beta"]),
    ]);
    h
}

fn mine(pid: u32, parent: u32, cwd: &'static str) -> ProcRow {
    proc(Spec {
        ppid: parent,
        cmd: MINE,
        cwd: Some(cwd),
        start: T0 - 5 * MIN,
        ..Spec::new(pid)
    })
}

fn ask(pid: u32, tree: bool) -> SelfKillRequest {
    SelfKillRequest {
        project: "alpha".to_string(),
        thread: "alpha-lead".to_string(),
        pid,
        tree,
        reason: Some("its run hung".to_string()),
    }
}

fn refused(h: &Harness, request: &SelfKillRequest) -> (ErrorCode, String) {
    let error = h.service.self_kill(request).unwrap_err();
    (error.code(), error.message().to_string())
}

fn kill_lines(h: &Harness) -> Vec<(Level, String)> {
    h.fake
        .acts()
        .into_iter()
        .filter(|a| a.kind == "proc-kill")
        .map(|a| (a.level, a.text))
        .collect()
}

#[test]
fn a_thread_ends_its_own_process_by_pid_with_one_activity_line() {
    let h = harness(1, None);
    h.at(T0, vec![mine(5001, 100, "X:\\alpha\\app")]);
    let out = h.service.self_kill(&ask(5001, false)).unwrap();
    assert!(out.killed && out.tree.is_empty());
    assert_eq!(out.exe, "C:\\nvm4w\\nodejs\\node.exe");
    assert_eq!(h.fake.trees(), [(5001, false, vec![])]);
    assert_eq!(
        kill_lines(&h),
        [(
            Level::Info,
            "thread alpha-lead of alpha killed C:\\nvm4w\\nodejs\\node.exe 5001: its run hung"
                .to_string()
        )]
    );
}

#[test]
fn a_worktree_and_the_ci_root_are_the_threads_own_folders_too() {
    let h = harness(1, None);
    h.at(
        T0,
        vec![
            mine(5001, 100, "X:\\wt\\alpha-1"),
            mine(5002, 100, "D:\\ci\\alpha\\target"),
        ],
    );
    assert!(h.service.self_kill(&ask(5001, false)).unwrap().killed);
    assert!(h.service.self_kill(&ask(5002, false)).unwrap().killed);
    assert_eq!(h.fake.killed(), [5001, 5002]);
}

#[test]
fn with_tree_every_process_of_the_tree_is_checked_and_ended() {
    let h = harness(1, None);
    h.at(
        T0,
        vec![
            mine(5001, 100, "X:\\alpha"),
            mine(5002, 5001, "X:\\alpha"),
            mine(5003, 5002, "X:\\wt\\alpha-1"),
        ],
    );
    let out = h.service.self_kill(&ask(5001, true)).unwrap();
    assert_eq!(out.tree, [5002, 5003]);
    assert_eq!(h.fake.trees(), [(5001, true, vec![5002, 5003])]);
    let (_, line) = &kill_lines(&h)[0];
    assert!(line.contains("killed C:\\nvm4w\\nodejs\\node.exe 5001 and its tree (2 more)"));
}

#[test]
fn a_tree_that_holds_another_projects_process_is_refused_whole() {
    let h = harness(1, None);
    h.at(
        T0,
        vec![mine(5001, 100, "X:\\alpha"), mine(5002, 5001, "X:\\beta")],
    );
    let (code, text) = refused(&h, &ask(5001, true));
    assert_eq!(code, ErrorCode::Forbidden);
    assert_eq!(
        text,
        "the tree of node 5001 holds node 5002, which belongs to project beta, not alpha: only that project's threads may kill it"
    );
    assert_eq!(h.fake.killed(), Vec::<u32>::new());
    assert!(h.service.self_kill(&ask(5001, false)).unwrap().killed);
}

#[test]
fn the_folder_is_read_from_the_sampler_when_the_sample_has_none() {
    let h = harness(1, None);
    let bare = proc(Spec {
        cmd: "node build.js",
        ..Spec::new(5001)
    });
    h.at(T0, vec![bare]);
    let (_, text) = refused(&h, &ask(5001, false));
    assert!(text.starts_with("node 5001 is not inside a folder of project alpha"));
    h.fake.set_cwd(5001, "X:\\alpha\\tools");
    assert!(h.service.self_kill(&ask(5001, false)).unwrap().killed);
}

fn named(pid: u32, name: &'static str, cmd: &'static str, path: &'static str) -> ProcRow {
    proc(Spec {
        name,
        cmd,
        path,
        cwd: Some("X:\\alpha"),
        ..Spec::new(pid)
    })
}

fn protected_rows() -> Vec<ProcRow> {
    let named_rows = [
        (
            5001,
            "claude.exe",
            "claude --resume",
            "C:\\claude\\claude.exe",
        ),
        (
            5002,
            "node.exe",
            "node C:/npm/@anthropic-ai/claude-code/cli.js",
            "C:\\nvm4w\\nodejs\\node.exe",
        ),
        (
            77,
            "argus.exe",
            "X:\\argus\\argus.exe serve",
            "X:\\argus\\argus.exe",
        ),
        (
            78,
            "argus.exe",
            "X:\\argus\\argus.exe tray",
            "X:\\argus\\argus.exe",
        ),
        (
            5003,
            "argus.exe",
            "C:\\argus\\argus.exe mcp alpha",
            "C:\\argus\\argus.exe",
        ),
        (
            5004,
            "svchost.exe",
            "svchost -k netsvcs",
            "C:\\Windows\\System32\\svchost.exe",
        ),
        (
            5005,
            "explorer.exe",
            "explorer",
            "C:\\Windows\\explorer.exe",
        ),
        (
            5006,
            "conhost.exe",
            "conhost 0x4",
            "C:\\Windows\\System32\\conhost.exe",
        ),
    ];
    let mut rows: Vec<ProcRow> = named_rows
        .into_iter()
        .map(|(pid, name, cmd, path)| named(pid, name, cmd, path))
        .collect();
    rows.push(mine(5007, 100, "X:\\beta\\app"));
    rows.push(proc(Spec {
        cmd: "node c:/tools/run.js",
        cwd: Some("C:\\Users\\me"),
        ..Spec::new(5008)
    }));
    rows.push(ProcRow {
        cmd: String::new(),
        path: None,
        cwd: None,
        ..named(5009, "x.exe", "", "")
    });
    rows
}

fn protected_cases() -> Vec<(u32, String)> {
    let claude = "is a Claude session process: it is never killed from here";
    let dashboard = "is a process of the dashboard itself (argus serve, the tray or an MCP bridge): it is never killed from here";
    let system = "is a system process: it is never killed from here";
    vec![
        (5001, format!("claude 5001 {claude}")),
        (5002, format!("node 5002 {claude}")),
        (77, format!("argus 77 {dashboard}")),
        (78, format!("argus 78 {dashboard}")),
        (5003, format!("argus 5003 {dashboard}")),
        (5004, format!("svchost 5004 {system}")),
        (5005, format!("explorer 5005 {system}")),
        (5006, format!("conhost 5006 {system}")),
        (
            5007,
            "node 5007 belongs to project beta, not alpha: only that project's threads may kill it"
                .to_string(),
        ),
        (
            5008,
            "node 5008 is not inside a folder of project alpha (its root, its worktrees or its ci root)"
                .to_string(),
        ),
        (
            5009,
            "x 5009 has details that cannot be read (no folder, executable or command line): it is never killed from here"
                .to_string(),
        ),
    ]
}

#[test]
fn every_protected_or_foreign_process_is_refused_with_its_reason() {
    let h = harness(77, Some(78));
    h.at(T0, protected_rows());
    let cases = protected_cases();
    for (pid, text) in &cases {
        assert_eq!(
            refused(&h, &ask(*pid, false)),
            (ErrorCode::Forbidden, text.clone())
        );
    }
    assert_eq!(h.fake.killed(), Vec::<u32>::new());
    let lines = kill_lines(&h);
    assert_eq!(lines.len(), cases.len());
    assert!(lines.iter().all(|(level, _)| *level == Level::Warn));
    assert_eq!(
        lines[0].1,
        format!(
            "refused thread alpha-lead of alpha the kill of 5001: claude 5001 {claude} (reason given: its run hung)"
        )
    );
}

#[test]
fn a_pid_that_is_not_running_or_zero_is_refused() {
    let h = harness(1, None);
    h.at(T0, vec![mine(5001, 100, "X:\\alpha")]);
    assert_eq!(
        refused(&h, &ask(9999, false)),
        (
            ErrorCode::NotFound,
            "process 9999 is not running, or its details cannot be read: nothing was killed"
                .to_string()
        )
    );
    assert_eq!(refused(&h, &ask(0, false)).0, ErrorCode::BadRequest);
    assert_eq!(h.fake.killed(), Vec::<u32>::new());
}

#[test]
fn a_failed_kill_answers_the_error_and_writes_an_error_line() {
    let h = harness(1, None);
    h.at(T0, vec![mine(5001, 100, "X:\\alpha")]);
    h.fake.fail_kills();
    let out = h.service.self_kill(&ask(5001, false)).unwrap();
    assert!(!out.killed);
    assert_eq!(out.error.as_deref(), Some("access is denied"));
    let (level, line) = &kill_lines(&h)[0];
    assert_eq!(*level, Level::Error);
    assert!(line.contains("could not kill C:\\nvm4w\\nodejs\\node.exe 5001: access is denied"));
}
