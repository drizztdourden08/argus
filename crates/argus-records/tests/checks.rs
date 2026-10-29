//! The check run store on a temp folder: numbering in first-seen order, numbers kept by the
//! book across a new store (a restart), filters, logs, and torn files.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use argus_records::checks::{CheckRun, CheckStore, IdBook, run_json};
use serde_json::{Value, json};

/// A book in memory that outlives the stores, as the database does.
#[derive(Default)]
struct Book(Mutex<HashMap<(String, String), u32>>);

impl IdBook for Book {
    fn number(&self, project: &str, run_id: &str) -> Option<u32> {
        let mut map = self.0.lock().expect("book");
        let next = u32::try_from(map.keys().filter(|(p, _)| p == project).count()).unwrap_or(0) + 1;
        Some(*map.entry((project.into(), run_id.into())).or_insert(next))
    }
}

fn run(root: &Path, project: &str, id: &str, started: &str, commit: &str, pr: &str) {
    let record = json!({
        "kind": "check-run", "id": "", "project": project, "commit": commit, "pr": pr,
        "status": "pass", "startedAt": started, "finishedAt": started,
        "steps": [{ "name": "lint", "command": "pnpm lint", "exitCode": 0, "log": format!("{id}/lint.log") }],
        "replay": { "state": "pending" },
    });
    let file = root.join(project).join(format!("{id}.json"));
    fs::create_dir_all(root.join(project).join(id)).expect("folders");
    fs::write(root.join(project).join(id).join("lint.log"), "lint ok\n").expect("log");
    fs::write(file, record.to_string()).expect("record");
}

fn store(root: &Path, book: &Arc<Book>) -> CheckStore {
    let s = CheckStore::new(root, book.clone());
    s.rescan();
    s
}

#[test]
fn runs_are_numbered_in_the_order_they_were_first_seen_and_listed_newest_first() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("ci");
    run(
        &root,
        "brock",
        "run-b",
        "2026-10-29T10:05:00Z",
        "bbbbbbb1",
        "PR-1",
    );
    run(
        &root,
        "brock",
        "run-a",
        "2026-10-29T10:00:00Z",
        "aaaaaaa1",
        "PR-1",
    );
    run(
        &root,
        "rotp",
        "run-z",
        "2026-10-29T09:00:00Z",
        "ccccccc1",
        "",
    );
    let s = store(&root, &Arc::new(Book::default()));
    let ids: Vec<String> = s
        .of_project("brock", None, None)
        .iter()
        .map(CheckRun::id)
        .collect();
    assert_eq!(ids, ["C-2", "C-1"], "run-a started first, so it is C-1");
    assert_eq!(
        s.of_project("rotp", None, None)[0].id(),
        "C-1",
        "numbers are per project"
    );
    let one = run_json(&s.find("brock", "C-2").expect("C-2"));
    assert_eq!(
        (one["id"].as_str(), one["runId"].as_str()),
        (Some("C-2"), Some("run-b"))
    );
    assert_eq!(one["project"], "brock");
}

#[test]
fn numbers_stay_after_a_restart_and_a_later_run_gets_the_next_one() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("ci");
    let book = Arc::new(Book::default());
    run(
        &root,
        "brock",
        "run-a",
        "2026-10-29T10:00:00Z",
        "aaaaaaa1",
        "",
    );
    run(
        &root,
        "brock",
        "run-b",
        "2026-10-29T10:05:00Z",
        "bbbbbbb1",
        "",
    );
    let first = store(&root, &book);
    let before: Vec<(String, String)> = first
        .runs()
        .iter()
        .map(|r| (r.run_id.clone(), r.id()))
        .collect();
    run(
        &root,
        "brock",
        "run-0",
        "2026-10-29T08:00:00Z",
        "0000000a",
        "",
    );
    let second = store(&root, &book);
    for (run_id, id) in before {
        let now = second
            .runs()
            .iter()
            .find(|r| r.run_id == run_id)
            .map(CheckRun::id);
        assert_eq!(now, Some(id), "{run_id}");
    }
    let old = second
        .runs()
        .iter()
        .find(|r| r.run_id == "run-0")
        .map(CheckRun::id);
    assert_eq!(
        old.as_deref(),
        Some("C-3"),
        "an older run seen later is numbered after"
    );
}

#[test]
fn runs_are_narrowed_by_commit_prefix_and_pull_request() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("ci");
    run(
        &root,
        "brock",
        "run-a",
        "2026-10-29T10:00:00Z",
        "aaaaaaa1234567",
        "PR-1",
    );
    run(
        &root,
        "brock",
        "run-b",
        "2026-10-29T10:05:00Z",
        "bbbbbbb1234567",
        "PR-2",
    );
    let s = store(&root, &Arc::new(Book::default()));
    assert_eq!(s.of_project("brock", Some("aaaaaaa"), None).len(), 1);
    assert_eq!(s.of_project("brock", Some("aaaaaaa1234567"), None).len(), 1);
    assert_eq!(
        s.of_project("brock", Some("aaa"), None).len(),
        0,
        "too short"
    );
    assert_eq!(s.of_project("brock", None, Some("pr-2"))[0].run_id, "run-b");
    assert_eq!(s.pending(), 2);
}

#[test]
fn a_step_log_is_found_inside_the_store_only_and_torn_files_are_skipped() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root = dir.path().join("ci");
    run(
        &root,
        "brock",
        "run-a",
        "2026-10-29T10:00:00Z",
        "aaaaaaa1",
        "",
    );
    fs::write(root.join("brock/torn.json"), "{ half").expect("torn");
    fs::write(root.join("brock/run-t.json.tmp"), "{}").expect("temp");
    let outside = dir.path().join("secret.log");
    fs::write(&outside, "no").expect("outside");
    let s = store(&root, &Arc::new(Book::default()));
    assert_eq!(s.runs().len(), 1);
    let found = s.find("brock", "C-1").expect("run");
    let log = s.step_log(&found, 0).expect("log");
    assert_eq!(fs::read_to_string(log).expect("read"), "lint ok\n");
    assert_eq!(s.step_log(&found, 5), None);
    let mut moved = found.clone();
    moved.record["steps"][0]["log"] = Value::String(outside.display().to_string());
    assert_eq!(
        s.step_log(&moved, 0),
        None,
        "an absolute path outside the store"
    );
    moved.record["steps"][0]["log"] = Value::String("../../secret.log".into());
    assert_eq!(
        s.step_log(&moved, 0),
        None,
        "a relative path out of the store"
    );
    moved.record["logDir"] = Value::String(dir.path().display().to_string());
    assert!(s.step_log(&moved, 0).is_some(), "the run's own log dir");
}
