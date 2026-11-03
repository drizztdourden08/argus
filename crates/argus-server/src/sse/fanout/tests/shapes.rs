//! The frame shapes of the two kinds, the order of the opening, and the client lifecycle.
// test-tag: critical

use serde_json::json;

use super::*;

/// Follows `meta`, adds `items` with one event landing after the rows were read, and answers the
/// frames of the add.
async fn add_with_a_late_event(r: &Rig, kind: Kind) -> Vec<(String, Value)> {
    let mut w = wire();
    let target = Target::New(kind, &w.sender);
    let h = r
        .fanout
        .subscribe(&r.db, ALPHA, target, &[Coll::Meta], None);
    let h = h.expect("subscribed");
    w.frames().await;
    let (token, fresh) = r.fanout.begin(&h, Target::Add(&h), &[Coll::Items]).unwrap();
    let first = first_frames(&r.db, ALPHA, &fresh, None).unwrap();
    r.set(Coll::Items, &format!("late-{kind:?}"));
    r.fanout.finish(&h, token, &first);
    w.frames().await
}

#[tokio::test]
async fn a_legacy_client_gets_ready_before_the_buffered_events_and_a_mux_client_after() {
    let r = rig();
    let legacy = add_with_a_late_event(&r, Kind::Legacy).await;
    assert_eq!(names(&legacy), ["snapshot", "ready", "changes"]);
    let mux = add_with_a_late_event(&r, Kind::Mux).await;
    assert_eq!(names(&mux), ["snapshot", "changes", "ready"]);
}

#[tokio::test]
async fn ready_and_reset_are_tagged_for_a_mux_client_and_untagged_for_a_legacy_one() {
    let r = rig();
    let (mut mux, mut legacy) = (wire(), wire());
    let colls = [Coll::Meta, Coll::Items];
    for (w, kind) in [(&mux, Kind::Mux), (&legacy, Kind::Legacy)] {
        let target = Target::New(kind, &w.sender);
        r.fanout
            .subscribe(&r.db, ALPHA, target, &colls, None)
            .unwrap();
    }
    let seq = r.db.docs().seq().unwrap();
    let (mux_open, legacy_open) = (mux.frames().await, legacy.frames().await);
    assert_eq!(
        mux_open[2].1,
        json!({"scope": ALPHA, "seq": seq, "colls": ["meta", "items"]})
    );
    assert_eq!(legacy_open[2].1, json!({ "seq": seq }));
    let reset = ResetEvent {
        scope: ALPHA.into(),
        reason: "resync".into(),
    };
    r.fanout.on_reset(&reset);
    let (mux_reset, legacy_reset) = (mux.frames().await, legacy.frames().await);
    assert_eq!(mux_reset[0].1, json!({"scope": ALPHA, "reason": "resync"}));
    assert_eq!(legacy_reset[0].1, json!({ "reason": "resync" }));
}

#[tokio::test]
async fn a_legacy_client_keeps_its_id_lines_and_a_mux_client_has_none() {
    let r = rig();
    let seq = r.set(Coll::Items, "I-1");
    for kind in [Kind::Legacy, Kind::Mux] {
        let mut w = wire();
        let target = Target::New(kind, &w.sender);
        r.fanout
            .subscribe(&r.db, ALPHA, target, &[Coll::Items], None)
            .unwrap();
        let text = w.raw().await.concat();
        assert_eq!(
            text.contains(&format!(
                "id: {seq}
"
            )),
            kind == Kind::Legacy
        );
        assert_eq!(text.contains("id:"), kind == Kind::Legacy);
    }
}

#[tokio::test]
async fn no_collection_is_refused() {
    let r = rig();
    let w = wire();
    let target = Target::New(Kind::Mux, &w.sender);
    assert!(r.fanout.subscribe(&r.db, ALPHA, target, &[], None).is_err());
    assert!(r.fanout.lock().is_empty());
}

#[tokio::test]
async fn a_scope_holds_32_clients() {
    let r = rig();
    let wires: Vec<Wire> = (0..MAX_CLIENTS_PER_SCOPE).map(|_| wire()).collect();
    for w in &wires {
        let target = Target::New(Kind::Mux, &w.sender);
        r.fanout
            .subscribe(&r.db, ALPHA, target, &[Coll::Meta], None)
            .unwrap();
    }
    let extra = wire();
    let target = Target::New(Kind::Mux, &extra.sender);
    let over = r
        .fanout
        .subscribe(&r.db, ALPHA, target, &[Coll::Meta], None);
    assert!(over.is_err());
}

#[tokio::test]
async fn dropping_one_scope_leaves_the_shared_sender_open_for_the_other() {
    let r = rig();
    let mut w = wire();
    let target = Target::New(Kind::Mux, &w.sender);
    let a = r
        .fanout
        .subscribe(&r.db, "alpha", target, &[Coll::Meta], None)
        .unwrap();
    let target = Target::New(Kind::Mux, &w.sender);
    r.fanout
        .subscribe(&r.db, "beta", target, &[Coll::Meta], None)
        .unwrap();
    w.frames().await;
    r.fanout.drop_client(&a);
    assert!(!w.sender.is_closed());
    r.set_in("alpha", Coll::Meta, "gone");
    r.set_in("beta", Coll::Meta, "kept");
    let got = w.frames().await;
    assert_eq!(names(&got), ["changes"]);
    assert_eq!(got[0].1["scope"], "beta");
}

#[tokio::test]
async fn a_handle_of_another_scope_is_refused() {
    let r = rig();
    let w = wire();
    let target = Target::New(Kind::Mux, &w.sender);
    let h = r
        .fanout
        .subscribe(&r.db, ALPHA, target, &[Coll::Meta], None);
    let h = h.unwrap();
    let err = r
        .fanout
        .subscribe(&r.db, "beta", Target::Add(&h), &[Coll::Items], None)
        .unwrap_err();
    assert_eq!(err.code, argus_core::ErrorCode::BadRequest);
}

#[tokio::test]
async fn close_all_ends_the_stream_of_every_client() {
    let r = rig();
    let w = wire();
    for scope in ["alpha", "beta"] {
        let target = Target::New(Kind::Mux, &w.sender);
        r.fanout
            .subscribe(&r.db, scope, target, &[Coll::Meta], None)
            .unwrap();
    }
    r.fanout.close_all();
    assert!(w.sender.is_closed());
    assert!(r.fanout.lock().is_empty());
}
