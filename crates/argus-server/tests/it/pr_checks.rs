//! Local pull requests over HTTP, on temp homes: the PR routes, their refusals and the write rules,
//! and the refusal of bad check arguments before anything runs. K2, K4: the route shapes and the
//! refusals. The runs of a check, which start the fake Brock as a real process, are in
//! `pr_checks_runs.rs`.
// test-tag: standard

use std::path::PathBuf;

use crate::pr_rig::{open_pr, pr_body, rig};
use argus_testkit::Request;
use serde_json::json;

const MERGE: &str = "fedcba9876543210fedcba9876543210fedcba98";

#[tokio::test(flavor = "multi_thread")]
async fn a_pr_is_opened_updated_merged_and_closed_through_the_routes() {
    let r = rig(false).await;
    let opened = open_pr(&r.t).await;
    assert_eq!(opened["outcome"], "PR-1 opened");
    let pr = &opened["pr"];
    assert_eq!(
        (pr["id"].as_str(), pr["status"].as_str()),
        (Some("PR-1"), Some("open"))
    );
    assert_eq!(pr["base"], "main");
    assert_eq!(
        pr["replay"],
        json!({ "state": "pending", "at": null, "url": null })
    );
    assert_eq!(pr["createdBy"], "cli");
    assert_eq!(r.t.board("alpha")["prs"]["PR-1"]["item"], "I-1");
    let fresh = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let update = json!({ "headCommit": fresh, "title": "Add x v2" });
    let updated = r.t.write("PATCH", "/api/p/alpha/prs/PR-1", &update).await;
    assert_eq!(updated.status, 200, "{}", updated.text());
    assert_eq!(updated.json()["pr"]["headCommit"], fresh);
    let merge = json!({ "action": "merge", "mergeCommit": MERGE, "by": "brock" });
    let merged = r.t.write("PATCH", "/api/p/alpha/prs/PR-1", &merge).await;
    assert_eq!(merged.status, 200, "{}", merged.text());
    let pr = merged.json()["pr"].clone();
    assert_eq!(
        (pr["status"].as_str(), pr["mergeCommit"].as_str()),
        (Some("merged"), Some(MERGE))
    );
    assert_eq!(pr["mergedBy"], "brock");
    let again = r.t.write("PATCH", "/api/p/alpha/prs/PR-1", &merge).await;
    assert_eq!(
        again.status, 400,
        "a merged pull request does not merge again"
    );
    let mirrored = r.t.svc.db.docs().list("alpha", argus_core::Coll::Prs);
    assert_eq!(
        mirrored.expect("prs docs").len(),
        1,
        "the board stream carries it"
    );
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_pr_can_be_closed_and_the_routes_refuse_what_they_should() {
    let r = rig(false).await;
    open_pr(&r.t).await;
    let closed =
        r.t.write(
            "PATCH",
            "/api/p/alpha/prs/PR-1",
            &json!({ "action": "close" }),
        )
        .await;
    assert_eq!(closed.json()["pr"]["status"], "closed");
    let missing =
        r.t.write(
            "PATCH",
            "/api/p/alpha/prs/PR-9",
            &json!({ "action": "close" }),
        )
        .await;
    assert_eq!(missing.status, 404);
    let odd =
        r.t.write(
            "PATCH",
            "/api/p/alpha/prs/PR-1",
            &json!({ "action": "burn" }),
        )
        .await;
    assert_eq!(odd.status, 400);
    let untitled =
        r.t.write("POST", "/api/p/alpha/prs", &json!({ "head": "feat/y" }))
            .await;
    assert_eq!(untitled.status, 400);
    let evil = json!({ "title": "t", "head": "a b; rm -rf" });
    assert_eq!(
        r.t.write("POST", "/api/p/alpha/prs", &evil).await.status,
        400
    );
    let nowhere = r.t.write("POST", "/api/p/nowhere/prs", &pr_body()).await;
    assert_eq!(nowhere.status, 404);
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_pr_routes_follow_the_write_rules_of_the_other_routes() {
    let r = rig(false).await;
    let page =
        r.t.page(Request::new("POST", "/api/p/alpha/prs").json(&pr_body()));
    let got = r.t.send(page).await;
    assert_eq!(got.status, 200, "{}", got.text());
    assert_eq!(got.json()["pr"]["createdBy"], "human");
    let foreign = Request::new("POST", "/api/p/alpha/prs")
        .header("Origin", "http://evil.test")
        .header("Sec-Fetch-Site", "cross-site")
        .json(&pr_body());
    assert_eq!(r.t.send(foreign).await.status, 403);
    r.t.stop(false).await;
    let shadow = rig(true).await;
    let refused = shadow.t.write("POST", "/api/p/alpha/prs", &pr_body()).await;
    assert_eq!(
        (refused.status, refused.code().as_str()),
        (403, "read_only")
    );
    let start = shadow
        .t
        .write("POST", "/api/p/alpha/checks", &json!({}))
        .await;
    assert_eq!(start.status, 403);
    shadow.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_check_run_refuses_bad_arguments_and_a_project_without_a_root() {
    let r = rig(false).await;
    for body in [
        json!({ "commit": "zz" }),
        json!({ "commit": "abc1234; rm" }),
        json!({ "pr": "3" }),
        json!({ "job": "all && x" }),
        json!({ "trigger": "now" }),
        json!({ "check": "a b" }),
        json!({ "check": "--x" }),
        json!({ "commit": 5 }),
    ] {
        let got = r.t.write("POST", "/api/p/alpha/checks", &body).await;
        assert_eq!(got.status, 400, "{body}: {}", got.text());
    }
    let no_root = r.t.write("POST", "/api/p/beta/checks", &json!({})).await;
    assert_eq!(no_root.status, 404, "{}", no_root.text());
    assert!(no_root.text().contains("root"), "{}", no_root.text());
    let none: Vec<PathBuf> = std::fs::read_dir(r.out.path())
        .expect("out")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.ends_with("argv.json"))
        .collect();
    assert!(none.is_empty(), "nothing ran");
    r.t.stop(false).await;
}
