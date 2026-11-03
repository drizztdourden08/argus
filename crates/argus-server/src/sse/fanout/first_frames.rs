//! What a new set of collections gets first: the changes it missed when the log still reaches
//! back to its resume point, else the rows of each collection. Read outside the fan-out lock.

use argus_core::{ApiError, AppliedChange, Coll};
use argus_store::Db;
use serde_json::{Map, Value, json};

/// The first data of an add, read at `seq`.
pub struct First {
    /// The store seq read before the rows. Buffered events at or below it are already in them.
    pub seq: i64,
    /// The rows or the missed changes.
    pub body: Body,
}

/// How the add starts.
pub enum Body {
    /// One entry per collection: its rows as snapshot docs.
    Snapshots(Vec<(Coll, Vec<Value>)>),
    /// What changed after the resume point, for the named collections only.
    Missed(Vec<AppliedChange>),
}

fn snapshot_doc(id: &str, data: &Value) -> Value {
    let mut doc: Map<String, Value> = data.as_object().cloned().unwrap_or_default();
    doc.insert("id".into(), json!(id));
    doc.insert("data".into(), data.clone());
    Value::Object(doc)
}

fn rows(db: &Db, scope: &str, coll: Coll) -> Result<(Coll, Vec<Value>), ApiError> {
    let docs = db
        .docs()
        .list(scope, coll)?
        .iter()
        .map(|r| snapshot_doc(&r.id, &r.data))
        .collect();
    Ok((coll, docs))
}

/// Reads the first data of `colls`. The seq is read before the rows, so a change that commits
/// after it is either in the rows and above the seq (a harmless repeat of a set) or only in the
/// events the caller buffered; none is lost.
///
/// # Errors
/// A store failure.
pub fn first_frames(
    db: &Db,
    scope: &str,
    colls: &[Coll],
    since: Option<i64>,
) -> Result<First, ApiError> {
    let docs = db.docs();
    let seq = docs.seq()?;
    let missed = match since {
        Some(s) => docs.changes_since(scope, s)?,
        None => None,
    };
    let body = match missed {
        Some(all) => Body::Missed(
            all.into_iter()
                .filter(|c| colls.contains(&c.change.coll))
                .collect(),
        ),
        None => Body::Snapshots(
            colls
                .iter()
                .map(|c| rows(db, scope, *c))
                .collect::<Result<_, _>>()?,
        ),
    };
    Ok(First { seq, body })
}
