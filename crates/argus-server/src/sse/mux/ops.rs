//! Applies the ops of `POST /api/mux/:conn` to the connection's scope handles. A bad op answers
//! its own error and the others still run.

use std::collections::HashMap;

use argus_core::{ApiError, Coll, ErrorCode};
use argus_store::Db;
use serde_json::{Value, json};

use super::parse::{Op, OpError, parse};
use crate::sse::{Fanout, Handle, Kind, SseSender, Target};

/// The scope handles of one connection, by scope.
pub type Handles = HashMap<String, Handle>;

/// What an op needs from the server and the connection.
pub struct Env<'a> {
    /// The store the first frames are read from.
    pub db: &'a Db,
    /// The fan-out the scope clients live in.
    pub fanout: &'a Fanout,
    /// The stream of the connection.
    pub sender: &'a SseSender,
}

fn open_new(
    env: &Env<'_>,
    handles: &mut Handles,
    scope: &str,
    colls: &[Coll],
    since: Option<i64>,
) -> Result<(), ApiError> {
    let target = Target::New(Kind::Mux, env.sender);
    let h = env.fanout.subscribe(env.db, scope, target, colls, since)?;
    handles.insert(scope.to_owned(), h);
    Ok(())
}

fn subscribe(
    env: &Env<'_>,
    handles: &mut Handles,
    scope: &str,
    colls: &[Coll],
    since: Option<i64>,
) -> Result<(), ApiError> {
    let Some(h) = handles.get(scope).cloned() else {
        return open_new(env, handles, scope, colls, since);
    };
    match env
        .fanout
        .subscribe(env.db, scope, Target::Add(&h), colls, since)
    {
        Err(e) if e.code == ErrorCode::NotFound => {
            handles.remove(scope);
            open_new(env, handles, scope, colls, since)
        }
        other => other.map(|_| ()),
    }
}

fn unsubscribe(fanout: &Fanout, handles: &mut Handles, scope: &str, colls: Option<&[Coll]>) {
    let Some(h) = handles.get(scope).cloned() else {
        return;
    };
    let gone = match colls {
        None => {
            fanout.drop_client(&h);
            true
        }
        Some(c) => fanout.unsubscribe(&h, c),
    };
    if gone {
        handles.remove(scope);
    }
}

fn run(env: &Env<'_>, handles: &mut Handles, op: Op) -> Result<(), OpError> {
    match op {
        Op::Sub {
            scope,
            colls,
            since,
        } => Ok(subscribe(env, handles, &scope, &colls, since)?),
        Op::Unsub { scope, colls } => {
            unsubscribe(env.fanout, handles, &scope, colls.as_deref());
            Ok(())
        }
        Op::Later(name) => Err(OpError {
            code: "not_supported",
            message: format!("the {name} op is not supported by this server yet"),
        }),
    }
}

/// Applies the ops in order; one result per op: `{ok: true}` or `{error: {code, message}}`.
pub fn apply(env: &Env<'_>, handles: &mut Handles, ops: &[Value]) -> Vec<Value> {
    ops.iter()
        .map(
            |raw| match parse(raw).and_then(|op| run(env, handles, op)) {
                Ok(()) => json!({ "ok": true }),
                Err(e) => e.to_value(),
            },
        )
        .collect()
}
