// test-tag: critical
//! The owner avatar route: an image avatar of the `_portfolio` board is served as its bytes with
//! the cache and safety headers, and no image (none, initials or a brand) is a 404.

use argus_testkit::{TestServer, write_board};
use serde_json::{Value, json};

use crate::posting::{assert_png_served, seeded};

const PNG: &str = "data:image/png;base64,iVBORw0KGgo=";

fn set_owner(t: &TestServer, owner: &Value) {
    let mut board = t.board("_portfolio");
    board["owner"] = owner.clone();
    write_board(&t.coord_home, "_portfolio", &board);
}

#[tokio::test(flavor = "multi_thread")]
async fn an_image_avatar_is_served_with_its_type_and_the_safety_headers() {
    let t = seeded().await;
    set_owner(&t, &json!({"name": "Ann", "avatar": PNG}));
    let r = t.get("/api/owner/avatar?v=abcd1234").await;
    assert_eq!(r.status, 200);
    assert_png_served(&r);
}

#[tokio::test(flavor = "multi_thread")]
async fn no_image_avatar_is_a_404() {
    let t = seeded().await;
    assert_eq!(t.get("/api/owner/avatar").await.status, 404);
    set_owner(&t, &json!({"avatar": null}));
    assert_eq!(t.get("/api/owner/avatar").await.status, 404);
    set_owner(&t, &json!({"avatar": "initials"}));
    assert_eq!(t.get("/api/owner/avatar").await.status, 404);
    set_owner(&t, &json!({"avatar": "icon:globe"}));
    assert_eq!(t.get("/api/owner/avatar").await.status, 404);
}
