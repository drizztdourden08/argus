//! Builders for the fixture boards of the tests, as plain JSON: board docs with their
//! bookkeeping, threads and test docs. Compiled only with the `testkit` feature.

mod alpha;

pub use alpha::fill_alpha;

use serde_json::{Map, Value, json};

use crate::clock::{iso_from_ms, now_ms};

/// The ISO time `min` minutes ago.
#[must_use]
pub fn ago(min: i64) -> String {
    iso_from_ms(now_ms() - min * 60_000)
}

/// `base` with the fields of `fields` laid over it.
#[must_use]
pub fn merge(mut base: Value, fields: Value) -> Value {
    if let (Some(b), Value::Object(f)) = (base.as_object_mut(), fields) {
        b.extend(f);
    }
    base
}

/// A board doc with its bookkeeping, then `fields`, then revision 1.
#[must_use]
pub fn doc(id: &str, by: &str, fields: Value, rank: i64) -> Value {
    let at = ago(30);
    let mut d = merge(
        json!({ "id": id, "createdAt": at, "createdBy": by, "updatedAt": at, "updatedBy": by, "rank": rank }),
        fields,
    );
    d["rev"] = json!(1);
    d
}

/// A thread doc: the coordinator is leading, any other thread works on its subject.
#[must_use]
pub fn thread(id: &str, role: &str, subject: &str, session: &str, rank: i64) -> Value {
    let note = if role == "coordinator" {
        "leading".to_owned()
    } else {
        format!("working on {subject}")
    };
    json!({
        "id": id, "name": id, "role": role, "status": "active", "subject": subject, "brief": null, "note": note,
        "flags": { "needsHuman": false, "blocked": false }, "root": null, "session": session, "lastSeen": ago(1),
        "activity": "working", "registeredAt": ago(60), "createdBy": id, "updatedAt": ago(5), "updatedBy": id, "rev": 1, "rank": rank,
    })
}

/// A test doc of `item` owned by `thread`, `fields` laid over the pending auto test.
#[must_use]
pub fn test_doc(id: &str, item: &str, thread: &str, fields: Value, rank: i64) -> Value {
    let base = json!({
        "item": item, "thread": thread, "subject": thread, "title": null, "covers": null, "build": null, "mode": "auto", "run": null,
        "context": null, "changed": [], "setup": [], "steps": [], "pass": [], "fail": [], "notes": [], "ac": [], "status": "pending",
        "result": null, "runs": [], "proofs": [],
    });
    doc(id, thread, merge(base, fields), rank)
}

/// The docs keyed by their `id`.
#[must_use]
pub fn by_id(docs: Vec<Value>) -> Value {
    Value::Object(
        docs.into_iter()
            .map(|d| (d["id"].as_str().unwrap_or_default().to_owned(), d))
            .collect::<Map<_, _>>(),
    )
}
