//! The frame shapes of the two kinds of client. A legacy client is one page stream of one scope:
//! `id:` lines, `ready {seq}`, `reset {reason}`, bytes as the Node server writes them. A mux
//! client shares one stream between scopes: no `id:` line, and `ready` and `reset` name the scope.

use argus_core::{AppliedChange, Coll, DocsEvent};
use serde_json::{Value, json};

use super::first_frames::{Body, First};
use crate::sse::SseSender;

/// Which frame shapes a client gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The page stream of one scope (`GET /api/p/:scope/stream`).
    Legacy,
    /// One scope of a mux stream (`GET /api/mux`).
    Mux,
}

/// Whether a frame counts toward the live backlog limit.
#[derive(Clone, Copy)]
enum Phase {
    Opening,
    Live,
}

fn put(
    kind: Kind,
    phase: Phase,
    sender: &SseSender,
    name: &str,
    data: &Value,
    id: Option<i64>,
) -> bool {
    match (kind, phase) {
        (Kind::Legacy, Phase::Opening) => sender.opening_event(name, data, id),
        (Kind::Legacy, Phase::Live) => sender.event(name, data, id),
        (Kind::Mux, Phase::Opening) => sender.mux_opening_event(name, data),
        (Kind::Mux, Phase::Live) => sender.mux_event(name, data),
    }
}

fn changes_data(scope: &str, seq: i64, mine: &[&AppliedChange]) -> Value {
    let list: Vec<Value> = mine
        .iter()
        .map(|c| serde_json::to_value(&c.change).unwrap_or(Value::Null))
        .collect();
    json!({ "scope": scope, "seq": seq, "changes": list })
}

/// A live `changes` frame of the changes `keep` accepts; nothing when none is. `false` once the
/// stream is closed.
pub fn changes(
    kind: Kind,
    sender: &SseSender,
    scope: &str,
    e: &DocsEvent,
    keep: impl Fn(Coll) -> bool,
) -> bool {
    let mine: Vec<&AppliedChange> = e.changes.iter().filter(|c| keep(c.change.coll)).collect();
    if mine.is_empty() {
        return true;
    }
    let data = changes_data(scope, e.seq, &mine);
    put(kind, Phase::Live, sender, "changes", &data, Some(e.seq))
}

/// A `reset` frame.
pub fn reset(kind: Kind, sender: &SseSender, scope: &str, reason: &str) -> bool {
    let data = match kind {
        Kind::Legacy => json!({ "reason": reason }),
        Kind::Mux => json!({ "scope": scope, "reason": reason }),
    };
    put(kind, Phase::Live, sender, "reset", &data, None)
}

/// The `ready` frame that ends the opening of `colls`.
pub fn ready(kind: Kind, sender: &SseSender, scope: &str, seq: i64, colls: &[Coll]) -> bool {
    let data = match kind {
        Kind::Legacy => json!({ "seq": seq }),
        Kind::Mux => {
            let names: Vec<&str> = colls.iter().copied().map(Coll::as_str).collect();
            json!({ "scope": scope, "seq": seq, "colls": names })
        }
    };
    put(kind, Phase::Opening, sender, "ready", &data, None)
}

fn missed(
    kind: Kind,
    sender: &SseSender,
    scope: &str,
    all: &[AppliedChange],
    colls: &[Coll],
) -> bool {
    let mine: Vec<&AppliedChange> = all
        .iter()
        .filter(|c| colls.contains(&c.change.coll))
        .collect();
    let Some(last) = mine.iter().map(|c| c.seq).max() else {
        return true;
    };
    let data = changes_data(scope, last, &mine);
    put(kind, Phase::Opening, sender, "changes", &data, Some(last))
}

/// The opening frames of `colls`: one snapshot each, or the missed changes.
pub fn opening(kind: Kind, sender: &SseSender, scope: &str, first: &First, colls: &[Coll]) -> bool {
    match &first.body {
        Body::Snapshots(all) => {
            all.iter()
                .filter(|(c, _)| colls.contains(c))
                .all(|(coll, rows)| {
                    let data = json!({
                        "scope": scope,
                        "seq": first.seq,
                        "coll": coll.as_str(),
                        "docs": rows,
                    });
                    put(
                        kind,
                        Phase::Opening,
                        sender,
                        "snapshot",
                        &data,
                        Some(first.seq),
                    )
                })
        }
        Body::Missed(all) => missed(kind, sender, scope, all, colls),
    }
}
