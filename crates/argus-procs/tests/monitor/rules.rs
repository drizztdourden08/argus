//! Rule matching and the rules file.

use super::harness::{Harness, Setup, T0};
use argus_procs::{
    CompiledRule, ProcRow, compile_rules, default_rules, match_rule, normalize_rule,
};
use serde_json::{Value, json};

fn compiled(rules: &[Value]) -> Vec<CompiledRule> {
    let rules: Vec<_> = rules.iter().map(|r| normalize_rule(r).unwrap()).collect();
    compile_rules(&rules)
}

fn row(name: &str, cmd: &str, path: Option<&str>) -> ProcRow {
    ProcRow {
        pid: 1,
        ppid: 0,
        name: name.to_string(),
        cmd: cmd.to_string(),
        path: path.map(ToString::to_string),
        mem_bytes: 0,
        start_ms: 0,
        cpu_ms: 0,
        responding: None,
    }
}

fn take(rules: &[CompiledRule], name: &str, cmd: &str, path: Option<&str>) -> Option<String> {
    match_rule(rules, &row(name, cmd, path)).map(|r| r.name.clone())
}

#[test]
fn the_first_rule_that_takes_a_process_wins_ignore_moves_on_and_paths_use_forward_slashes() {
    let c = compiled(&[
        json!({"name": "vite", "match": {"name": "^node([.]exe)?$", "cmd": "vite"}, "ignore": ["dashboard[.]mjs"]}),
        json!({"name": "proj", "match": {"path": "^x:/alpha/"}}),
        json!({"name": "rest", "match": {}}),
    ]);
    let is = |name, cmd, path, want: &str| {
        assert_eq!(
            take(&c, name, cmd, path).as_deref(),
            Some(want),
            "{name} {cmd}"
        );
    };
    is("node.exe", "node x:/a/vite.js", None, "vite");
    is("Node.EXE", "node x:/a/vite.js", None, "vite");
    is(
        "node.exe",
        "node x:/vite/bin/dashboard.mjs start",
        Some("X:\\alpha\\node.exe"),
        "proj",
    );
    is("game.exe", "", Some("X:\\alpha\\build\\game.exe"), "proj");
    is("other.exe", "", Some("C:\\x.exe"), "rest");
    let only_node = compiled(&[json!({"name": "n", "match": {"name": "^node"}})]);
    assert_eq!(take(&only_node, "python.exe", "", None), None);
}

type Case = (
    &'static str,
    &'static str,
    Option<&'static str>,
    &'static str,
);

fn assert_defaults_take(cases: &[Case]) {
    let c = compile_rules(&default_rules());
    for (name, cmd, path, want) in cases {
        assert_eq!(
            take(&c, name, cmd, *path).as_deref(),
            Some(*want),
            "{name} {cmd}"
        );
    }
}

#[test]
fn the_claude_and_node_default_rules_take_what_they_should() {
    assert_defaults_take(&[
        ("claude.exe", "claude.exe --type=renderer", None, "claude"),
        (
            "node.exe",
            "node C:/Users/x/AppData/npm/node_modules/@anthropic-ai/claude-code/cli.js",
            None,
            "claude-code-node",
        ),
        (
            "electron.exe",
            "\"X:\\archipelia\\node_modules\\electron\\dist\\electron.exe\" .",
            None,
            "electron-dev",
        ),
        (
            "node.exe",
            "node X:\\rotp\\node_modules\\vite\\bin\\vite.js",
            None,
            "node-dev-server",
        ),
        (
            "node.exe",
            "node X:\\rotp\\node_modules\\.pnpm\\knip@5\\node_modules\\knip\\bin\\knip.js",
            None,
            "node-other",
        ),
        (
            "node.exe",
            "node C:\\nvm4w\\nodejs\\node_modules\\pnpm\\bin\\pnpm.cjs dev",
            None,
            "node-dev-server",
        ),
    ]);
}

#[test]
fn the_dashboard_is_ignored_by_every_node_rule_and_the_other_default_rules_take_theirs() {
    assert_defaults_take(&[
        (
            "node.exe",
            "node X:/ai-config/dashboard/bin/dashboard.mjs start",
            None,
            "catch-all",
        ),
        (
            "node.exe",
            "node x:/ai-config/shared/tools/coord/coord.mjs dashboard wake-stream --project a",
            None,
            "catch-all",
        ),
        ("python.exe", "python -m uvicorn app:main", None, "python"),
        ("Godot_v4.3.exe", "godot --editor", None, "game-engine"),
        (
            "powershell.exe",
            "powershell -File X:/ai-config/dashboard/src/tray/tray.ps1",
            None,
            "catch-all",
        ),
        ("cmd.exe", "cmd /c npm run dev", None, "shell-leftovers"),
        (
            "game.exe",
            "",
            Some("X:\\Bxel\\dist\\game.exe"),
            "project-build",
        ),
        ("chrome.exe", "chrome", None, "catch-all"),
    ]);
}

#[test]
fn the_rules_file_is_written_through_the_atomic_writer_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut book = argus_procs::RuleBook::new(dir.path().join("nested").join("process-rules.json"));
    assert!(book.load().is_none());
    book.set_rule(&json!({"name": "mine", "match": {"name": "^mine$"}}))
        .unwrap();
    let names: Vec<_> = std::fs::read_dir(dir.path().join("nested"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["process-rules.json"]);
}

#[test]
fn the_rules_file_is_seeded_changed_and_reloaded_and_a_bad_file_keeps_the_rules() {
    let h = Harness::new(Setup::default());
    h.at(T0, vec![]);
    let file: Value = serde_json::from_str(
        &std::fs::read_to_string(h.dir.path().join("process-rules.json")).unwrap(),
    )
    .unwrap();
    let comment = file["_comment"].as_array().unwrap();
    assert!(
        comment
            .iter()
            .any(|l| l.as_str().unwrap().starts_with("catch-all:"))
    );
    let seeded: Vec<&str> = file["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    let defaults: Vec<String> = default_rules().into_iter().map(|r| r.name).collect();
    assert_eq!(seeded, defaults);

    let added = h.service.set_rule(&json!({"name": "webpack-x", "match": {"cmd": "webpack"}, "maxMemMB": 900, "killable": true})).unwrap();
    assert!(added.created);
    let names = h.rule_names();
    assert_eq!(names[names.len() - 2], "webpack-x");
    assert_eq!(names[names.len() - 1], "catch-all");

    let changed = h.service.set_rule(&json!({"name": "webpack-x", "maxMemMB": null, "idleMin": 30, "match": {"cmd": null, "name": "^node"}})).unwrap();
    assert!(!changed.created);
    assert_eq!(
        serde_json::to_value(&changed.rule).unwrap(),
        json!({"name": "webpack-x", "match": {"name": "^node"}, "killable": true, "idleMin": 30})
    );
    let bad = h
        .service
        .set_rule(&json!({"name": "webpack-x", "match": {"cmd": "("}}));
    assert!(bad.unwrap_err().message().contains("not a valid regex"));
    assert!(h.service.remove_rule("webpack-x").unwrap());
    assert!(!h.service.remove_rule("webpack-x").unwrap());

    let path = h.dir.path().join("process-rules.json");
    std::fs::write(
        &path,
        json!({"rules": [{"name": "only", "match": {"name": "^python"}}]}).to_string(),
    )
    .unwrap();
    h.at(T0 + 1000, vec![]);
    assert_eq!(h.rule_names(), ["only"]);

    std::fs::write(&path, "{ not json").unwrap();
    let meta = h.at(T0 + 2000, vec![]);
    assert!(meta.rules_error.unwrap().contains("not valid JSON"));
    assert_eq!(h.rule_names(), ["only"]);
}
