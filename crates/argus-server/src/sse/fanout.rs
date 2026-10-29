//! The doc stream (`GET /api/p/:scope/stream`): one stream per page, several collections over
//! it. On connect: the missed changes when the log still reaches back to `since`, else one
//! snapshot per collection; then `ready`. Every doc apply becomes at most one `changes` event
//! per client, filtered to its collections; a hard resync sends `reset`.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};
use std::time::Duration;

use argus_core::{ApiError, AppliedChange, Coll, DocsEvent, ResetEvent};
use argus_store::Db;
use axum::response::Response;
use serde_json::{Map, Value, json};

use super::{SseSender, open_stream};

/// Streams one scope may hold.
pub const MAX_CLIENTS_PER_SCOPE: usize = 32;

struct Client {
    id: u64,
    colls: HashSet<Coll>,
    sender: SseSender,
    pending: Option<Vec<DocsEvent>>,
}

/// The fan-out hub: bus events to the streams of their scope.
#[derive(Default)]
pub struct Fanout {
    clients: Mutex<HashMap<String, Vec<Client>>>,
    next: AtomicU64,
}

fn changes_json(changes: &[&AppliedChange]) -> Value {
    Value::Array(
        changes
            .iter()
            .map(|c| serde_json::to_value(&c.change).unwrap_or(Value::Null))
            .collect(),
    )
}

fn changes_frame(sender: &SseSender, colls: &HashSet<Coll>, scope: &str, e: &DocsEvent) -> bool {
    let mine: Vec<&AppliedChange> = e
        .changes
        .iter()
        .filter(|c| colls.contains(&c.change.coll))
        .collect();
    if mine.is_empty() {
        return true;
    }
    sender.event(
        "changes",
        &json!({ "scope": scope, "seq": e.seq, "changes": changes_json(&mine) }),
        Some(e.seq),
    )
}

fn snapshot_doc(id: &str, data: &Value) -> Value {
    let mut doc: Map<String, Value> = data.as_object().cloned().unwrap_or_default();
    doc.insert("id".into(), json!(id));
    doc.insert("data".into(), data.clone());
    Value::Object(doc)
}

/// What a new client asked for.
pub struct StreamRequest<'a> {
    /// The scope.
    pub scope: &'a str,
    /// Its collections.
    pub colls: Vec<Coll>,
    /// The resume point.
    pub since: Option<i64>,
}

impl Fanout {
    fn lock(&self) -> MutexGuard<'_, HashMap<String, Vec<Client>>> {
        self.clients.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Delivers one doc apply.
    pub fn on_docs(&self, e: &DocsEvent) {
        let mut all = self.lock();
        let Some(list) = all.get_mut(&e.scope) else {
            return;
        };
        list.retain_mut(|c| match &mut c.pending {
            Some(buf) => {
                buf.push(e.clone());
                true
            }
            None => changes_frame(&c.sender, &c.colls, &e.scope, e),
        });
    }

    /// Delivers a reset.
    pub fn on_reset(&self, e: &ResetEvent) {
        let mut all = self.lock();
        if let Some(list) = all.get_mut(&e.scope) {
            list.retain(|c| {
                c.sender
                    .event("reset", &json!({ "reason": e.reason }), None)
            });
        }
    }

    /// Ends every stream (shutdown).
    pub fn close_all(&self) {
        self.lock().clear();
    }

    fn remove(&self, scope: &str, id: u64) {
        let mut all = self.lock();
        if let Some(list) = all.get_mut(scope) {
            list.retain(|c| c.id != id);
            if list.is_empty() {
                all.remove(scope);
            }
        }
    }

    fn register(
        self: &Arc<Self>,
        req: &StreamRequest,
        heartbeat: Duration,
        open: &Arc<AtomicUsize>,
    ) -> Result<(u64, Response), ApiError> {
        let mut all = self.lock();
        let list = all.entry(req.scope.to_owned()).or_default();
        if list.len() >= MAX_CLIENTS_PER_SCOPE {
            return Err(ApiError::unavailable(format!(
                "too many streams open for {} (max {MAX_CLIENTS_PER_SCOPE})",
                req.scope
            )));
        }
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        let weak: Weak<Fanout> = Arc::downgrade(self);
        let scope = req.scope.to_owned();
        let on_drop = Box::new(move || {
            if let Some(f) = weak.upgrade() {
                f.remove(&scope, id);
            }
        });
        let (sender, res) = open_stream(heartbeat, open, Some(on_drop));
        list.push(Client {
            id,
            colls: req.colls.iter().copied().collect(),
            sender,
            pending: Some(Vec::new()),
        });
        Ok((id, res))
    }

    /// Opens a doc stream: the first frames, then live changes.
    ///
    /// # Errors
    /// 503 past 32 streams for the scope; a store failure.
    pub fn attach(
        self: &Arc<Self>,
        db: &Db,
        req: &StreamRequest,
        heartbeat: Duration,
        open: &Arc<AtomicUsize>,
    ) -> Result<Response, ApiError> {
        let (id, res) = self.register(req, heartbeat, open)?;
        let first = first_frames(db, req);
        let mut all = self.lock();
        let Some(client) = all
            .get_mut(req.scope)
            .and_then(|l| l.iter_mut().find(|c| c.id == id))
        else {
            return Ok(res);
        };
        let (frames, seq) = match first {
            Ok(v) => v,
            Err(e) => {
                drop(all);
                self.remove(req.scope, id);
                return Err(e);
            }
        };
        let sent = frames
            .iter()
            .all(|(name, data, fid)| client.sender.opening_event(name, data, *fid));
        let pending = client.pending.take().unwrap_or_default();
        let live = sent
            && pending
                .iter()
                .filter(|e| e.seq > seq)
                .all(|e| changes_frame(&client.sender, &client.colls, req.scope, e));
        if !live {
            client.sender.close();
        }
        Ok(res)
    }
}

type Frame = (&'static str, Value, Option<i64>);

fn first_frames(db: &Db, req: &StreamRequest) -> Result<(Vec<Frame>, i64), ApiError> {
    let docs = db.docs();
    let seq = docs.seq()?;
    let missed = match req.since {
        Some(s) => docs.changes_since(req.scope, s)?,
        None => None,
    };
    let mut frames = Vec::new();
    if let Some(missed) = missed {
        let mine: Vec<&AppliedChange> = missed
            .iter()
            .filter(|c| req.colls.contains(&c.change.coll))
            .collect();
        if let Some(last) = mine.iter().map(|c| c.seq).max() {
            frames.push((
                "changes",
                json!({ "scope": req.scope, "seq": last, "changes": changes_json(&mine) }),
                Some(last),
            ));
        }
    } else {
        for coll in &req.colls {
            let rows: Vec<Value> = docs
                .list(req.scope, *coll)?
                .iter()
                .map(|r| snapshot_doc(&r.id, &r.data))
                .collect();
            frames.push((
                "snapshot",
                json!({ "scope": req.scope, "seq": seq, "coll": coll.as_str(), "docs": rows }),
                Some(seq),
            ));
        }
    }
    frames.push(("ready", json!({ "seq": seq }), None));
    Ok((frames, seq))
}
