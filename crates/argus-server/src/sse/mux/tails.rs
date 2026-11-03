//! Run-log tails on the mux: a `tail` op starts the shared run tail with a sink that writes
//! `log`, `run` and `tail-end` frames tagged with the op's `sub`. The tail only ever drops its
//! own listeners; it never closes the stream, which every scope of the connection shares.

use std::collections::HashMap;
use std::sync::Arc;

use argus_core::ApiError;
use argus_launchers::{LogEvent, RunRow, check_key, check_rid};
use serde_json::{Value, json};

use super::parse::OpError;
use crate::Services;
use crate::routes::runs::tail::core::{Tail, Watching, locked};
use crate::routes::runs::tail::sink::TailSink;
use crate::sse::SseSender;

/// Longest `sub` id a tail may carry.
const MAX_SUB: usize = 64;

/// A checked `tail` op.
pub struct Want {
    sub: String,
    key: String,
    rid: String,
    after: u64,
}

/// The tails of a connection, by `sub`; each holds the listeners of its tail.
pub type Tails = HashMap<String, Arc<Watching>>;

/// The mux side of a tail: frames tagged with the `sub`.
struct MuxSink {
    sender: SseSender,
    sub: String,
}

impl TailSink for MuxSink {
    fn line(&self, event: &LogEvent) {
        let data = json!({
            "sub": self.sub, "seq": event.seq, "stream": event.stream,
            "at": event.at, "text": event.text,
        });
        let _ = self.sender.mux_event("log", &data);
    }

    fn status(&self, run: &RunRow) {
        let data = json!({ "sub": self.sub, "run": run });
        let _ = self.sender.mux_event("run", &data);
    }

    fn end(&self) {
        let data = json!({ "sub": self.sub });
        let _ = self.sender.mux_event("tail-end", &data);
    }
}

fn bad(message: impl Into<String>) -> OpError {
    ApiError::bad_request(message).into()
}

/// The `sub` of a tail or untail op: a short, non-empty text.
///
/// # Errors
/// `bad_request` for a missing, empty or too long `sub`.
pub fn sub_of(raw: &Value) -> Result<String, OpError> {
    let sub = raw["sub"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= MAX_SUB);
    match sub {
        Some(s) => Ok(s.to_owned()),
        None => Err(bad(format!("sub must be 1 to {MAX_SUB} characters"))),
    }
}

/// Checks a `tail` op.
///
/// # Errors
/// `bad_request` for a bad `sub`, project key or run id.
pub fn want_of(raw: &Value) -> Result<Want, OpError> {
    let key = check_key(raw["key"].as_str().unwrap_or_default());
    let rid = check_rid(raw["rid"].as_str().unwrap_or_default());
    let (key, rid) = match (key, rid) {
        (Ok(k), Ok(r)) => (k.to_owned(), r.to_owned()),
        (Err(e), _) | (_, Err(e)) => return Err(bad(e.message)),
    };
    Ok(Want {
        sub: sub_of(raw)?,
        key,
        rid,
        after: raw["after"].as_u64().unwrap_or(0),
    })
}

/// Starts the tail: replays after the cursor, then follows the run. The same `sub` again
/// replaces the earlier tail. A run that is gone answers `tail-end` and nothing else.
///
/// # Errors
/// `unavailable` when the run service is not running.
pub fn start(
    svc: &Services,
    sender: &SseSender,
    tails: &mut Tails,
    want: Want,
) -> Result<(), OpError> {
    let runs = svc
        .runs
        .clone()
        .ok_or_else(|| ApiError::unavailable("the run service is not running"))?;
    stop(tails, &want.sub);
    let watching: Arc<Watching> = Arc::default();
    let sink = Box::new(MuxSink {
        sender: sender.clone(),
        sub: want.sub.clone(),
    });
    tails.insert(want.sub, Arc::clone(&watching));
    Tail::new(runs, (want.key, want.rid), want.after, sink, watching).start(svc);
    Ok(())
}

/// Drops the tail's listeners. A `sub` that holds none is not an error.
pub fn stop(tails: &mut Tails, sub: &str) {
    if let Some(watching) = tails.remove(sub) {
        locked(&watching).clear();
    }
}

/// Drops every tail (the stream ended).
pub fn release_all(tails: &mut Tails) {
    for (_, watching) in tails.drain() {
        locked(&watching).clear();
    }
}
