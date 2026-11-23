// test-tag: critical
//! The Overlord transcript routes: the owner's page only, lines after an index, the newest file
//! when no session is named, and a live stream told by the feed.

use argus_overlord::ports::TranscriptStore;
use argus_server::adapters::overlord::feed::FedLine;
use argus_server::adapters::overlord::transcript::FileTranscripts;
use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::json;

async fn start() -> TestServer {
    TestServer::start(TestOptions::default()).await
}

fn seed(t: &TestServer, session: &str, count: usize) {
    let store = FileTranscripts::new(&t.dashboard_home);
    for n in 0..count {
        store
            .append(session, &json!({ "n": n }).to_string())
            .unwrap();
    }
}

fn read(path: &str) -> Request {
    Request::new("GET", path)
}

#[tokio::test(flavor = "multi_thread")]
async fn lines_come_after_the_index_and_a_foreign_origin_is_refused() {
    let t = start().await;
    seed(&t, "s1", 3);
    let all = t
        .send(t.page(read("/api/overlord/transcript?session=s1")))
        .await;
    assert_eq!(all.status, 200, "{}", all.text());
    assert_eq!(all.json()["session"], json!("s1"));
    assert_eq!(
        all.json()["lines"],
        json!([{ "n": 0 }, { "n": 1 }, { "n": 2 }])
    );
    assert_eq!(all.json()["next"], json!(3));
    let rest = t
        .send(t.page(read("/api/overlord/transcript?session=s1&after=2")))
        .await;
    assert_eq!(rest.json()["lines"], json!([{ "n": 2 }]));
    let foreign = read("/api/overlord/transcript?session=s1").header("Origin", "http://evil.test");
    assert_eq!(t.send(foreign).await.status, 403);
    let bad = t
        .send(t.page(read("/api/overlord/transcript?session=..%2Fx")))
        .await;
    assert_eq!(bad.status, 400);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn no_session_means_the_newest_file_or_an_empty_answer() {
    let t = start().await;
    let none = t.send(t.page(read("/api/overlord/transcript"))).await;
    assert_eq!(
        none.json(),
        json!({ "session": null, "lines": [], "next": 0 })
    );
    seed(&t, "old", 1);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    seed(&t, "new", 2);
    let newest = t.send(t.page(read("/api/overlord/transcript"))).await;
    assert_eq!(newest.json()["session"], json!("new"));
    assert_eq!(newest.json()["next"], json!(2));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_stream_replays_then_follows_the_feed() {
    let t = start().await;
    seed(&t, "s1", 2);
    let stream = t
        .sse_with(t.page(read("/api/overlord/transcript/stream?session=s1&after=1")))
        .await;
    let first = stream
        .next(Some("line"), 5000)
        .await
        .expect("the stored line");
    assert_eq!(first.data["index"], json!(2));
    assert_eq!(first.data["event"], json!({ "n": 1 }));
    t.svc.overlord_feed.publish(&FedLine {
        session: "other".to_owned(),
        index: 1,
        line: "{}".to_owned(),
        stored: true,
    });
    t.svc.overlord_feed.publish(&FedLine {
        session: "s1".to_owned(),
        index: 3,
        line: json!({ "n": 2 }).to_string(),
        stored: true,
    });
    let live = stream
        .next(Some("line"), 5000)
        .await
        .expect("the live line");
    assert_eq!(live.data["index"], json!(3));
    assert_eq!(live.data["event"], json!({ "n": 2 }));
    stream.close();
    t.stop(false).await;
}
