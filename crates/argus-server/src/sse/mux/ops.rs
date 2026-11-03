//! Applies the ops of `POST /api/mux/:conn` to the connection's scope handles. A bad op answers
//! its own error and the others still run.

use std::collections::HashMap;

use argus_core::{ApiError, Coll, ErrorCode};
use argus_wake::service::driver::SharedWake;
use serde_json::{Value, json};

use super::conn::State;
use super::parse::{Op, OpError, parse};
use super::{presence, tails};
use crate::Services;
use crate::sse::{Fanout, Handle, Kind, SseSender, Target};

/// The scope handles of one connection, by scope.
pub type Handles = HashMap<String, Handle>;

/// What an op needs from the server and the connection.
pub struct Env<'a> {
    /// The services: the store the first frames are read from, the fan-out the scope clients
    /// live in, the run service and the run log feed.
    pub svc: &'a Services,
    /// The stream of the connection.
    pub sender: &'a SseSender,
    /// The wake service the presence claims go to, when it runs.
    pub wake: Option<&'a SharedWake>,
}

fn open_new(
    env: &Env<'_>,
    handles: &mut Handles,
    scope: &str,
    colls: &[Coll],
    since: Option<i64>,
) -> Result<(), ApiError> {
    let target = Target::New(Kind::Mux, env.sender);
    let h = env
        .svc
        .fanout
        .subscribe(&env.svc.db, scope, target, colls, since)?;
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
        .svc
        .fanout
        .subscribe(&env.svc.db, scope, Target::Add(&h), colls, since)
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

fn run(env: &Env<'_>, state: &mut State, op: Op) -> Result<(), OpError> {
    match op {
        Op::Sub {
            scope,
            colls,
            since,
        } => Ok(subscribe(env, &mut state.handles, &scope, &colls, since)?),
        Op::Unsub { scope, colls } => {
            unsubscribe(
                &env.svc.fanout,
                &mut state.handles,
                &scope,
                colls.as_deref(),
            );
            Ok(())
        }
        Op::Presence(claim) => presence::attach(env.wake, &mut state.presence, &claim),
        Op::Unpresence { scope, tab } => {
            presence::detach(env.wake, &mut state.presence, &scope, &tab);
            Ok(())
        }
        Op::Tail(want) => tails::start(env.svc, env.sender, &mut state.tails, want),
        Op::Untail { sub } => {
            tails::stop(&mut state.tails, &sub);
            Ok(())
        }
    }
}

/// Applies the ops in order; one result per op: `{ok: true}` or `{error: {code, message}}`.
pub fn apply(env: &Env<'_>, state: &mut State, ops: &[Value]) -> Vec<Value> {
    ops.iter()
        .map(|raw| match parse(raw).and_then(|op| run(env, state, op)) {
            Ok(()) => json!({ "ok": true }),
            Err(e) => e.to_value(),
        })
        .collect()
}
