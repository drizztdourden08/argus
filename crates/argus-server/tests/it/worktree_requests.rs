//! Removal requests through the whole server (Node `c81fa17`, `worktrees.requests.http.test.mjs`): a
//! thread asks over HTTP or with `coord worktree request`, which only records ("queued R-n ...
//! Nothing is removed by this command"), the server's worker removes behind the same guards, the
//! result comes to the lead's inbox, `coord worktree requests` lists the sentences, and the
//! summary the page reads carries the brief form. A stranger, a thread of another project and the
//! page are refused. The CLI cases need `ARGUS_COORD_CLI` (Node) or the `argus` binary.
//! E2, K2, K4: removal requests through the whole server, with the real git and the coord CLI.
// test-tag: e2e

use crate::worktrees_rig::{Rig, id_of, until, wakes_of};
use argus_core::clock::{iso_from_ms, now_ms};
use argus_testkit::CoordCli;
use argus_worktrees::requests_key;
use serde_json::{Value, json};

const REQUESTS: &str = "/api/p/alpha/worktrees/requests";

async fn requests(r: &Rig) -> Vec<Value> {
    r.t.get(REQUESTS).await.json()["requests"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

async fn first_is(r: &Rig, status: &str) -> Option<Vec<Value>> {
    let list = requests(r).await;
    (list.first()?["status"] == status).then_some(list)
}

fn inbox_note(r: &Rig, text: &str) -> Option<Value> {
    let board = r.t.board("alpha");
    let notes = board["inbox"]["ui"].as_array().cloned().unwrap_or_default();
    notes.into_iter().find(|m| m["text"] == text)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn a_stranger_a_thread_of_another_project_and_the_page_cannot_ask_nothing_is_stored_and_a_bad_shape_is_400()
 {
    let r = Rig::start().await;
    r.scanned().await;
    let wid = id_of(&r.path("old-merged"));
    let ask = |by: &str| json!({ "targets": [wid], "by": by });
    let ghost = r.cli(REQUESTS, &ask("ghost")).await;
    assert_eq!(ghost.status, 403, "{}", ghost.text());
    assert!(
        ghost.json()["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("\"ghost\" is not a live thread here"))
    );
    let api = r.cli(REQUESTS, &ask("api")).await; // api is a thread of beta
    assert_eq!(api.status, 403);
    let beta = r.cli("/api/p/beta/worktrees/requests", &ask("ui")).await; // ui is a thread of alpha
    assert_eq!(beta.status, 403);
    let page = r.page(REQUESTS, &json!({ "targets": [wid] })).await;
    assert_eq!(page.status, 403);
    let bad = r.cli(REQUESTS, &json!({ "by": "ui" })).await;
    assert_eq!(bad.status, 400, "{}", bad.text());
    assert_eq!(requests(&r).await.len(), 0);
    assert!(r.path("old-merged").exists());
    r.t.stop(false).await;
}

async fn the_summary_and_the_activity_follow(r: &Rig) {
    until("the summary", || async {
        let meta = r.meta().await;
        let seen: Vec<(Value, Value)> = meta["requests"]
            .as_array()?
            .iter()
            .map(|x| (x["id"].clone(), x["status"].clone()))
            .collect();
        (seen
            == [
                (json!("R-2"), json!("refused")),
                (json!("R-1"), json!("done")),
            ])
        .then_some(())
    })
    .await;
    let lines = r.activity("worktree-request").await;
    assert!(
        lines
            .iter()
            .any(|t| t == "alpha: worktree request R-1 done: removed 1 (asked by ui)"),
        "{lines:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn a_request_is_recorded_at_once_the_worker_removes_the_lead_is_told_and_the_summary_the_page_reads_has_the_brief()
 {
    let r = Rig::start().await;
    r.scanned().await;
    let merged = id_of(&r.path("old-merged"));
    let queued = r
        .cli(REQUESTS, &json!({ "targets": [merged], "branch": true, "reason": "finished agents", "by": "ui" }))
        .await;
    assert_eq!(queued.status, 200, "{}", queued.text());
    let answer = queued.json();
    assert_eq!(
        (&answer["id"], &answer["status"]),
        (&json!("R-1"), &json!("queued"))
    );
    let text = answer["text"].as_str().unwrap();
    assert!(
        text.starts_with(
            "queued R-1: Argus will remove 1 worktree (and their branches where merged): "
        ),
        "{text}"
    );
    assert!(text.ends_with("Nothing is removed by this command; the result comes to your inbox (coord worktree requests shows it)."), "{text}");
    assert_eq!(answer["request"]["total"], 1);
    let list = until("the request done", || first_is(&r, "done")).await;
    assert_eq!(list[0]["results"][0]["ok"], true);
    assert!(!r.path("old-merged").exists(), "the server removed it");
    let note = until("the note in the inbox", || async {
        inbox_note(&r, "worktree request R-1 done: removed 1")
    })
    .await;
    assert_eq!(note["from"], "Argus");
    until("the wake", || async {
        wakes_of(&r.t, "alpha", "thread-message")
            .await
            .iter()
            .any(|w| w["thread"] == "ui")
            .then_some(())
    })
    .await;
    let unsafe_one = json!({ "targets": [id_of(&r.path("dirty-old"))], "by": "ui" });
    assert_eq!(r.cli(REQUESTS, &unsafe_one).await.status, 200);
    let both = until("the refusal", || first_is(&r, "refused")).await;
    assert!(
        r.path("dirty-old").exists(),
        "uncommitted work is not removed through a request"
    );
    let ids: Vec<&Value> = both.iter().map(|x| &x["id"]).collect();
    assert_eq!(ids, [&json!("R-2"), &json!("R-1")]);
    the_summary_and_the_activity_follow(&r).await;
    r.t.stop(false).await;
}

/// A request by path as the server records it, queued and not yet run.
fn queued_by_path(path: &str) -> String {
    let target = json!({ "id": path, "label": path });
    let rec = json!({ "id": "R-1", "key": "alpha", "by": "ui", "at": iso_from_ms(now_ms()), "mode": "targets", "branch": true, "targets": [target], "resolved": true, "status": "queued" });
    json!({ "seq": 1, "list": [rec] }).to_string()
}

/// The live bug of 2026-10-29: a request filed before a restart, naming its worktree by path, must
/// be carried out by the next server, which scans the project on its own first.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn a_request_by_path_filed_before_a_restart_is_carried_out_by_the_next_server_with_no_scan_before_it()
 {
    let r = Rig::start().await;
    let path = r
        .path("old-merged")
        .display()
        .to_string()
        .replace('\\', "/");
    r.t.svc
        .db
        .kv_set(&requests_key("alpha"), &queued_by_path(&path))
        .expect("the request is stored");
    let r = r.restart().await;
    let list = until("the request done", || first_is(&r, "done")).await;
    assert_eq!(list[0]["results"][0]["ok"], true, "{}", list[0]);
    assert!(!r.path("old-merged").exists(), "the server removed it");
    r.scanned().await;
    r.t.stop(false).await;
}

fn said(out: &argus_testkit::CliOut) -> String {
    format!("{} {}", out.stdout, out.stderr)
}

/// One engine of the CLI against one rig, as the thread `ui`.
struct Asker<'a> {
    cli: &'a CoordCli,
    r: &'a Rig,
    dir: tempfile::TempDir,
}

impl Asker<'_> {
    async fn run(&self, words: &[&str]) -> argus_testkit::CliOut {
        let mut all = words.to_vec();
        all.extend(["--project", "alpha", "--as", "ui"]);
        self.cli.run(&self.r.t, &all, self.dir.path(), &[]).await
    }

    fn path(&self, name: &str) -> String {
        self.r.path(name).display().to_string().replace('\\', "/")
    }
}

async fn queues_and_removes(a: &Asker<'_>) {
    let name = a.cli.name();
    let merged = a.path("old-merged");
    let refused = a
        .run(&["worktree", "request", "--finished", "--discard"])
        .await;
    assert_eq!(refused.status, 1, "{name}: {}", said(&refused));
    assert!(
        refused.stderr.contains("never discard work"),
        "{name}: {}",
        refused.stderr
    );
    let both = a.run(&["worktree", "request", "--finished", &merged]).await;
    assert_eq!(both.status, 1, "{name}");
    assert!(
        both.stderr.contains("ask for one thing"),
        "{name}: {}",
        both.stderr
    );
    // A path after a switch is a worktree again.
    let mixed = a.run(&["worktree", "request", "--branch", &merged]).await;
    assert_eq!(mixed.status, 0, "{name}: {}", said(&mixed));
    let head = "queued R-1: Argus will remove 1 worktree (and their branches where merged): ";
    let tail = "old-merged. Nothing is removed by this command; the result comes to your inbox";
    assert!(
        mixed.stdout.starts_with(head) && mixed.stdout.contains(tail),
        "{name}: {}",
        mixed.stdout
    );
    until("the request done", || first_is(a.r, "done")).await;
    assert!(
        !a.r.path("old-merged").exists(),
        "{name}: the server removed it"
    );
    until("the note in the inbox", || async {
        inbox_note(a.r, "worktree request R-1 done: removed 1")
    })
    .await;
}

async fn lists_the_sentences(a: &Asker<'_>) {
    let name = a.cli.name();
    let unsafe_one = a.run(&["worktree", "request", &a.path("dirty-old")]).await;
    assert_eq!(unsafe_one.status, 0, "{name}: {}", said(&unsafe_one));
    until("the refusal", || first_is(a.r, "refused")).await;
    assert!(a.r.path("dirty-old").exists(), "{name}");
    let table = a.run(&["worktree", "requests"]).await;
    assert_eq!(table.status, 0, "{name}: {}", table.stderr);
    let lines: Vec<&str> = table.stdout.lines().collect();
    assert!(
        lines[0].starts_with("R-2 refused  1 worktree  by ui "),
        "{name}: {}",
        table.stdout
    );
    let sentence = "dirty-old has 1 uncommitted change: removing it discards them";
    assert!(
        lines[1].starts_with("    refused: ") && lines[1].contains(sentence),
        "{name}: {}",
        table.stdout
    );
    assert!(
        lines[2].starts_with("R-1 done  1 worktree"),
        "{name}: {}",
        table.stdout
    );
    let json = a.run(&["worktree", "requests", "--json"]).await;
    let parsed: Value = serde_json::from_str(&json.stdout).expect("json");
    let ids: Vec<&Value> = parsed["requests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| &x["id"])
        .collect();
    assert_eq!(ids, [&json!("R-2"), &json!("R-1")], "{name}");
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "e2e"]
async fn coord_worktree_request_records_and_returns_and_coord_worktree_requests_lists_the_sentences()
 {
    for cli in CoordCli::locate_each() {
        let r = Rig::start().await;
        r.scanned().await;
        let asker = Asker {
            cli: &cli,
            r: &r,
            dir: tempfile::tempdir().expect("a folder"),
        };
        queues_and_removes(&asker).await;
        lists_the_sentences(&asker).await;
        r.t.stop(false).await;
    }
}
