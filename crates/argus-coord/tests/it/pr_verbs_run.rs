//! E2: coord pr merge runs a fake `brock` program, and coord check starts runs of a fake `brock`
//! script through an in-process Argus test server.

// test-tag: e2e

use std::fs;
use std::path::{Path, PathBuf};

use crate::common::Ran;
use crate::pr_verbs::Local;
use crate::remote::{Rig, alpha_with_worker, env_plus, ok, refused};
use argus_coord::Env;
use serde_json::json;

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const MERGE: &str = "fedcba9876543210fedcba9876543210fedcba98";

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
#[ignore = "e2e"]
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
    assert_eq!(pr["mergedBy"], "lead");
    assert!(pr["mergedAt"].is_string());
    let args = fs::read_to_string(&record).expect("args");
    assert_eq!(args.trim().trim_matches('"'), "pr merge --local PR-1");
    let cwd = fs::read_to_string(format!("{}.cwd", record.display())).expect("cwd");
    let flat = |s: &str| s.trim().replace('\\', "/").to_lowercase();
    assert!(flat(&cwd).ends_with("checkout"), "brock ran in {cwd}");
}

#[test]
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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

/// A server whose `checks.command` is a fake `brock` script, with the board seeded by `seed`. The two temp folders
/// (the script's and the project root) live as long as the second value.
async fn check_rig(
    seed: impl FnOnce(&Path, &Path),
) -> (Rig, (tempfile::TempDir, tempfile::TempDir)) {
    let dirs = (
        tempfile::tempdir().expect("tools"),
        tempfile::tempdir().expect("root"),
    );
    let script = dirs.0.path().join("fake-brock.cjs");
    let config = json!({
        "checks": { "command": format!("node \"{}\"", script.display()), "enabled": true },
        "projectRoots": { "alpha": dirs.1.path().display().to_string() },
    });
    let rig = Rig::start(config, seed).await;
    write_fake_script(&rig.t.coord_home, &script);
    (rig, dirs)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn check_of_a_pr_without_a_commit_checks_the_head_commit_the_pr_records() {
    let (rig, _dirs) = check_rig(seed_with_pr).await;
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
#[ignore = "e2e"]
async fn check_of_a_run_that_ends_at_once_always_prints_its_record() {
    let (rig, _dirs) = check_rig(seed).await;
    for n in 1..=20 {
        let ran = rig.run(&["check", "--commit", SHA, "--project", "alpha"]);
        assert_eq!(ok(&ran).trim(), format!("R-{n} C-{n} check started"));
    }
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn check_starts_a_run_and_prints_its_run_and_record() {
    let (rig, _dirs) = check_rig(seed).await;
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

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn check_with_localpkg_passes_the_flag_to_brock_ci() {
    let (rig, _dirs) = check_rig(seed).await;
    let ran = rig.run(&["check", "--commit", SHA, "--localpkg", "--project", "alpha"]);
    assert_eq!(ok(&ran).trim(), "R-1 C-1 check started");
    let run = rig.t.get("/api/p/alpha/runs/R-1").await.json()["run"].clone();
    let command = run["cmd"].as_str().unwrap_or_default();
    assert!(
        command.contains("--localpkg --project alpha"),
        "brock got {command}"
    );
    let alone = rig.run(&["check", "--localpkg", "--project", "alpha"]);
    assert_eq!(
        ok(&alone).trim(),
        "R-2 C-2 check started",
        "--localpkg alone asks for a run"
    );
    let swallowed = rig.run(&["check", "--localpkg", "gate", "--project", "alpha"]);
    refused(&swallowed, 1, "--localpkg takes no value");
    rig.stop().await;
}
