//! The item image upload: kept on the item, served at the url it answers, listed in the item's
//! `images`, the same bytes once, and refusals for a bad type, a bad id and a missing item.
// test-tag: critical

use crate::posting::{doc, seeded};
use argus_testkit::{Request, png};
use serde_json::json;

fn upload(path: &str) -> Request {
    Request::new("POST", path)
        .header("Content-Type", "image/png")
        .body(png())
}

#[tokio::test(flavor = "multi_thread")]
async fn an_item_image_is_kept_listed_and_served() {
    let t = seeded().await;
    let r = t
        .send(upload("/api/p/alpha/items/I-1/images?caption=the%20form"))
        .await
        .json();
    let url = "/files/alpha/proofs/I-1-1.png";
    let image = json!({ "n": 1, "file": "I-1-1.png", "url": url, "caption": "the form" });
    assert_eq!(r, json!({ "item": "I-1", "image": image }));
    let stored = doc(&t, "alpha", "items", "I-1").await;
    assert_eq!(stored["images"][0]["url"], url);
    assert_eq!(stored["images"][0]["caption"], "the form");
    let img = t.get(url).await;
    assert_eq!((img.status, img.body), (200, png()));
    let again = t
        .send(upload("/api/p/alpha/items/I-1/images?caption=again"))
        .await
        .json();
    assert_eq!(again["image"]["n"], 1);
    assert_eq!(
        doc(&t, "alpha", "items", "I-1").await["images"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_item_image_is_refused_for_a_bad_type_a_bad_id_or_a_missing_item() {
    let t = seeded().await;
    let text = Request::new("POST", "/api/p/alpha/items/I-1/images")
        .header("Content-Type", "text/plain")
        .body("hello");
    assert_eq!(t.send(text).await.status, 415);
    let bad_id = upload("/api/p/alpha/items/X-1/images?caption=x");
    assert_eq!(t.send(bad_id).await.status, 400);
    let missing = upload("/api/p/alpha/items/I-99/images?caption=x");
    assert_eq!(t.send(missing).await.status, 404);
}
