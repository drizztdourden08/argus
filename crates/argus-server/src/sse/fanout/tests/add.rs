//! Adding and removing collections on a client that is already streaming.
// test-tag: critical

use serde_json::json;

use super::*;

/// A mux client of `alpha` that follows `meta`, with its opening frames read.
async fn following_meta(r: &Rig) -> (Handle, Wire) {
    r.set(Coll::Meta, "board");
    let mut w = wire();
    let target = Target::New(Kind::Mux, &w.sender);
    let h = r
        .fanout
        .subscribe(&r.db, ALPHA, target, &[Coll::Meta], None);
    let h = h.expect("subscribed");
    assert_eq!(names(&w.frames().await), ["snapshot", "ready"]);
    (h, w)
}

#[tokio::test]
async fn adding_a_collection_mid_stream_has_no_gap_and_no_duplicate() {
    let r = rig();
    r.set(Coll::Items, "I-1");
    let (h, mut w) = following_meta(&r).await;
    let (token, fresh) = r.fanout.begin(&h, Target::Add(&h), &[Coll::Items]).unwrap();
    r.set(Coll::Items, "I-2");
    let first = first_frames(&r.db, ALPHA, &fresh, None).unwrap();
    r.set(Coll::Items, "I-3");
    r.fanout.finish(&h, token, &first);
    r.set(Coll::Items, "I-4");
    let got = w.frames().await;
    assert_eq!(names(&got), ["snapshot", "changes", "ready", "changes"]);
    assert_eq!(ids(&got[0].1), ["I-1", "I-2"]);
    assert_eq!(ids(&got[1].1), ["I-3"]);
    assert_eq!(got[2].1["seq"], first.seq);
    assert_eq!(ids(&got[3].1), ["I-4"]);
}

#[tokio::test]
async fn an_event_at_or_below_the_read_seq_is_not_sent_again() {
    let r = rig();
    let (h, mut w) = following_meta(&r).await;
    let (token, fresh) = r.fanout.begin(&h, Target::Add(&h), &[Coll::Items]).unwrap();
    let seq = r.set(Coll::Items, "I-1");
    let first = first_frames(&r.db, ALPHA, &fresh, None).unwrap();
    assert_eq!(first.seq, seq);
    r.fanout.finish(&h, token, &first);
    let got = w.frames().await;
    assert_eq!(names(&got), ["snapshot", "ready"]);
    assert_eq!(ids(&got[0].1), ["I-1"]);
}

#[tokio::test]
async fn only_the_added_collections_snapshot_is_sent() {
    let r = rig();
    r.set(Coll::Items, "I-1");
    let (h, mut w) = following_meta(&r).await;
    let target = Target::Add(&h);
    r.fanout
        .subscribe(&r.db, ALPHA, target, &[Coll::Items], None)
        .unwrap();
    let got = w.frames().await;
    assert_eq!(names(&got), ["snapshot", "ready"]);
    assert_eq!(got[0].1["coll"], "items");
    assert_eq!(got[0].1["scope"], ALPHA);
    assert_eq!(got[1].1["colls"], json!(["items"]));
}

#[tokio::test]
async fn the_other_collections_keep_streaming_during_the_add() {
    let r = rig();
    let (h, mut w) = following_meta(&r).await;
    let (token, fresh) = r.fanout.begin(&h, Target::Add(&h), &[Coll::Items]).unwrap();
    r.set(Coll::Meta, "second");
    r.set(Coll::Items, "I-1");
    let during = w.frames().await;
    assert_eq!(names(&during), ["changes"]);
    assert_eq!(ids(&during[0].1), ["second"]);
    let first = first_frames(&r.db, ALPHA, &fresh, None).unwrap();
    r.fanout.finish(&h, token, &first);
    assert_eq!(names(&w.frames().await), ["snapshot", "ready"]);
}

#[tokio::test]
async fn a_repeat_subscribe_sends_no_snapshot_only_an_empty_ready() {
    let r = rig();
    let (h, mut w) = following_meta(&r).await;
    let target = Target::Add(&h);
    r.fanout
        .subscribe(&r.db, ALPHA, target, &[Coll::Meta], None)
        .unwrap();
    let seq = r.db.docs().seq().unwrap();
    let got = w.frames().await;
    assert_eq!(names(&got), ["ready"]);
    assert_eq!(got[0].1, json!({"scope": ALPHA, "seq": seq, "colls": []}));
}

#[tokio::test]
async fn a_cancelled_add_hands_its_collections_to_the_add_that_also_asked() {
    let r = rig();
    r.set(Coll::Items, "I-1");
    let (h, mut w) = following_meta(&r).await;
    let (one, _) = r.fanout.begin(&h, Target::Add(&h), &[Coll::Items]).unwrap();
    let both = [Coll::Items, Coll::Tests];
    let (two, read) = r.fanout.begin(&h, Target::Add(&h), &both).unwrap();
    assert_eq!(read, [Coll::Tests, Coll::Items]);
    let first = first_frames(&r.db, ALPHA, &read, None).unwrap();
    r.set(Coll::Items, "I-2");
    r.fanout.abort(&h, one);
    r.set(Coll::Items, "I-3");
    r.fanout.finish(&h, two, &first);
    let got = w.frames().await;
    let want = ["snapshot", "snapshot", "changes", "changes", "ready"];
    assert_eq!(names(&got), want);
    assert_eq!(got[1].1["coll"], "items");
    assert_eq!(ids(&got[1].1), ["I-1"]);
    assert_eq!(ids(&got[2].1), ["I-2"]);
    assert_eq!(ids(&got[3].1), ["I-3"]);
    assert_eq!(got[4].1["colls"], json!(["tests", "items"]));
}

#[tokio::test]
async fn unsubscribe_stops_one_collection_and_the_last_one_ends_the_client() {
    let r = rig();
    let (h, mut w) = following_meta(&r).await;
    let target = Target::Add(&h);
    let items = [Coll::Items];
    r.fanout
        .subscribe(&r.db, ALPHA, target, &items, None)
        .unwrap();
    w.frames().await;
    assert!(!r.fanout.unsubscribe(&h, &items));
    r.set(Coll::Items, "I-1");
    r.set(Coll::Meta, "second");
    let got = w.frames().await;
    assert_eq!(names(&got), ["changes"]);
    assert_eq!(ids(&got[0].1), ["second"]);
    assert!(r.fanout.unsubscribe(&h, &[Coll::Meta]));
    r.set(Coll::Meta, "third");
    assert_eq!(w.frames().await, Vec::<(String, serde_json::Value)>::new());
    assert!(r.fanout.lock().is_empty());
}

#[tokio::test]
async fn a_collection_dropped_while_pending_is_never_sent() {
    let r = rig();
    let (h, mut w) = following_meta(&r).await;
    let (token, fresh) = r.fanout.begin(&h, Target::Add(&h), &[Coll::Items]).unwrap();
    r.set(Coll::Items, "I-1");
    let first = first_frames(&r.db, ALPHA, &fresh, None).unwrap();
    assert!(!r.fanout.unsubscribe(&h, &[Coll::Items]));
    r.fanout.finish(&h, token, &first);
    r.set(Coll::Items, "I-2");
    assert_eq!(w.frames().await, Vec::<(String, serde_json::Value)>::new());
}

#[tokio::test]
async fn a_servable_since_sends_only_the_missed_changes_of_the_added_collections() {
    let r = rig();
    r.set(Coll::Items, "I-1");
    let since = r.set(Coll::Meta, "board");
    r.set(Coll::Items, "I-2");
    r.set(Coll::Meta, "second");
    let mut w = wire();
    let target = Target::New(Kind::Mux, &w.sender);
    r.fanout
        .subscribe(&r.db, ALPHA, target, &[Coll::Items], Some(since))
        .unwrap();
    let got = w.frames().await;
    assert_eq!(names(&got), ["changes", "ready"]);
    assert_eq!(ids(&got[0].1), ["I-2"]);
}

#[tokio::test]
async fn a_since_the_log_cannot_serve_falls_back_to_snapshots() {
    let r = rig();
    r.set(Coll::Items, "I-1");
    let mut w = wire();
    let target = Target::New(Kind::Mux, &w.sender);
    r.fanout
        .subscribe(&r.db, ALPHA, target, &[Coll::Items], Some(9999))
        .unwrap();
    assert_eq!(names(&w.frames().await), ["snapshot", "ready"]);
}
