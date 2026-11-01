//! E2: check runs on the run engine with a fake `brock` that is a real Node process, and a real
//! temporary repository for the guard: the start and its link to the record, the head commit of
//! a pull request, Stop, the integration checkout and the guard that fails a run that changed
//! tracked files. Never the real Brock, never a repository of the owner.
// test-tag: e2e

use std::path::Path;

use crate::pr_rig::{SHA, linked_record, open_pr, rig, start_check};
use crate::wait::until;
use serde_json::{Value, json};

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn a_check_run_starts_on_the_run_engine_and_is_linked_to_its_record() {
    let r = rig(false).await;
    open_pr(&r.t).await;
    let body =
        json!({ "commit": SHA, "pr": "PR-1", "job": "gate", "trigger": "pr", "check": "check" });
    let rid = start_check(&r.t, &body).await;
    assert_eq!(rid, "R-1");
    let record = linked_record(&r.t, "?pr=PR-1", &rid).await;
    assert_eq!(record["id"], "C-1");
    assert_eq!(record["commit"], SHA);
    let argv = std::fs::read_to_string(r.out.path().join("argv.json")).expect("argv");
    let want = json!([
        "ci",
        "check",
        "--commit",
        SHA,
        "--pr",
        "PR-1",
        "--job",
        "gate",
        "--trigger",
        "pr",
        "--project",
        "alpha"
    ]);
    assert_eq!(serde_json::from_str::<Value>(&argv).expect("json"), want);
    let run = r.t.get(&format!("/api/p/alpha/runs/{rid}")).await.json()["run"].clone();
    assert_eq!(
        (run["name"].as_str(), run["visible"].as_bool()),
        (Some("check"), Some(false))
    );
    let t = &r.t;
    let lists = until("the check list of the pull request", || async move {
        let checks = t.board("alpha")["prs"]["PR-1"]["checks"].clone();
        (checks != json!([])).then_some(checks)
    })
    .await;
    assert_eq!(lists, json!(["C-1"]));
    r.t.stop(false).await;
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn a_check_run_of_a_pr_without_a_commit_checks_the_head_commit_the_pr_records() {
    let r = rig(false).await;
    open_pr(&r.t).await;
    let update = json!({ "headCommit": "a6c5aba0" });
    let updated = r.t.write("PATCH", "/api/p/alpha/prs/PR-1", &update).await;
    assert_eq!(updated.status, 200, "{}", updated.text());
    let rid = start_check(&r.t, &json!({ "pr": "PR-1", "trigger": "pr" })).await;
    let record = linked_record(&r.t, "?pr=PR-1", &rid).await;
    assert_eq!(record["commit"], "a6c5aba0");
    let text = std::fs::read_to_string(r.out.path().join("argv.json")).expect("argv");
    let words = serde_json::from_str::<Value>(&text).expect("json");
    assert_eq!(
        (&words[1], &words[2]),
        (&json!("--commit"), &json!("a6c5aba0"))
    );
    let explicit = json!({ "pr": "PR-1", "commit": SHA });
    let rid = start_check(&r.t, &explicit).await;
    let record = linked_record(&r.t, "?pr=PR-1", &rid).await;
    assert_eq!(record["commit"], SHA, "an explicit commit wins");
    let unknown =
        r.t.write("POST", "/api/p/alpha/checks", &json!({ "pr": "PR-9" }))
            .await;
    assert_eq!(unknown.status, 404, "{}", unknown.text());
    assert!(unknown.text().contains("PR-9"), "{}", unknown.text());
    r.t.stop(false).await;
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn a_check_run_stops_through_the_run_engine() {
    let r = rig(false).await;
    let rid = start_check(&r.t, &json!({ "trigger": "release" })).await;
    linked_record(&r.t, "", &rid).await;
    let stop =
        r.t.write("POST", &format!("/api/p/alpha/runs/{rid}/stop"), &json!({}))
            .await;
    assert_eq!(stop.status, 200, "{}", stop.text());
    let t = &r.t;
    let path = format!("/api/p/alpha/runs/{rid}");
    let run = until("the run to stop", || async {
        let run = t.get(&path).await.json()["run"].clone();
        (run["status"] == "stopped").then_some(run)
    })
    .await;
    assert_eq!(run["status"], "stopped");
    r.t.stop(false).await;
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn a_check_run_starts_in_the_integration_checkout_when_one_is_set() {
    let r = rig(false).await;
    let ci = tempfile::tempdir().expect("ci");
    set_ci_root(&r.t.coord_home, ci.path());
    let rid = start_check(&r.t, &json!({ "trigger": "manual" })).await;
    let run = r.t.get(&format!("/api/p/alpha/runs/{rid}")).await.json()["run"].clone();
    let flat = |s: &str| s.replace('\\', "/").to_lowercase();
    let want = flat(&ci.path().display().to_string());
    assert_eq!(flat(run["cwd"].as_str().expect("cwd")), want);
    let gone = ci.path().join("gone");
    set_ci_root(&r.t.coord_home, &gone);
    let refused = r.t.write("POST", "/api/p/alpha/checks", &json!({})).await;
    assert_eq!(refused.status, 400, "{}", refused.text());
    assert!(
        refused.text().contains("not a folder"),
        "{}",
        refused.text()
    );
    r.t.stop(false).await;
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn a_check_run_that_deletes_a_tracked_file_is_failed_with_a_note_and_a_warn_line() {
    let r = rig(false).await;
    tracked_repo(r.root.path());
    let rid = start_check(&r.t, &json!({ "trigger": "push" })).await;
    let t = &r.t;
    let path = format!("/api/p/alpha/runs/{rid}");
    let run = until("the guard to fail the run", || async {
        let run = t.get(&path).await.json()["run"].clone();
        let noted = run["error"]
            .as_str()
            .is_some_and(|e| e.contains("tracked.txt"));
        noted.then_some(run)
    })
    .await;
    assert_eq!(run["status"], "failed");
    let note = run["error"].as_str().expect("a note").to_owned();
    assert!(
        note.starts_with("brock ci changed tracked files in "),
        "{note}"
    );
    let record = linked_record(&r.t, "", &rid).await;
    assert_eq!(record["id"], "C-1");
    assert_eq!(record["status"], "failed");
    assert_eq!(record["guardNote"], note.as_str());
    let log = std::fs::read_to_string(r.t.svc.log.file()).expect("server log");
    let warned: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("check run changed tracked files"))
        .collect();
    assert_eq!(warned.len(), 1, "{log}");
    assert!(warned[0].contains("\"warn\""), "{}", warned[0]);
    r.t.stop(false).await;
}

#[ignore = "e2e"]
#[tokio::test(flavor = "multi_thread")]
async fn a_check_run_that_leaves_the_tracked_files_alone_stays_as_brock_wrote_it() {
    let r = rig(false).await;
    tracked_repo(r.root.path());
    let rid = start_check(&r.t, &json!({ "trigger": "manual" })).await;
    let t = &r.t;
    let path = format!("/api/p/alpha/runs/{rid}");
    let run = until("the run to end", || async {
        let run = t.get(&path).await.json()["run"].clone();
        (run["status"] != "running" && run["status"] != "queued").then_some(run)
    })
    .await;
    assert_eq!(
        (run["status"].as_str(), run["error"].as_str()),
        (Some("exited"), None)
    );
    let record = linked_record(&r.t, "", &rid).await;
    assert_eq!(record["status"], "running", "as the fake Brock wrote it");
    assert!(record.get("guardNote").is_none());
    r.t.stop(false).await;
}

fn set_ci_root(coord_home: &Path, dir: &Path) {
    let roots = json!({ "alpha": dir.display().to_string().replace('\\', "/") });
    let text = json!({ "mode": "local", "ciRoots": roots }).to_string();
    std::fs::write(coord_home.join("remote.json"), text).expect("remote.json");
}

fn git(dir: &Path, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .stdout(std::process::Stdio::null())
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?}");
}

/// The project root as a temp repository with one tracked file the fake Brock deletes.
fn tracked_repo(dir: &Path) {
    std::fs::write(dir.join("tracked.txt"), "keep me\n").expect("file");
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "tracked.txt"]);
    git(dir, &["commit", "-q", "-m", "one"]);
}
