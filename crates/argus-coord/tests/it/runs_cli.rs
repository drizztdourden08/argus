//! K2: the board-only verbs of coord run: define, defs, undefine and their refusals.

// test-tag: critical

use std::path::Path;

use crate::common::{Home, Ran, coord};
use crate::remote::{ok, read_board_at, refused};
use crate::runs_support::{AS, run_argv, seed};
use serde_json::{Value, json};

fn same_folder(a: &Value, b: &Path) -> bool {
    let flat = |s: &str| s.replace('\\', "/").to_lowercase();
    a.as_str()
        .is_some_and(|a| flat(a) == flat(&b.to_string_lossy()))
}

/// A home with the board `alpha`, and `coord run <words> --project alpha <extra>` in it.
struct Local {
    home: Home,
}

impl Local {
    fn new() -> Local {
        let home = Home::new();
        seed(&home.coord_home(), home.path());
        Local { home }
    }

    fn go(&self, words: &[&str], extra: &[&str]) -> Ran {
        coord(&run_argv(words, extra), self.home.env(), self.home.path())
    }

    fn defs(&self) -> Value {
        read_board_at(&self.home.coord_home(), "alpha")["runDefs"].clone()
    }
}

#[test]
fn define_records_the_launch_and_a_second_define_updates_it() {
    let local = Local::new();
    let build = [
        "define",
        "build",
        "--cmd",
        "npm run build",
        "--kind",
        "build",
    ];
    let first = local.go(
        &build,
        &[
            "--env",
            "API_TOKEN=s3cret",
            "--env",
            "MODE=prod",
            "--as",
            "lead",
        ],
    );
    let out = ok(&first);
    assert!(
        out.contains("defined build: build") && out.contains("timeout 1800s"),
        "{out}"
    );
    let def = local.defs()["build"].clone();
    let seen = json!([
        def["cmd"],
        def["kind"],
        def["hand"],
        def["visible"],
        def["timeoutSec"],
        def["parallel"],
        def["rev"],
        def["definedBy"]
    ]);
    assert_eq!(
        seen,
        json!(["npm run build", "build", false, false, 1800, 1, 1, "lead"])
    );
    assert_eq!(def["env"], json!({ "API_TOKEN": "s3cret", "MODE": "prod" }));
    assert!(
        same_folder(&def["cwd"], local.home.path()),
        "cwd defaults to the caller cwd: {}",
        def["cwd"]
    );
}

#[test]
fn a_second_define_updates_the_launch_and_a_hand_launch_may_be_visible() {
    let local = Local::new();
    local.go(
        &[
            "define",
            "build",
            "--cmd",
            "npm run build",
            "--kind",
            "build",
        ],
        &AS,
    );
    let again = local.go(
        &[
            "define",
            "build",
            "--cmd",
            "npm run build:all",
            "--kind",
            "build",
            "--timeout",
            "0",
        ],
        &AS,
    );
    assert!(ok(&again).contains("updated build"));
    assert_eq!(local.defs()["build"]["rev"], 2);
    let hand = local.go(
        &[
            "define",
            "app",
            "--cmd",
            "start /wait app.exe",
            "--hand",
            "--visible",
            "--parallel",
            "1",
        ],
        &AS,
    );
    assert!(ok(&hand).contains("hand (visible)"));
}

#[test]
fn defs_lists_the_launches_without_their_env_values_and_undefine_removes_one() {
    let local = Local::new();
    local.go(
        &[
            "define",
            "build",
            "--cmd",
            "npm run build",
            "--env",
            "API_TOKEN=s3cret",
        ],
        &AS,
    );
    local.go(
        &[
            "define",
            "app",
            "--cmd",
            "start /wait app.exe",
            "--hand",
            "--visible",
        ],
        &AS,
    );
    let parsed: Value =
        serde_json::from_str(ok(&local.go(&["defs", "--json"], &[]))).expect("json");
    let names: Vec<&Value> = parsed["defs"]
        .as_array()
        .expect("defs")
        .iter()
        .map(|d| &d["name"])
        .collect();
    assert_eq!(names, [&json!("app"), &json!("build")]);
    assert!(
        !parsed.to_string().contains("s3cret"),
        "defs never prints env values"
    );
    let text = ok(&local.go(&["defs"], &[]))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(text.contains("app task hand,visible"), "{text}");
    ok(&local.go(&["undefine", "build"], &AS));
    refused(
        &local.go(&["undefine", "build"], &AS),
        1,
        "no launch \"build\"",
    );
    assert_eq!(
        local
            .defs()
            .as_object()
            .map(|m| m.keys().cloned().collect::<Vec<_>>()),
        Some(vec!["app".to_owned()])
    );
}

#[test]
fn define_and_undefine_refuse_what_is_not_the_leads_or_not_valid() {
    let local = Local::new();
    let worker = ["--as", "worker"];
    let refusals: [(&[&str], &[&str], &str); 9] = [
        (
            &["define", "evil", "--cmd", "calc"],
            &worker,
            "only the lead",
        ),
        (
            &["define", "x", "--cmd", "echo", "--visible"],
            &AS,
            "--visible only goes with --hand",
        ),
        (&["define", "Bad Name", "--cmd", "echo"], &AS, "lowercase"),
        (&["define", "x"], &AS, "--cmd"),
        (
            &["define", "x", "--cmd", "echo", "--cwd", "nope"],
            &AS,
            "not a folder",
        ),
        (
            &["define", "x", "--cmd", "echo", "--kind", "weird"],
            &AS,
            "--kind",
        ),
        (
            &["define", "x", "--cmd", "echo", "--env", "NOEQUALS"],
            &AS,
            "KEY=VALUE",
        ),
        (
            &["define", "x", "--cmd", "echo", "--timeout", "soon"],
            &AS,
            "--timeout",
        ),
        (&["undefine", "build"], &worker, "only the lead"),
    ];
    for (words, extra, text) in refusals {
        refused(&local.go(words, extra), 1, text);
    }
    refused(&local.go(&["nonsense"], &[]), 1, "run verb must be one of");
}
