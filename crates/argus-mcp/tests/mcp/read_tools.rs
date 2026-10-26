//! Every read tool against the fixture boards: shapes, filters, sorting and the argument rules.

use argus_mcp::{MemoryBackend, RpcHost};
use serde_json::{Value, json};

use crate::support::{Rig, ctx, fixture, ids, run_lines, run_rows, sorted_ids};

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

#[tokio::test]
async fn health_and_portfolio_overview() {
    let rig = &Rig::new();
    let health = rig.ok("health", json!({})).await;
    assert_eq!(health["ok"], true);
    assert_eq!(health["scopes"], json!(["alpha", "beta"]));
    let cards = rig.ok("portfolio_overview", json!({})).await;
    assert_eq!(sorted_ids(&cards, "cards"), strings(&["alpha", "beta"]));
}

#[tokio::test]
async fn board_summary_builds_the_shape_and_adds_runs_and_the_handed_build() {
    let rig = &Rig::new();
    let s = rig.ok("board_summary", json!({ "project": "alpha" })).await;
    assert_eq!(s["project"], "alpha");
    assert_eq!(s["handedBuild"], "b1");
    assert_eq!(s["runs"], json!({ "running": 1, "queued": 0 }));
    assert_eq!(s["openQuestions"], json!(["Q-1"]));
    assert_eq!(s["pendingDecisions"], json!(["D-1"]));
    assert_eq!(s["pendingActions"], json!(["H-1"]));
    let i1 = s["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|i| i["id"] == "I-1");
    assert_eq!(i1.expect("I-1")["ac"], "1/2");
    assert!(
        !ids(&s, "items").iter().any(|id| id == "I-3" || id == "I-4"),
        "archived and deleted items stay out"
    );
    let coordinator = s["threads"]
        .as_array()
        .expect("threads")
        .iter()
        .find(|t| t["id"] == "coordinator");
    assert_eq!(coordinator.expect("coordinator")["agents"], 1);
    for dropped in ["artifact", "dirty", "inbox"] {
        assert!(s.get(dropped).is_none(), "{dropped}");
    }
}

#[tokio::test]
async fn board_summary_keeps_the_node_key_order() {
    let rig = &Rig::new();
    let s = rig.ok("board_summary", json!({ "project": "alpha" })).await;
    let keys: Vec<&str> = s
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    let want = [
        "project",
        "updatedAt",
        "threads",
        "items",
        "openQuestions",
        "pendingDecisions",
        "handedBuild",
        "tests",
        "pendingActions",
        "locks",
        "runs",
    ];
    assert_eq!(keys, want);
}

#[tokio::test]
async fn list_threads_hides_archived_and_paused_unless_asked() {
    let rig = &Rig::new();
    let shown = rig.ok("list_threads", json!({ "project": "alpha" })).await;
    assert_eq!(
        sorted_ids(&shown, "threads"),
        strings(&["coordinator", "story"])
    );
    let all = rig
        .ok(
            "list_threads",
            json!({ "project": "alpha", "includeHidden": true }),
        )
        .await;
    assert_eq!(ids(&all, "threads").len(), 4);
}

#[tokio::test]
async fn list_items_filters_hides_deleted_and_shows_archived_on_request() {
    let rig = &Rig::new();
    let list = |args: Value| async move { ids(&rig.ok("list_items", args).await, "items") };
    assert_eq!(
        list(json!({ "project": "alpha" })).await,
        strings(&["I-1", "I-2"])
    );
    assert_eq!(
        list(json!({ "project": "alpha", "status": "todo" })).await,
        strings(&["I-2"])
    );
    let archived = json!({ "project": "alpha", "thread": "story", "includeArchived": true });
    assert_eq!(list(archived).await, strings(&["I-2", "I-3"]));
    assert_eq!(
        list(json!({ "project": "alpha", "limit": 1 })).await.len(),
        1
    );
    let first = rig.ok("list_items", json!({ "project": "alpha" })).await;
    assert_eq!(first["items"][0]["ac"], "1/2");
    assert_eq!(first["items"][0]["rev"], 1);
}

#[tokio::test]
async fn get_item_returns_tests_entries_without_op_plumbing_and_actions() {
    let rig = &Rig::new();
    let r = rig
        .ok("get_item", json!({ "project": "alpha", "id": "I-1" }))
        .await;
    assert_eq!(r["item"]["title"], "Ship the pedestal fix");
    assert_eq!(ids(&r, "tests"), strings(&["T-1", "T-2"]));
    assert_eq!(ids(&r, "entries"), strings(&["D-1", "Q-1"]));
    assert_eq!(ids(&r, "actions"), strings(&["H-1"]));
    rig.err(
        "get_item",
        json!({ "project": "alpha", "id": "I-99" }),
        "not_found",
    )
    .await;
}

#[tokio::test]
async fn list_tests_and_get_test_make_proof_urls_absolute() {
    let rig = &Rig::new();
    let list = |args: Value| async move { ids(&rig.ok("list_tests", args).await, "tests") };
    assert_eq!(
        list(json!({ "project": "alpha", "mode": "manual" })).await,
        strings(&["T-1", "T-3"])
    );
    assert_eq!(
        list(json!({ "project": "alpha", "build": "b2" })).await,
        strings(&["T-3"])
    );
    assert_eq!(
        list(json!({ "project": "alpha", "status": "pass" })).await,
        strings(&["T-2"])
    );
    assert_eq!(
        list(json!({ "project": "alpha", "item": "I-2" })).await,
        strings(&["T-3"])
    );
    let t = rig
        .ok("get_test", json!({ "project": "alpha", "id": "T-1" }))
        .await;
    assert_eq!(
        t["test"]["proofs"][0]["url"],
        "http://127.0.0.1:47615/files/alpha/proofs/T-1-1.png"
    );
    assert_eq!(t["test"]["steps"], json!(["Walk in."]));
}

#[tokio::test]
async fn list_agents_joins_runs_and_filters_live_and_thread() {
    let rig = &Rig::new();
    let all = rig.ok("list_agents", json!({ "project": "alpha" })).await;
    assert_eq!(sorted_ids(&all, "agents"), strings(&["A-1", "A-2"]));
    let builder = all["agents"]
        .as_array()
        .expect("agents")
        .iter()
        .find(|a| a["id"] == "A-1");
    assert_eq!(ids(builder.expect("A-1"), "runs"), strings(&["R-1"]));
    let live = rig
        .ok("list_agents", json!({ "project": "alpha", "live": true }))
        .await;
    assert_eq!(ids(&live, "agents"), strings(&["A-1"]));
    let story = rig
        .ok(
            "list_agents",
            json!({ "project": "alpha", "thread": "story" }),
        )
        .await;
    assert_eq!(ids(&story, "agents"), strings(&["A-2"]));
}

#[tokio::test]
async fn list_launchers_puts_master_first_and_joins_play_and_bundles() {
    let rig = &Rig::new();
    let names = |r: &Value| -> Vec<String> {
        r["launchers"]
            .as_array()
            .expect("launchers")
            .iter()
            .filter_map(|l| l["name"].as_str().map(str::to_owned))
            .collect()
    };
    let r = rig
        .ok("list_launchers", json!({ "project": "alpha" }))
        .await;
    assert_eq!(names(&r), strings(&["master", "build-1"]));
    assert_eq!(r["master"], true);
    let b1 = &r["launchers"][1];
    let got = json!({ "play": b1["play"], "bundles": b1["bundles"], "cmd": b1["cmd"], "current": b1["current"] });
    assert_eq!(
        got,
        json!({ "play": "launcher-build-1", "bundles": ["b1"], "cmd": "game run --wt build-1", "current": true })
    );
    assert_eq!(r["launchers"][0]["play"], "launcher-master");
    let all = rig
        .ok(
            "list_launchers",
            json!({ "project": "alpha", "includeRetired": true }),
        )
        .await;
    assert_eq!(names(&all), strings(&["master", "build-1", "old-1"]));
    assert_eq!(
        all["launchers"][2]["play"],
        Value::Null,
        "a retired launcher has no Play"
    );
    let none = rig.ok("list_launchers", json!({ "project": "beta" })).await;
    assert_eq!(none, json!({ "launchers": [], "master": false }));
    rig.err(
        "list_launchers",
        json!({ "project": "alpha", "extra": 1 }),
        "bad_request",
    )
    .await;
}

#[tokio::test]
async fn list_entries_leaves_op_entries_out_and_filters_open_and_kind() {
    let rig = &Rig::new();
    let list = |args: Value| async move { ids(&rig.ok("list_entries", args).await, "entries") };
    assert_eq!(
        list(json!({ "project": "alpha" })).await,
        strings(&["D-1", "Q-1"])
    );
    assert_eq!(
        list(json!({ "project": "alpha", "open": true })).await,
        strings(&["D-1", "Q-1"])
    );
    assert_eq!(
        list(json!({ "project": "alpha", "kind": "question" })).await,
        strings(&["Q-1"])
    );
    assert_eq!(
        list(json!({ "project": "alpha", "kind": "edit" })).await,
        strings(&["O-m1x2-ab12"])
    );
    let q = rig
        .ok(
            "list_entries",
            json!({ "project": "alpha", "kind": "question" }),
        )
        .await;
    assert!(
        q["entries"][0]["context"]
            .as_str()
            .is_some_and(|c| c.starts_with("A visual polish choice"))
    );
    assert_eq!(
        q["entries"][0]["options"],
        json!(["Add the glow", "Keep it plain"])
    );
}

async fn hits(rig: &Rig, args: Value) -> Vec<Value> {
    let found = rig.ok("search", args).await;
    found["hits"].as_array().expect("hits").clone()
}

#[tokio::test]
async fn search_across_projects_and_inside_one() {
    let rig = &Rig::new();
    let all = hits(rig, json!({ "query": "pedestal" })).await;
    let mut projects: Vec<&str> = all.iter().filter_map(|h| h["project"].as_str()).collect();
    projects.sort_unstable();
    projects.dedup();
    assert_eq!(projects, ["alpha", "beta"]);
    assert!(all.iter().all(|h| {
        h["snippet"]
            .as_str()
            .is_some_and(|s| s.to_lowercase().contains("pedestal"))
            && h["title"].is_string()
    }));
    let one = hits(rig, json!({ "query": "pedestal", "project": "beta" })).await;
    let named: Vec<String> = one
        .iter()
        .map(|h| {
            format!(
                "{}/{}/{}",
                h["project"].as_str().unwrap_or(""),
                h["coll"].as_str().unwrap_or(""),
                h["id"].as_str().unwrap_or("")
            )
        })
        .collect();
    assert_eq!(named, ["beta/items/I-1"]);
}

#[tokio::test]
async fn search_takes_collections_finds_archived_hides_deleted_and_scopes_to_the_default() {
    let rig = &Rig::new();
    let log = json!({ "query": "glow shader", "project": "alpha", "collections": ["log"] });
    assert_eq!(hits(rig, log).await.len(), 1);
    let archived = hits(rig, json!({ "query": "Archived scene" })).await;
    assert_eq!(archived.len(), 1, "archived items are searchable");
    let deleted = hits(rig, json!({ "query": "Deleted scene" })).await;
    assert_eq!(deleted.len(), 0, "deleted ones are not");
    let beta = argus_mcp::CallContext {
        default_project: Some("beta".into()),
        ..ctx()
    };
    let scoped = rig
        .ok_as("search", json!({ "query": "pedestal" }), &beta)
        .await;
    assert_eq!(
        scoped["hits"].as_array().expect("hits").len(),
        1,
        "the default project scopes it"
    );
}

#[tokio::test]
async fn a_hit_has_the_keys_project_coll_id_title_snippet_in_that_order() {
    let rig = &Rig::new();
    let r = rig
        .ok(
            "search",
            json!({ "query": "harbor", "project": "alpha", "collections": ["items"] }),
        )
        .await;
    let keys: Vec<&str> = r["hits"][0]
        .as_object()
        .expect("a hit")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["project", "coll", "id", "title", "snippet"]);
}

#[tokio::test]
async fn run_status_hides_env_values_and_run_logs_follow_a_cursor() {
    let rig = &Rig::new();
    let s = rig.ok("run_status", json!({ "project": "alpha" })).await;
    assert_eq!(s["defs"][0]["name"], "hello");
    assert!(s["defs"][0].get("env").is_none());
    assert_eq!(s["defs"][0]["envKeys"], json!(["SECRET_TOKEN"]));
    assert!(!s.to_string().contains("abc"));
    assert_eq!(s["runs"].as_array().expect("runs").len(), 2);
    let running = rig
        .ok(
            "run_status",
            json!({ "project": "alpha", "status": "running" }),
        )
        .await;
    assert_eq!(ids(&running, "runs"), strings(&["R-2"]));
    let one = rig
        .ok("run_status", json!({ "project": "alpha", "run": "R-1" }))
        .await;
    assert_eq!(one["run"]["status"], "exited");
    rig.err(
        "run_status",
        json!({ "project": "alpha", "run": "R-9" }),
        "not_found",
    )
    .await;
    let logs = rig
        .ok("run_logs", json!({ "project": "alpha", "run": "R-1" }))
        .await;
    assert_eq!(logs["lines"][0]["text"], "hi\n");
    assert_eq!(logs["lines"][1]["text"], "there\n");
    assert_eq!(logs["status"], "exited");
    let after = rig
        .ok(
            "run_logs",
            json!({ "project": "alpha", "run": "R-1", "after": 1 }),
        )
        .await;
    assert_eq!(after["lines"].as_array().expect("lines").len(), 1);
}

#[tokio::test]
async fn run_status_defs_come_from_the_launches_docs_without_a_sync_definition_list() {
    let rig = Rig::over(MemoryBackend::new(fixture()).with_runs(run_rows(), run_lines()));
    let s = rig.ok("run_status", json!({ "project": "alpha" })).await;
    assert_eq!(s["defs"][0]["name"], "hello");
    assert_eq!(s["defs"][0]["envKeys"], json!([]));
}

#[tokio::test]
async fn argument_problems_unknown_projects_and_the_default_project() {
    let rig = &Rig::new();
    let bad = |name: &'static str, args: Value, code: &'static str| rig.err(name, args, code);
    bad("list_items", json!({}), "bad_request").await;
    bad("list_items", json!({ "project": "Bad Key" }), "bad_request").await;
    bad("list_items", json!({ "project": "nope" }), "not_found").await;
    bad("list_items", json!({ "project": "gamma" }), "not_found").await;
    bad(
        "list_items",
        json!({ "project": "alpha", "limit": 9999 }),
        "bad_request",
    )
    .await;
    bad(
        "list_items",
        json!({ "project": "alpha", "status": "weird" }),
        "bad_request",
    )
    .await;
    bad(
        "list_items",
        json!({ "project": "alpha", "surprise": 1 }),
        "bad_request",
    )
    .await;
    bad("get_item", json!({ "project": "alpha" }), "bad_request").await;
    bad("no_such_tool", json!({}), "unknown_tool").await;
    let beta = argus_mcp::CallContext {
        default_project: Some("beta".into()),
        ..ctx()
    };
    let from_default = rig.ok_as("list_items", json!({}), &beta).await;
    assert_eq!(
        from_default["items"][0]["title"],
        "Pedestal in the beta game"
    );
    let named = rig
        .ok_as("list_items", json!({ "project": "alpha" }), &beta)
        .await;
    assert_eq!(named["items"][0]["id"], "I-1");
}

#[tokio::test]
async fn an_unknown_project_lists_the_known_ones_in_the_details() {
    let rig = &Rig::new();
    let e = rig
        .err("list_items", json!({ "project": "nope" }), "not_found")
        .await;
    assert_eq!(e["message"], "no board for project \"nope\"");
    assert_eq!(e["details"], json!({ "known": ["alpha", "beta"] }));
}

#[tokio::test]
async fn the_schema_messages_are_the_node_ones() {
    let rig = &Rig::new();
    let message = |args: Value| async move {
        rig.err("list_items", args, "bad_request").await["message"]
            .as_str()
            .unwrap_or("")
            .to_owned()
    };
    assert_eq!(
        message(json!({ "project": "alpha", "limit": 9999 })).await,
        "arguments.limit must be at most 500"
    );
    assert_eq!(
        message(json!({ "project": "alpha", "status": "weird" })).await,
        "arguments.status must be one of todo, doing, review, done, dropped"
    );
    assert_eq!(
        message(json!({ "project": "alpha", "surprise": 1 })).await,
        "arguments.surprise is not a known argument"
    );
    assert_eq!(
        message(json!({ "project": 5 })).await,
        "arguments.project must be a string"
    );
    assert_eq!(
        message(json!({ "project": "alpha", "limit": 1.5 })).await,
        "arguments.limit must be an integer"
    );
    assert_eq!(
        message(json!({ "project": "alpha", "limit": 0, "status": "x" })).await,
        "arguments.limit must be at least 1; arguments.status must be one of todo, doing, review, done, dropped"
    );
    let get = rig
        .err(
            "get_item",
            json!({ "project": "alpha", "id": "x" }),
            "bad_request",
        )
        .await;
    assert_eq!(get["message"], "arguments.id does not match ^I-\\d+$");
    let none = rig.host.call_tool("get_item", &json!(null), &ctx()).await;
    assert_eq!(none.result["error"]["message"], "arguments.id is required");
}

#[tokio::test]
async fn services_that_are_down_answer_unavailable() {
    let bare = Rig::over(MemoryBackend::new(fixture()).without_sync());
    let s = bare
        .ok("board_summary", json!({ "project": "alpha" }))
        .await;
    assert_eq!(s["project"], "alpha", "reads still work from the docs");
    bare.err(
        "post_entry",
        json!({ "project": "alpha", "kind": "followup", "text": "x" }),
        "unavailable",
    )
    .await;
    bare.err(
        "run_start",
        json!({ "project": "alpha", "name": "hello" }),
        "unavailable",
    )
    .await;
    bare.err(
        "run_logs",
        json!({ "project": "alpha", "run": "R-1" }),
        "unavailable",
    )
    .await;
    let status = bare.ok("run_status", json!({ "project": "alpha" })).await;
    assert_eq!(status["runs"], json!([]));
}
