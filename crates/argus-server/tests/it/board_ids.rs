//! Boards by id: an old key in an API path is served as the id, `/p/<alias>/` is a 302 to the id's
//! page, `GET /api/boards` lists table boards and legacy folders, and `PATCH /api/boards/:id`
//! renames either kind and refuses a bad name.
// test-tag: critical

use std::fs;

use argus_core::{Coll, DocChange};
use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::{Value, json};

const ID: &str = "b3k9x2";

fn write_board(t: &TestServer, folder: &str, board: &Value) {
    let dir = t.coord_home.join(folder);
    fs::create_dir_all(&dir).expect("board folder");
    fs::write(dir.join("board.json"), board.to_string()).expect("board file");
}

async fn rig() -> TestServer {
    let options = TestOptions {
        sync: false,
        ..TestOptions::default()
    };
    let t = TestServer::start(options).await;
    write_board(&t, ID, &json!({ "project": ID }));
    write_board(
        &t,
        "legacy",
        &json!({ "project": "legacy", "name": "Old Name" }),
    );
    let table = json!({
        "version": 1,
        "state": "active",
        "boards": {
            ID: {
                "name": "Alpha Project",
                "aliases": ["alpha"],
                "role": "argus",
                "createdAt": "2026-11-12T00:00:00.000Z"
            }
        }
    });
    fs::write(t.coord_home.join("boards.json"), table.to_string()).expect("table");
    t
}

async fn patch(t: &TestServer, path: &str, body: &Value) -> argus_testkit::Reply {
    t.send(Request::new("PATCH", path).json(body)).await
}

#[tokio::test(flavor = "multi_thread")]
async fn an_alias_in_an_api_path_is_served_as_the_id() {
    let t = rig().await;
    let doc = json!({ "id": "I-1", "title": "one" });
    t.svc
        .db
        .docs()
        .apply(ID, &[DocChange::set(Coll::Items, "I-1", doc)])
        .expect("doc");
    let by_alias = t.get("/api/p/alpha/db/items/I-1").await;
    let by_id = t.get(&format!("/api/p/{ID}/db/items/I-1")).await;
    assert_eq!(by_alias.json()["exists"], json!(true));
    assert_eq!(by_alias.json()["data"], by_id.json()["data"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_page_of_an_alias_is_a_302_to_the_page_of_the_id() {
    let t = rig().await;
    let moved = t.get("/p/alpha/?thread=lead").await;
    assert_eq!(moved.status, 302);
    assert_eq!(
        moved.header("location"),
        Some(format!("/p/{ID}/?thread=lead").as_str())
    );
    let bare = t.get("/p/alpha").await;
    assert_eq!(bare.header("location"), Some(format!("/p/{ID}/").as_str()));
    assert_eq!(t.get(&format!("/p/{ID}/")).await.status, 200);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_boards_list_has_the_table_and_the_legacy_folders() {
    let t = rig().await;
    let rows = t.get("/api/boards").await.json();
    assert_eq!(
        rows[0],
        json!({ "id": ID, "name": "Alpha Project", "aliases": ["alpha"], "role": "argus" })
    );
    assert_eq!(
        rows[1],
        json!({ "id": "legacy", "name": "Old Name", "aliases": [], "role": null })
    );
    assert_eq!(rows.as_array().map(Vec::len), Some(2));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_legacy_folder_without_a_board_name_shows_its_registry_name() {
    let t = rig().await;
    write_board(&t, "geo", &json!({ "project": "geo" }));
    write_board(
        &t,
        "_portfolio",
        &json!({ "projects": { "geo": { "key": "geo", "name": "ShardTide" } } }),
    );
    let rows = t.get("/api/boards").await.json();
    let geo = rows
        .as_array()
        .and_then(|r| r.iter().find(|b| b["id"] == "geo"))
        .cloned();
    assert_eq!(
        geo,
        Some(json!({ "id": "geo", "name": "ShardTide", "aliases": [], "role": null }))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_patch_renames_a_table_board_and_a_legacy_folder() {
    let t = rig().await;
    let done = patch(
        &t,
        &format!("/api/boards/{ID}"),
        &json!({ "name": "  Beta  " }),
    )
    .await;
    assert_eq!(done.status, 200);
    assert_eq!(done.json(), json!({ "id": ID, "name": "Beta" }));
    let legacy = patch(&t, "/api/boards/legacy", &json!({ "name": "New Name" })).await;
    assert_eq!(legacy.json(), json!({ "id": "legacy", "name": "New Name" }));
    let rows = t.get("/api/boards").await.json();
    assert_eq!(rows[0]["name"], json!("Beta"));
    assert_eq!(rows[1]["name"], json!("New Name"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_patch_refuses_a_long_name_and_an_unknown_board() {
    let t = rig().await;
    let long = "x".repeat(31);
    let refused = patch(&t, &format!("/api/boards/{ID}"), &json!({ "name": long })).await;
    assert_eq!(refused.status, 400);
    let unknown = patch(&t, "/api/boards/nowhere", &json!({ "name": "Fine" })).await;
    assert_eq!(unknown.status, 404);
}
