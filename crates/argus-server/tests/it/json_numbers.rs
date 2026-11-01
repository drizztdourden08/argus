//! A number with no fraction is written without one wherever a doc or an answer leaves the
//! server, as Node does: `meta.leftoverOld.days` is `1`, not `1.0`. The stored doc, the doc
//! route, the list route and the page stream all carry the one rule of `argus_core::json_text`.
//! K2: whole numbers carry no fraction in docs, answers and the page stream.
// test-tag: standard

use argus_core::{Coll, DocChange};
use argus_testkit::{TestOptions, TestServer};
use serde_json::{Number, Value, json};

fn float(n: f64) -> Value {
    Value::Number(Number::from_f64(n).expect("a finite number"))
}

fn meta() -> Value {
    json!({ "leftoverOld": { "count": 2, "days": float(1.0), "sizeMb": float(12.5) }, "ratio": float(3.0) })
}

async fn served() -> TestServer {
    let t = TestServer::start(TestOptions {
        sync: false,
        ..TestOptions::default()
    })
    .await;
    let change = DocChange::set(Coll::Meta, "worktrees", meta());
    t.svc
        .db
        .docs()
        .apply("alpha", &[change])
        .expect("the doc is stored");
    t
}

fn assert_whole(text: &str) {
    assert!(text.contains(r#""days":1,"#), "{text}");
    assert!(text.contains(r#""ratio":3"#), "{text}");
    assert!(text.contains(r#""sizeMb":12.5"#), "{text}");
    assert!(!text.contains("1.0") && !text.contains("3.0"), "{text}");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_doc_route_and_the_list_route_write_whole_numbers_without_a_fraction() {
    let t = served().await;
    assert_whole(&t.get("/api/p/alpha/db/meta/worktrees").await.text());
    assert_whole(&t.get("/api/p/alpha/db/meta").await.text());
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_stored_text_and_the_page_stream_do_too() {
    let t = served().await;
    let stored: String = t
        .svc
        .db
        .with_conn(|c| {
            c.query_row(
                "SELECT data FROM docs WHERE scope = 'alpha' AND coll = 'meta' AND id = 'worktrees'",
                [],
                |row| row.get(0),
            )
        })
        .expect("the stored row");
    assert_whole(&stored);
    let stream = t.sse("/api/p/alpha/stream?c=meta").await;
    assert!(stream.next(Some("ready"), 2000).await.is_some());
    assert_whole(&stream.raw());
    let live = DocChange::set(Coll::Meta, "live", meta());
    t.svc.db.docs().apply("alpha", &[live]).expect("applied");
    assert!(stream.next(Some("changes"), 2000).await.is_some());
    let raw = stream.raw();
    let changes = raw.split("event: changes").nth(1).expect("a changes frame");
    assert_whole(changes);
    t.stop(false).await;
}
