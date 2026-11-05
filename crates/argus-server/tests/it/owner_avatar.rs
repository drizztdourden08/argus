// test-tag: critical
//! The owner avatar route: an image avatar of the `_portfolio` board is served as its bytes with
//! the cache and safety headers, and no image (none, initials or a brand) is a 404.

use argus_testkit::{TestServer, write_board};
use serde_json::{Value, json};

use crate::posting::{assert_png_served, seeded};

const PNG: &str = "data:image/png;base64,iVBORw0KGgo=";

/// Writes the owner into the `_portfolio` board file and resyncs it, so the server reads it.
async fn set_owner(t: &TestServer, owner: &Value) {
    let mut board = t.board("_portfolio");
    board["owner"] = owner.clone();
    write_board(&t.coord_home, "_portfolio", &board);
    let done = t
        .write(
            "POST",
            "/api/admin/resync",
            &json!({ "project": "_portfolio" }),
        )
        .await;
    assert_eq!(done.status, 200, "{}", done.text());
}

#[tokio::test(flavor = "multi_thread")]
async fn an_image_avatar_is_served_with_its_type_and_the_safety_headers() {
    let t = seeded().await;
    set_owner(&t, &json!({"name": "Ann", "avatar": PNG})).await;
    let r = t.get("/api/owner/avatar?v=abcd1234").await;
    assert_eq!(r.status, 200);
    assert_png_served(&r);
}

#[tokio::test(flavor = "multi_thread")]
async fn no_image_avatar_is_a_404() {
    let t = seeded().await;
    assert_eq!(t.get("/api/owner/avatar").await.status, 404);
    set_owner(&t, &json!({"avatar": null})).await;
    assert_eq!(t.get("/api/owner/avatar").await.status, 404);
    set_owner(&t, &json!({"avatar": "initials"})).await;
    assert_eq!(t.get("/api/owner/avatar").await.status, 404);
    set_owner(&t, &json!({"avatar": "icon:globe"})).await;
    assert_eq!(t.get("/api/owner/avatar").await.status, 404);
}
