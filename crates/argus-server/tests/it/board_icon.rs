//! The board icon route: an image icon is served as its bytes with the cache and safety headers,
//! and a board with a brand, a curated name or no icon is a 404.
// test-tag: critical

use crate::posting::{assert_png_served, post, seeded};
use serde_json::json;

const PNG: &str = "data:image/png;base64,iVBORw0KGgo=";
const SVG: &str =
    "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciLz4=";

#[tokio::test(flavor = "multi_thread")]
async fn an_image_icon_is_served_with_its_type_and_the_safety_headers() {
    let t = seeded().await;
    post(&t, &json!({"kind": "icon", "icon": PNG})).await;
    let r = t.get("/api/p/alpha/icon").await;
    assert_eq!(r.status, 200);
    assert_png_served(&r);
    post(&t, &json!({"kind": "icon", "icon": SVG})).await;
    let svg = t.get("/api/p/alpha/icon?v=abcd1234").await;
    assert_eq!(svg.header("content-type"), Some("image/svg+xml"));
    assert!(svg.text().starts_with("<svg"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_board_without_an_image_icon_has_no_icon_route() {
    let t = seeded().await;
    assert_eq!(t.get("/api/p/alpha/icon").await.status, 404);
    post(&t, &json!({"kind": "icon", "icon": "icon:globe"})).await;
    assert_eq!(t.get("/api/p/alpha/icon").await.status, 404);
    assert_eq!(t.get("/api/p/nowhere/icon").await.status, 404);
}
