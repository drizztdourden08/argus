//! `coord pr open | list | merge | close` and `coord check [<check>] ...`: the local pull requests
//! on a temp `COORD_HOME`, the merge with a fake `brock` first on the PATH (never the real one),
//! and a check run against an Argus test server whose `checks.command` is a fake `brock` script.
//! Never the real `~/.coord`, never port 47615, never a real repository.

mod common;
mod remote;

use std::fs;
use std::path::{Path, PathBuf};

use argus_coord::Env;
use common::{Home, Ran, coord};
use remote::{Rig, alpha_with_worker, env_plus, lines, ok, read_board_at, refused};
use serde_json::{Value, json};

const AS: [&str; 2] = ["--as", "coordinator"];
const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const MERGE: &str = "fedcba9876543210fedcba9876543210fedcba98";

/// A home with the board `alpha`, and `coord <words> --project alpha --as coordinator` in it.
struct Local {
    home: Home,
}

impl Local {
    fn new() -> Local {
        let home = Home::new();
        let mut board = alpha_with_worker(home.path());
        board["items"] = json!({ "I-1": { "id": "I-1", "title": "An item", "thread": "worker", "status": "todo" } });
        home.write_board("alpha", &board);
        Local { home }
    }

    fn go(&self, words: &[&str]) -> Ran {
        self.go_in(words, self.home.env())
    }

    fn go_in(&self, words: &[&str], env: Env) -> Ran {
        let argv: Vec<&str> = words
            .iter()
            .chain(&["--project", "alpha"])
            .chain(&AS)
            .copied()
            .collect();
        coord(&argv, env, self.home.path())
    }

    fn board(&self) -> Value {
        read_board_at(&self.home.coord_home(), "alpha")
    }

    fn open(&self, title: &str, head: &str) -> Ran {
        self.go(&["pr", "open", title, "--head", head, "--commit", SHA])
    }
}

#[test]
fn open_records_the_pr_and_list_shows_it_as_a_table_and_as_json() {
    let local = Local::new();
    let said = local.go(&[
        "pr", "open", "Add x", "--head", "feat/x", "--commit", SHA, "--item", "I-1", "--body",
        "why",
    ]);
    assert_eq!(ok(&said).trim(), "PR-1 opened: feat/x into main at 0123456");
    let pr = &local.board()["prs"]["PR-1"];
    assert_eq!(
        (pr["status"].as_str(), pr["base"].as_str()),
        (Some("open"), Some("main"))
    );
    assert_eq!(
        (pr["headCommit"].as_str(), pr["item"].as_str()),
        (Some(SHA), Some("I-1"))
    );
    assert_eq!(pr["replay"]["state"], "pending");
    let table = local.go(&["pr", "list"]);
    let table = ok(&table);
    let shown = lines(table);
    assert!(shown[0].starts_with("pr"), "{table}");
    assert!(
        shown[1].contains("PR-1") && shown[1].contains("feat/x -> main"),
        "{table}"
    );
    assert!(
        shown[1].contains("0123456") && shown[1].ends_with("Add x"),
        "{table}"
    );
    let json = local.go(&["pr", "list", "--json"]);
    let list: Value = serde_json::from_str(ok(&json)).expect("json");
    let first = &list[0];
    assert_eq!(first["id"], "PR-1");
    for field in ["head", "base", "headCommit", "status", "checks"] {
        assert!(first.get(field).is_some(), "{field} missing in {first}");
    }
    assert_eq!(first["checks"], json!([]));
    assert_eq!(
        ok(&local.go(&["pr", "list", "--status", "merged"])).trim(),
        "no pull requests."
    );
}

#[test]
fn open_refuses_what_is_missing_or_wrong() {
    let local = Local::new();
    refused(
        &local.go(&["pr", "open", "--head", "feat/x"]),
        1,
        "give the title",
    );
    refused(
        &local.go(&["pr", "open", "T"]),
        1,
        "--head <branch> is required.",
    );
    refused(
        &local.open("T", "main"),
        1,
        "head and base are the same branch.",
    );
    refused(
        &local.open("T", "bad name"),
        1,
        "head must be a branch name",
    );
    ok(&local.open("T", "feat/x"));
    refused(&local.open("T again", "feat/x"), 1, "PR-1 is already open");
    refused(
        &local.go(&["pr", "frobnicate"]),
        1,
        "pr verb must be one of open, list, merge, close.",
    );
    refused(
        &local.go(&["pr", "close", "7"]),
        1,
        "give the pull request id",
    );
    refused(
        &local.go(&["pr", "close", "PR-7"]),
        1,
        "no pull request \"PR-7\".",
    );
}

#[test]
fn close_ends_a_pr_and_a_closed_pr_neither_closes_nor_merges() {
    let local = Local::new();
    ok(&local.open("T", "feat/x"));
    assert_eq!(
        ok(&local.go(&["pr", "close", "PR-1"])).trim(),
        "PR-1 closed"
    );
    assert_eq!(local.board()["prs"]["PR-1"]["status"], "closed");
    refused(&local.go(&["pr", "close", "PR-1"]), 1, "PR-1 is closed");
    refused(
        &local.go(&["pr", "merge", "PR-1"]),
        1,
        "only an open pull request merges",
    );
}

/// What the fake `brock` does.
enum Fake {
    /// Prints the merge commit.
    Merges,
    /// Says why on stderr and exits 1.
    Refuses,
    /// Exits 0 and prints something that is not a commit.
    Babbles,
}

#[cfg(windows)]
fn write_fake(bin: &Path, record: &Path, fake: &Fake) {
    let said = match fake {
        Fake::Merges => format!("echo {MERGE}\r\n"),
        Fake::Refuses => "echo required step lint is not passing 1>&2\r\nexit /b 1\r\n".to_owned(),
        Fake::Babbles => "echo merged, trust me\r\n".to_owned(),
    };
    let args = format!(
        "@echo off\r\necho %* > \"{}\"\r\ncd > \"{}.cwd\"\r\n",
        record.display(),
        record.display()
    );
    fs::write(bin.join("brock.cmd"), format!("{args}{said}")).expect("fake brock");
}

#[cfg(not(windows))]
fn write_fake(bin: &Path, record: &Path, fake: &Fake) {
    use std::os::unix::fs::PermissionsExt;

    let said = match fake {
        Fake::Merges => format!("echo {MERGE}\n"),
        Fake::Refuses => "echo 'required step lint is not passing' >&2\nexit 1\n".to_owned(),
        Fake::Babbles => "echo 'merged, trust me'\n".to_owned(),
    };
    let args = format!(
        "#!/bin/sh\necho \"$@\" > '{0}'\npwd > '{0}.cwd'\n",
        record.display()
    );
    let file = bin.join("brock");
    fs::write(&file, format!("{args}{said}")).expect("fake brock");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).expect("executable");
}

/// The environment of a call with the fake `brock` folder first on the PATH.
fn with_fake(local: &Local, fake: &Fake) -> (Env, PathBuf) {
    let bin = local.home.folder("fake-bin");
    let record = bin.join("args.txt");
    write_fake(&bin, &record, fake);
    let mut dirs = vec![bin];
    dirs.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(dirs)
        .expect("path")
        .to_string_lossy()
        .into_owned();
    (env_plus(&local.home.env(), &[("PATH", &path)]), record)
}

fn merge(local: &Local, fake: &Fake) -> (Ran, PathBuf) {
    let (env, record) = with_fake(local, fake);
    let root = local.home.folder("checkout");
    let root = root.to_string_lossy().into_owned();
    let ran = local.go_in(&["pr", "merge", "PR-1", "--root", &root], env);
    (ran, record)
}

#[test]
fn merge_runs_brock_in_the_project_root_and_records_the_commit_it_printed() {
    let local = Local::new();
    ok(&local.open("T", "feat/x"));
    let (ran, record) = merge(&local, &Fake::Merges);
    assert_eq!(ok(&ran).trim(), "PR-1 merged into main at fedcba9");
    let pr = &local.board()["prs"]["PR-1"];
    assert_eq!(
        (pr["status"].as_str(), pr["mergeCommit"].as_str()),
        (Some("merged"), Some(MERGE))
    );
    assert_eq!(pr["mergedBy"], "coordinator");
    assert!(pr["mergedAt"].is_string());
    let args = fs::read_to_string(&record).expect("args");
    assert_eq!(args.trim().trim_matches('"'), "pr merge --local PR-1");
    let cwd = fs::read_to_string(format!("{}.cwd", record.display())).expect("cwd");
    let flat = |s: &str| s.trim().replace('\\', "/").to_lowercase();
    assert!(flat(&cwd).ends_with("checkout"), "brock ran in {cwd}");
}

#[test]
fn merge_runs_brock_in_the_integration_checkout_before_the_project_root() {
    let local = Local::new();
    ok(&local.open("T", "feat/x"));
    let ci = local.home.folder("integration");
    let dir = ci.to_string_lossy().replace('\\', "/");
    let roots = json!({ "mode": "local", "ciRoots": { "alpha": dir } });
    let file = local.home.coord_home().join("remote.json");
    fs::write(file, roots.to_string()).expect("remote.json");
    let (env, record) = with_fake(&local, &Fake::Merges);
    ok(&local.go_in(&["pr", "merge", "PR-1"], env));
    let cwd = fs::read_to_string(format!("{}.cwd", record.display())).expect("cwd");
    let flat = |s: &str| s.trim().replace('\\', "/").to_lowercase();
    assert!(flat(&cwd).ends_with("integration"), "brock ran in {cwd}");
}

#[test]
fn merge_prints_the_reason_of_brock_and_leaves_the_pr_open() {
    let local = Local::new();
    ok(&local.open("T", "feat/x"));
    let (ran, _) = merge(&local, &Fake::Refuses);
    refused(&ran, 1, "coord: required step lint is not passing");
    assert_eq!(local.board()["prs"]["PR-1"]["status"], "open");
    let (babble, _) = merge(&local, &Fake::Babbles);
    refused(&babble, 1, "did not print a merge commit");
    assert_eq!(local.board()["prs"]["PR-1"]["status"], "open");
    refused(
        &local.go(&["pr", "merge", "PR-9"]),
        1,
        "no pull request \"PR-9\".",
    );
}

fn seed(coord_home: &Path, folder: &Path) {
    argus_testkit::write_board(coord_home, "alpha", &alpha_with_worker(folder));
}

/// Writes the fake brock where `command` points: it records a run of project alpha under the
/// coord home and prints the path of the record.
fn write_fake_script(coord_home: &Path, script: &Path) {
    let ci = json!(coord_home.join("ci").join("alpha").display().to_string());
    let source = format!(
        r#"
const fs = require("fs"), path = require("path");
const args = process.argv.slice(2);
const dir = {ci};
fs.mkdirSync(dir, {{ recursive: true }});
const runId = "20261029T000000000Z-" + Date.now().toString(36);
const file = path.join(dir, runId + ".json");
const record = {{ kind: "check-run", id: runId, project: "alpha", commit: args[args.indexOf("--commit") + 1], status: "running", startedAt: new Date().toISOString(), steps: [] }};
fs.writeFileSync(file + ".tmp", JSON.stringify(record));
fs.renameSync(file + ".tmp", file);
console.log(file);
"#
    );
    fs::write(script, source).expect("fake brock");
}

fn seed_with_pr(coord_home: &Path, folder: &Path) {
    let mut board = alpha_with_worker(folder);
    let pr = json!({ "id": "PR-1", "title": "Add x", "head": "feat/x", "base": "main",
        "headCommit": "a6c5aba0", "status": "open", "checks": [] });
    board["prs"] = json!({ "PR-1": pr });
    argus_testkit::write_board(coord_home, "alpha", &board);
}

#[tokio::test(flavor = "multi_thread")]
async fn check_of_a_pr_without_a_commit_checks_the_head_commit_the_pr_records() {
    let (tools, root) = (
        tempfile::tempdir().expect("tools"),
        tempfile::tempdir().expect("root"),
    );
    let script = tools.path().join("fake-brock.cjs");
    let config = json!({
        "checks": { "command": format!("node \"{}\"", script.display()), "enabled": true },
        "projectRoots": { "alpha": root.path().display().to_string() },
    });
    let rig = Rig::start(config, seed_with_pr).await;
    write_fake_script(&rig.t.coord_home, &script);
    let ran = rig.run(&["check", "--pr", "PR-1", "--project", "alpha"]);
    assert_eq!(ok(&ran).trim(), "R-1 C-1 check started");
    let run = rig.t.get("/api/p/alpha/runs/R-1").await.json()["run"].clone();
    let command = run["cmd"].as_str().unwrap_or_default();
    assert!(command.contains("--commit a6c5aba0"), "brock got {command}");
    let record = rig.t.get("/api/p/alpha/checks?run=R-1").await.json();
    assert_eq!(record["runs"][0]["commit"], "a6c5aba0");
    let explicit = [
        "check",
        "--pr",
        "PR-1",
        "--commit",
        SHA,
        "--project",
        "alpha",
    ];
    assert_eq!(ok(&rig.run(&explicit)).trim(), "R-2 C-2 check started");
    let record = rig.t.get("/api/p/alpha/checks?run=R-2").await.json();
    assert_eq!(record["runs"][0]["commit"], SHA, "an explicit commit wins");
    let unknown = rig.run(&["check", "--pr", "PR-9", "--project", "alpha"]);
    refused(&unknown, 1, "PR-9");
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn check_of_a_run_that_ends_at_once_always_prints_its_record() {
    let (tools, root) = (
        tempfile::tempdir().expect("tools"),
        tempfile::tempdir().expect("root"),
    );
    let script = tools.path().join("fake-brock.cjs");
    let config = json!({
        "checks": { "command": format!("node \"{}\"", script.display()), "enabled": true },
        "projectRoots": { "alpha": root.path().display().to_string() },
    });
    let rig = Rig::start(config, seed).await;
    write_fake_script(&rig.t.coord_home, &script);
    for n in 1..=20 {
        let ran = rig.run(&["check", "--commit", SHA, "--project", "alpha"]);
        assert_eq!(ok(&ran).trim(), format!("R-{n} C-{n} check started"));
    }
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn check_starts_a_run_and_prints_its_run_and_record() {
    let (tools, root) = (
        tempfile::tempdir().expect("tools"),
        tempfile::tempdir().expect("root"),
    );
    let script = tools.path().join("fake-brock.cjs");
    let config = json!({
        "checks": { "command": format!("node \"{}\"", script.display()), "enabled": true },
        "projectRoots": { "alpha": root.path().display().to_string() },
    });
    let rig = Rig::start(config, seed).await;
    write_fake_script(&rig.t.coord_home, &script);
    let ran = rig.run(&[
        "check",
        "--commit",
        SHA,
        "--pr",
        "PR-1",
        "--project",
        "alpha",
    ]);
    assert_eq!(ok(&ran).trim(), "R-1 C-1 check started");
    let run = rig.t.get("/api/p/alpha/runs/R-1").await.json()["run"].clone();
    assert_eq!(
        (run["name"].as_str(), run["by"].as_str()),
        (Some("check"), Some("worker"))
    );
    let record = rig.t.get("/api/p/alpha/checks?run=R-1").await.json();
    assert_eq!(record["runs"][0]["commit"], SHA);
    let refused_run = rig.run(&["check", "--commit", "zz", "--project", "alpha"]);
    refused(&refused_run, 1, "commit must be 7 to 40 hex characters");
    let board_check = rig.run(&["check", "--project", "alpha", "--json"]);
    assert_eq!(board_check.code, 0, "{}", board_check.err);
    let runs = rig.t.get("/api/p/alpha/runs").await.json();
    assert_eq!(
        runs["runs"].as_array().map(Vec::len),
        Some(1),
        "the board checker starts no run"
    );
    rig.stop().await;
}
