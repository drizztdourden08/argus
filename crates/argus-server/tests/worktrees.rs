//! The worktree scanner and its removal routes inside the whole server, on throwaway repositories:
//! the first scan, the docs and the summary, the one wake to the rule's thread, removal from the
//! page and by a thread, the leftover folders and their background deletion, a change of rules,
//! and shadow mode. Ported from the HTTP side of `worktrees.http.test.mjs` and
//! `worktrees.fallback.http.test.mjs`; the `coord worktree` verbs stay in the Node `coord` tests.

#[path = "worktrees/fixture.rs"]
mod fixture;
#[path = "worktrees/rig.rs"]
mod rig;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use argus_testkit::{Request, write_board};
use argus_worktrees::{GitExec, GitOutput, SystemExec};
use fixture::git;
use rig::{Rig, Setup, id_of, rule, until, wakes_of};
use serde_json::{Value, json};

fn status_counts(meta: &Value) -> Value {
    meta["byStatus"].clone()
}

async fn gone(path: &Path) {
    until("the folder to go", || async {
        (!path.exists()).then_some(())
    })
    .await;
}

async fn row_gone(r: &Rig, name: &str) {
    until(&format!("the row {name} to go"), || async {
        (!r.docs().await.contains_key(name)).then_some(())
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_first_scan_makes_a_doc_per_worktree_a_summary_and_one_wake_to_the_rules_thread() {
    let r = Rig::start().await;
    let meta = r.scanned().await;
    assert_eq!(
        (&meta["count"], &meta["gone"]),
        (&json!(5), &json!(1)),
        "the deleted folder is only counted"
    );
    assert_eq!(
        status_counts(&meta),
        json!({ "active": 1, "stale": 1, "merged": 1, "dirty-stale": 1, "leftover": 1 })
    );
    assert_eq!(meta["rules"][0]["id"], "W-1");
    let docs = r.docs().await;
    assert_eq!(docs["fresh"]["id"], json!(id_of(&r.path("fresh"))));
    assert_eq!(docs["old-merged"]["status"], "merged");
    let wake = until("the stale wake", || async {
        wakes_of(&r.t, "alpha", "worktree-stale").await.pop()
    })
    .await;
    assert_eq!(wake["thread"], "ui");
    let text = wake["text"].as_str().unwrap();
    assert!(text.starts_with("3 worktrees need a decision: "), "{text}");
    assert_eq!(
        wakes_of(&r.t, "alpha", "worktree-stale").await.len(),
        1,
        "one wake"
    );
    until("the activity line", || async {
        r.activity("worktree-stale")
            .await
            .iter()
            .any(|t| t.contains("need a decision"))
            .then_some(())
    })
    .await;
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_board_export_keeps_the_scanners_meta_doc_and_its_rows() {
    let r = Rig::start().await;
    let before = r.scanned().await;
    r.t.write("POST", "/api/admin/resync", &json!({ "project": "alpha" }))
        .await;
    let after = r.meta().await;
    assert_eq!(after["scannedAt"], before["scannedAt"]);
    assert_eq!(r.docs().await.len(), 5);
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_docs_are_read_only_for_the_page_and_the_collection_is_known_to_the_stream() {
    let r = Rig::start().await;
    r.scanned().await;
    let path = format!("/api/p/alpha/db/worktrees/{}", id_of(&r.path("fresh")));
    let put = Request::new("PUT", &path).json(&json!({ "data": { "id": "x" } }));
    assert_eq!(r.t.send(r.t.page(put)).await.status, 403);
    let stream = r.t.sse("/api/p/alpha/stream?c=worktrees,meta").await;
    assert!(
        stream.next(Some("snapshot"), 5000).await.is_some(),
        "a snapshot of the two collections"
    );
    stream.close();
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_page_removes_a_merged_worktree_and_its_merged_branch_the_row_leaves_and_an_activity_line_says_so()
 {
    let r = Rig::start().await;
    r.scanned().await;
    let id = id_of(&r.path("old-merged"));
    let out = r
        .page(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id, "branch": true }),
        )
        .await;
    assert_eq!(out.status, 200, "{}", out.text());
    let done = out.json();
    assert_eq!(
        [
            &done["removed"],
            &done["prunedOnly"],
            &done["deletedBranch"],
            &done["backup"]
        ],
        [&json!(true), &json!(false), &json!(true), &Value::Null]
    );
    let text = done["text"].as_str().unwrap();
    assert!(
        text.starts_with("removed worktree ")
            && text.ends_with(
                "old-merged, branch old-merged deleted: merged into local main (not pushed)"
            ),
        "{text}"
    );
    assert!(!r.docs().await.contains_key("old-merged"));
    assert!(!r.path("old-merged").exists());
    let lines = r.activity("worktree-remove").await;
    assert!(
        lines[0].starts_with("alpha: removed worktree ")
            && lines[0].ends_with("for the owner (page)"),
        "{lines:?}"
    );
    let again = r
        .page("/api/p/alpha/worktrees/remove", &json!({ "id": id }))
        .await;
    assert_eq!((again.status, again.code().as_str()), (404, "not_found"));
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cli_caller_must_be_a_live_thread_and_discarding_needs_the_owner_and_the_page_is_never_named_in_a_body()
 {
    let r = Rig::start().await;
    r.scanned().await;
    let id = id_of(&r.path("dirty-old"));
    let ghost = r
        .cli(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id, "by": "ghost" }),
        )
        .await;
    assert_eq!((ghost.status, ghost.code().as_str()), (403, "forbidden"));
    assert!(
        ghost.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("\"ghost\" is not a live thread here")
    );
    let named_page = r
        .cli(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id, "by": "page", "discard": true }),
        )
        .await;
    assert_eq!(
        named_page.status, 403,
        "a body cannot make the caller the page"
    );
    let thread = r
        .cli(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id, "by": "ui", "discard": true }),
        )
        .await;
    assert_eq!(thread.status, 403);
    assert!(
        thread.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("needs the owner: no owner entry was given")
    );
    let bad = r
        .cli("/api/p/Bad_Key/worktrees/remove", &json!({ "id": id }))
        .await;
    assert_eq!(bad.status, 400);
    assert!(r.path("dirty-old").exists(), "nothing was removed");
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_page_may_discard_work_after_a_backup_and_the_gone_worktree_is_pruned() {
    let r = Rig::start().await;
    r.scanned().await;
    let id = id_of(&r.path("dirty-old"));
    let refused = r
        .page("/api/p/alpha/worktrees/remove", &json!({ "id": id }))
        .await;
    assert_eq!(refused.status, 403);
    assert!(
        refused.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("has 1 uncommitted change: removing it discards them")
    );
    let out = r
        .page(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id, "discard": true, "reason": "the owner said so" }),
        )
        .await;
    assert_eq!(out.status, 200, "{}", out.text());
    let done = out.json();
    let patch = done["backup"]["patch"].as_str().expect("a patch backup");
    assert!(Path::new(patch).exists(), "{patch}");
    assert_eq!(done["discarded"]["dirty"], 1);
    row_gone(&r, "dirty-old").await;

    let pruned = r
        .page("/api/p/alpha/worktrees/prune", &json!({}))
        .await
        .json();
    assert_eq!(pruned["pruned"].as_array().unwrap().len(), 1);
    assert!(
        pruned["text"]
            .as_str()
            .unwrap()
            .starts_with("pruned 1 gone worktree"),
        "{pruned}"
    );
    let none = r
        .page("/api/p/alpha/worktrees/prune", &json!({}))
        .await
        .json();
    assert!(none["text"].as_str().unwrap().contains("nothing to prune"));
    r.t.stop(false).await;
}

/// The live check lists the new folder without git, and the summary counts it apart.
async fn sees_leftover(r: &Rig) {
    let row = until("the leftover row", || async {
        r.docs()
            .await
            .get("junk-a")
            .filter(|d| d["status"] == "leftover")
            .cloned()
    })
    .await;
    assert_eq!(
        [
            &row["leftover"]["files"],
            &row["leftover"]["ageDays"],
            &row["leftover"]["hasNestedGit"]
        ],
        [&json!(1), &json!(10), &json!(false)]
    );
    let meta = until("the summary", || async {
        let m = r.meta().await;
        (m["leftovers"].as_u64().unwrap_or(0) >= 2).then_some(m)
    })
    .await;
    assert_eq!(
        (&meta["git"], meta["leftoverOld"]["days"].as_f64()),
        (&json!(4), Some(1.0))
    );
    let raw = r.t.get("/api/p/alpha/db/meta/worktrees").await.text();
    assert!(
        raw.contains(r#""days":1,"#),
        "a whole number has no fraction: {raw}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_leftover_folder_is_found_by_the_live_check_and_deleted_by_a_background_job() {
    let r = Rig::start().await;
    r.scanned().await;
    let junk = r.fx.leftover("junk-a", 10.0);
    sees_leftover(&r).await;
    let no = r
        .cli(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id_of(&junk), "by": "ui", "branch": true }),
        )
        .await;
    assert_eq!(no.status, 400, "{}", no.text());
    let go = r
        .cli(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id_of(&junk), "by": "ui", "reason": "old agent folder" }),
        )
        .await;
    assert_eq!(go.status, 200, "{}", go.text());
    assert_eq!(
        (&go.json()["started"], &go.json()["leftover"]),
        (&json!(true), &json!(true))
    );
    gone(&junk).await;
    row_gone(&r, "junk-a").await;
    until("the finished line", || async {
        r.activity("worktree-remove")
            .await
            .iter()
            .any(|t| t.contains("deleted leftover folder") && t.contains("junk-a"))
            .then_some(())
    })
    .await;
    assert!(
        r.activity("worktree-remove")
            .await
            .iter()
            .any(|t| t.contains("deleting leftover folder")
                && t.ends_with("for ui: old agent folder"))
    );
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_young_leftover_needs_the_stronger_confirm_from_the_page() {
    let r = Rig::start().await;
    r.scanned().await;
    let plain = r.path("plain");
    let young = r
        .page(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id_of(&plain) }),
        )
        .await;
    assert_eq!(young.status, 403);
    assert!(
        young.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("confirm that you want a young folder deleted")
    );
    let go = r
        .page(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id_of(&plain), "confirmYoung": true }),
        )
        .await;
    assert_eq!(go.status, 200, "{}", go.text());
    let out = go.json();
    assert_eq!(
        (&out["started"], &out["leftover"]),
        (&json!(true), &json!(true))
    );
    assert!(out["jobId"].as_str().unwrap().starts_with("L-"));
    gone(&plain).await;
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn removing_all_old_leftovers_queues_them_one_after_another_and_skips_the_young_with_its_reason()
 {
    let r = Rig::start().await;
    r.scanned().await;
    let young = r.fx.leftover("junk-young", 0.0);
    let old = r.fx.leftover("junk-c", 10.0);
    until("the rows", || async {
        let docs = r.docs().await;
        (docs.contains_key("junk-young") && docs.contains_key("junk-c")).then_some(())
    })
    .await;
    let ghost = r
        .cli(
            "/api/p/alpha/worktrees/leftovers/remove",
            &json!({ "by": "ghost" }),
        )
        .await;
    assert_eq!(ghost.status, 403);
    let run = r
        .cli(
            "/api/p/alpha/worktrees/leftovers/remove",
            &json!({ "by": "ui", "olderThanDays": 1, "reason": "cleanup" }),
        )
        .await;
    assert_eq!(run.status, 200, "{}", run.text());
    let out = run.json();
    assert!(out["started"].as_u64().unwrap() >= 1, "{out}");
    let skipped = out["skipped"].as_array().unwrap();
    assert!(
        skipped.iter().any(|s| s["path"]
            .as_str()
            .is_some_and(|p| p.ends_with("junk-young"))
            && s["reason"].as_str().unwrap().contains("under 1 day")),
        "{out}"
    );
    gone(&old).await;
    assert!(young.exists(), "the young one stays");
    until("the batch line", || async {
        r.activity("worktree-remove")
            .await
            .iter()
            .any(|t| t.contains("leftover batch finished"))
            .then_some(())
    })
    .await;
    let page = r
        .page(
            "/api/p/alpha/worktrees/leftovers/remove",
            &json!({ "ids": [id_of(&young)], "olderThanDays": 0, "confirmYoung": true }),
        )
        .await;
    assert_eq!(
        (page.status, page.json()["started"].clone()),
        (200, json!(1))
    );
    gone(&young).await;
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_change_of_rules_is_scanned_within_a_moment_and_a_removed_rule_empties_the_tab() {
    let r = Rig::start().await;
    r.scanned().await;
    let home = r.t.coord_home.clone();
    let mut board = r.t.board("alpha");
    let repo_rule = json!({ "id": "W-2", "path": argus_worktrees::clean_path(&r.fx.repo.display().to_string()), "kind": "repo", "include": [], "exclude": [], "thread": "ui", "note": "the repo", "staleDays": null, "addedBy": "ui", "addedAt": "2026-10-26T00:00:00.000Z" });
    board["worktreeRules"]
        .as_array_mut()
        .unwrap()
        .push(repo_rule);
    write_board(&home, "alpha", &board);
    let meta = until("the scan of the new rule", || async {
        let m = r.meta().await;
        (m["rules"].as_array().map_or(0, Vec::len) == 2 && m["rules"][1]["candidates"] == 4)
            .then_some(m)
    })
    .await;
    assert_eq!(
        (&meta["rules"][1]["kind"], &meta["rules"][1]["exists"]),
        (&json!("repo"), &json!(true))
    );
    board["worktreeRules"] = json!([board["worktreeRules"][1].clone()]);
    write_board(&home, "alpha", &board);
    let fresh = until("the repo rule to list the same worktrees", || async {
        r.docs()
            .await
            .get("fresh")
            .filter(|d| d["rule"] == "W-2")
            .cloned()
    })
    .await;
    assert_eq!(fresh["rule"], "W-2");
    assert_eq!(r.meta().await["rules"].as_array().unwrap().len(), 1);
    r.t.stop(false).await;
}

/// What git does to a worktree, by folder name: `stuck` deletes the tracked files and `.git`, then
/// says "Directory not empty"; `leaves` exits 0 with a file left in the folder.
struct Stubborn(
    SystemExec,
    std::collections::HashMap<&'static str, &'static str>,
);

impl GitExec for Stubborn {
    fn exec(&self, cwd: &str, args: &[String], timeout: Duration) -> GitOutput {
        let is_remove = args.first().map(String::as_str) == Some("worktree")
            && args.get(1).map(String::as_str) == Some("remove");
        let path = args.last().cloned().unwrap_or_default();
        let name = path
            .replace('\\', "/")
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .to_owned();
        match (is_remove, self.1.get(name.as_str())) {
            (true, Some(&"stuck")) => {
                let _ = std::fs::remove_file(Path::new(&path).join("a.txt"));
                let _ = std::fs::remove_file(Path::new(&path).join(".git"));
                GitOutput::failure(
                    128,
                    &format!("error: failed to delete '{path}': Directory not empty\n"),
                )
            }
            (true, Some(&"leaves")) => {
                let done = self.0.exec(cwd, args, timeout);
                let _ = std::fs::create_dir_all(&path);
                let _ = std::fs::write(
                    Path::new(&path).join("left-behind.txt"),
                    "git could not delete me\n",
                );
                done
            }
            _ => self.0.exec(cwd, args, timeout),
        }
    }
}

fn stubborn() -> Arc<dyn GitExec> {
    let modes = [("cli-stuck", "stuck"), ("cli-left", "leaves")]
        .into_iter()
        .collect();
    Arc::new(Stubborn(SystemExec::default(), modes))
}

/// A rig whose version control leaves some worktrees stubborn, with a new worktree `name` listed.
async fn stubborn_rig(name: &'static str) -> (Rig, std::path::PathBuf) {
    let mut r = Rig::with(Setup {
        git: Some(stubborn()),
        ..Setup::default()
    })
    .await;
    r.scanned().await;
    let path = r.fx.add(name);
    until("the new worktree in the listing", || async {
        r.docs().await.contains_key(name).then_some(())
    })
    .await;
    (r, path)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_worktree_git_could_not_delete_is_finished_by_argus_and_the_branch_goes() {
    let (r, path) = stubborn_rig("cli-stuck").await;
    let out = r
        .cli(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id_of(&path), "by": "ui", "branch": true, "reason": "merged long ago" }),
        )
        .await;
    assert_eq!(out.status, 200, "{}", out.text());
    let done = out.json();
    assert_eq!(done["finishedByArgus"], true);
    assert!(
        done["text"]
            .as_str()
            .unwrap()
            .contains("folder finished by Argus: git could not delete it"),
        "{done}"
    );
    gone(&path).await;
    row_gone(&r, "cli-stuck").await;
    let branches = git(&r.fx.repo, &["branch", "--format=%(refname:short)"], None);
    assert!(
        !branches.lines().any(|b| b == "cli-stuck"),
        "the branch went: {branches}"
    );
    until("the finished line", || async {
        r.activity("worktree-remove")
            .await
            .iter()
            .any(|t| t.contains("deleted worktree folder") && t.contains("finished by Argus"))
            .then_some(())
    })
    .await;
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_folder_git_left_behind_is_a_leftover_without_an_age_limit_for_the_batch() {
    let (r, path) = stubborn_rig("cli-left").await;
    let out = r
        .cli(
            "/api/p/alpha/worktrees/remove",
            &json!({ "id": id_of(&path), "by": "ui" }),
        )
        .await;
    assert_eq!(out.status, 200, "{}", out.text());
    assert_eq!(out.json()["folderLeft"], true);
    assert!(path.exists());
    until("the leftover row marked as made by a removal", || async {
        r.docs()
            .await
            .get("cli-left")
            .filter(|d| d["status"] == "leftover" && d["leftover"]["made"] == true)
            .cloned()
    })
    .await;
    let young = r.fx.leftover("cli-young", 0.0);
    until("the young leftover", || async {
        r.docs().await.contains_key("cli-young").then_some(())
    })
    .await;
    let run = r
        .cli(
            "/api/p/alpha/worktrees/leftovers/remove",
            &json!({ "by": "ui", "reason": "cleanup" }),
        )
        .await;
    assert_eq!(run.status, 200, "{}", run.text());
    assert_eq!(run.json()["started"], 1);
    gone(&path).await;
    assert!(young.exists(), "the young one nobody made stays");
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_threads_release_answers_at_once_with_a_request_the_worker_removes_and_refuses_one_with_work_in_it()
 {
    let r = Rig::start().await;
    r.scanned().await;
    let dirty = r
        .cli(
            "/api/p/alpha/worktrees/release",
            &json!({ "id": id_of(&r.path("dirty-old")), "by": "ui" }),
        )
        .await;
    assert_eq!((dirty.status, dirty.code().as_str()), (403, "forbidden"));
    assert!(
        dirty.json()["error"]["message"]
            .as_str()
            .unwrap()
            .contains("is not released: it has 1 uncommitted change")
    );
    let page = r
        .page(
            "/api/p/alpha/worktrees/release",
            &json!({ "id": id_of(&r.path("old-merged")) }),
        )
        .await;
    assert_eq!(page.status, 403, "release is a thread's word");
    let done = r
        .cli(
            "/api/p/alpha/worktrees/release",
            &json!({ "id": id_of(&r.path("old-merged")), "by": "ui" }),
        )
        .await;
    assert_eq!(done.status, 200, "{}", done.json());
    let answer = done.json();
    assert_eq!(
        (&answer["id"], &answer["status"]),
        (&json!("R-1"), &json!("queued")),
        "the answer is the request filed, not the removal"
    );
    assert_eq!(
        answer["text"],
        "R-1: Argus removes it now; the result comes to your inbox (coord worktree requests shows it)"
    );
    assert!(
        answer["removed"].is_null(),
        "nothing was removed before it answered"
    );
    gone(&r.path("old-merged")).await;
    let list = r.t.get("/api/p/alpha/worktrees/requests").await.json();
    assert_eq!(list["requests"][0]["id"], "R-1");
    assert_eq!(list["requests"][0]["branch"], true);
    assert!(r.path("dirty-old").exists());
    let removed = until("the removal in the summary", || async {
        let meta = r.meta().await;
        meta["hygiene"]["removed"][0]["path"]
            .as_str()
            .map(str::to_owned)
    })
    .await;
    assert!(removed.ends_with("/old-merged"), "{removed}");
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn shadow_mode_scans_and_shows_and_removes_nothing() {
    let r = Rig::with(Setup {
        shadow: true,
        ..Setup::default()
    })
    .await;
    let meta = r.scanned().await;
    assert_eq!(meta["count"], 5);
    let id = id_of(&r.path("old-merged"));
    for (path, body) in [
        ("/api/p/alpha/worktrees/remove", json!({ "id": id })),
        ("/api/p/alpha/worktrees/prune", json!({})),
        ("/api/p/alpha/worktrees/leftovers/remove", json!({})),
        ("/api/p/alpha/worktrees/release", json!({ "id": id })),
    ] {
        let refused = r.page(path, &body).await;
        assert_eq!(
            (refused.status, refused.code().as_str()),
            (403, "read_only"),
            "{path}"
        );
    }
    assert!(r.path("old-merged").exists());
    assert!(wakes_of(&r.t, "alpha", "worktree-stale").await.len() <= 1);
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_scanner_that_is_off_answers_503_and_leaves_no_service() {
    let r = Rig::with(Setup {
        enabled_off: true,
        ..Setup::default()
    })
    .await;
    assert!(r.t.svc.worktrees.is_none());
    let out = r
        .page("/api/p/alpha/worktrees/remove", &json!({ "id": "x" }))
        .await;
    assert_eq!((out.status, out.code().as_str()), (503, "unavailable"));
    let _ = rule(&r.fx);
    r.t.stop(false).await;
}
