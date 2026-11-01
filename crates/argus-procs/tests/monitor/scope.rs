//! The scope rule a thread kill and the reaper share: whose process this is, and each refusal.

use super::harness::{Spec, proc};
use argus_procs::{Guard, GuardCtx, ProcRow, ProjectFolders, Scope, refusal};

fn scope() -> Scope {
    let project = |key: &str, folders: &[&str]| ProjectFolders {
        key: key.to_string(),
        folders: folders.iter().map(ToString::to_string).collect(),
    };
    Scope::new(&[
        project("alpha", &["X:\\alpha", "X:\\wt\\alpha-1\\", "D:/ci/alpha"]),
        project("beta", &["X:\\beta", "X:\\alpha\\vendor\\beta"]),
        project("broken", &["X:", "", "/"]),
    ])
}

fn at(cwd: Option<&'static str>, cmd: &'static str) -> ProcRow {
    proc(Spec {
        cmd,
        cwd,
        ..Spec::new(500)
    })
}

fn why(row: &ProcRow) -> Option<String> {
    refusal(row, None, &scope(), "alpha")
}

#[test]
fn a_process_in_the_root_a_worktree_or_the_ci_root_is_the_threads_own() {
    for cwd in ["X:\\alpha\\app", "x:/wt/alpha-1", "D:\\ci\\alpha\\target"] {
        assert_eq!(why(&at(Some(cwd), "node run.js")), None, "{cwd}");
    }
    let by_cmd = at(
        None,
        "node \"X:\\alpha\\node_modules\\vitest\\vitest.mjs\" run",
    );
    assert_eq!(why(&by_cmd), None);
    let by_exe = ProcRow {
        path: Some("X:\\alpha\\target\\debug\\deps\\a-1.exe".to_string()),
        cmd: "a-1.exe --test-threads 1".to_string(),
        ..at(None, "")
    };
    assert_eq!(why(&by_exe), None);
    assert_eq!(scope().owner(&by_exe).as_deref(), Some("alpha"));
}

#[test]
fn a_process_of_another_project_is_refused_with_its_name() {
    let other = at(Some("X:\\beta"), "node build.js");
    assert_eq!(
        why(&other).as_deref(),
        Some("belongs to project beta, not alpha: only that project's threads may kill it")
    );
    let nested = at(Some("X:\\alpha\\vendor\\beta\\src"), "cargo build");
    assert!(why(&nested).unwrap().starts_with("belongs to project beta"));
    let both = at(Some("X:\\alpha"), "grep -r foo X:/beta/src");
    assert!(why(&both).unwrap().starts_with("belongs to project beta"));
}

#[test]
fn a_process_outside_every_project_folder_is_refused() {
    let outside = at(Some("C:\\Users\\me"), "find / -name tessera");
    assert_eq!(
        why(&outside).as_deref(),
        Some("is not inside a folder of project alpha (its root, its worktrees or its ci root)")
    );
    let lookalike = at(Some("X:\\alphabet"), "node x:/alphabet/run.js");
    assert!(
        why(&lookalike)
            .unwrap()
            .starts_with("is not inside a folder")
    );
}

#[test]
fn a_drive_or_an_empty_folder_never_counts_as_a_project_folder() {
    let anywhere = at(Some("X:\\other"), "node x:/other/run.js");
    assert_eq!(scope().owners(&anywhere).len(), 0);
}

#[test]
fn a_process_whose_details_cannot_be_read_is_refused() {
    let blank = ProcRow {
        path: None,
        cmd: "  ".to_string(),
        cwd: None,
        ..at(None, "")
    };
    assert_eq!(
        why(&blank).as_deref(),
        Some(
            "has details that cannot be read (no folder, executable or command line): it is never killed from here"
        )
    );
}

#[test]
fn each_guard_is_its_own_refusal() {
    let mine = at(Some("X:\\alpha"), "node run.js");
    let say = |guard| refusal(&mine, Some(guard), &scope(), "alpha").unwrap();
    assert_eq!(
        say(Guard::Claude),
        "is a Claude session process: it is never killed from here"
    );
    for guard in [Guard::Dashboard, Guard::Tray] {
        assert!(say(guard).starts_with("is a process of the dashboard itself (argus serve"));
    }
    for guard in [Guard::System, Guard::Windows] {
        assert_eq!(
            say(guard),
            "is a system process: it is never killed from here"
        );
    }
}

#[test]
fn the_guard_knows_the_rust_dashboard_by_its_command_line() {
    let guard = GuardCtx::new(&[], 1, None, Some("C:\\Windows"));
    let named = |name: &str, cmd: &str, path: &str| ProcRow {
        name: name.to_string(),
        cmd: cmd.to_string(),
        path: Some(path.to_string()),
        ..at(Some("X:\\alpha"), "")
    };
    let serve = named(
        "argus.exe",
        "X:\\alpha\\target\\debug\\argus.exe serve --port 7300",
        "X:\\alpha\\target\\debug\\argus.exe",
    );
    let bridge = named(
        "argus.exe",
        "\"C:\\argus\\argus.exe\" mcp alpha",
        "C:\\argus\\argus.exe",
    );
    let tray = named("argus", "/usr/local/bin/argus tray", "/usr/local/bin/argus");
    for row in [&serve, &bridge, &tray] {
        assert_eq!(
            guard.guard_of(row, true),
            Some(Guard::Dashboard),
            "{}",
            row.cmd
        );
    }
    let build = named("argus.exe", "argus.exe --version", "X:\\alpha\\argus.exe");
    assert_eq!(guard.guard_of(&build, true), None);
    let session = named("claude.exe", "claude --resume", "C:\\claude\\claude.exe");
    assert_eq!(guard.guard_of(&session, true), Some(Guard::Claude));
    let host = named(
        "svchost.exe",
        "svchost -k x",
        "C:\\Windows\\System32\\svchost.exe",
    );
    assert_eq!(guard.guard_of(&host, true), Some(Guard::System));
    let shell = named("cmd.exe", "cmd /c x", "C:\\Windows\\System32\\cmd.exe");
    assert_eq!(guard.guard_of(&shell, true), Some(Guard::Windows));
}
