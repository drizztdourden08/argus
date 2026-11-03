//! The op list of `POST /api/mux/:conn`: each op is parsed from its JSON object, then applied to
//! the connection's scope handles. A bad op answers its own error and the others still run.

use std::collections::HashMap;

use argus_core::ids::is_scope;
use argus_core::{ApiError, Coll, ErrorCode};
use argus_store::Db;
use serde_json::{Value, json};

use crate::sse::{Fanout, Handle, Kind, SseSender, Target};

/// The scope handles of one connection, by scope.
pub type Handles = HashMap<String, Handle>;

/// What an op did wrong: a wire code and a message.
pub struct OpError {
    code: &'static str,
    message: String,
}

impl From<ApiError> for OpError {
    fn from(e: ApiError) -> Self {
        OpError {
            code: e.code.as_str(),
            message: e.message,
        }
    }
}

impl OpError {
    fn bad(message: impl Into<String>) -> Self {
        ApiError::bad_request(message).into()
    }

    fn to_value(&self) -> Value {
        json!({ "error": { "code": self.code, "message": self.message } })
    }
}

/// One checked op.
enum Op {
    Sub {
        scope: String,
        colls: Vec<Coll>,
        since: Option<i64>,
    },
    Unsub {
        scope: String,
        colls: Option<Vec<Coll>>,
    },
    /// A known op whose piece is not built yet.
    Later(&'static str),
}

/// What an op needs from the server and the connection.
pub struct Env<'a> {
    /// The store the first frames are read from.
    pub db: &'a Db,
    /// The fan-out the scope clients live in.
    pub fanout: &'a Fanout,
    /// The stream of the connection.
    pub sender: &'a SseSender,
}

fn scope_of(raw: &Value) -> Result<String, OpError> {
    match raw["scope"].as_str().filter(|s| is_scope(s)) {
        Some(s) => Ok(s.to_owned()),
        None => Err(OpError::bad("scope must be a project key or _portfolio")),
    }
}

fn colls_of(list: &[Value]) -> Result<Vec<Coll>, OpError> {
    let mut out: Vec<Coll> = Vec::new();
    for item in list {
        let name = item.as_str().unwrap_or_default();
        let c: Coll = name
            .parse()
            .map_err(|_| OpError::bad(format!("unknown collection \"{name}\"")))?;
        if !out.contains(&c) {
            out.push(c);
        }
    }
    Ok(out)
}

fn parse_sub(raw: &Value) -> Result<Op, OpError> {
    let list = raw["colls"].as_array().filter(|l| !l.is_empty());
    let Some(list) = list else {
        return Err(OpError::bad("colls must name at least one collection"));
    };
    Ok(Op::Sub {
        scope: scope_of(raw)?,
        colls: colls_of(list)?,
        since: raw["since"].as_i64().filter(|n| *n >= 0),
    })
}

fn parse_unsub(raw: &Value) -> Result<Op, OpError> {
    let colls = match raw.get("colls") {
        None | Some(Value::Null) => None,
        Some(Value::Array(list)) => Some(colls_of(list)?),
        Some(_) => return Err(OpError::bad("colls must be a list of collections")),
    };
    Ok(Op::Unsub {
        scope: scope_of(raw)?,
        colls,
    })
}

fn parse(raw: &Value) -> Result<Op, OpError> {
    match raw["op"].as_str() {
        Some("sub") => parse_sub(raw),
        Some("unsub") => parse_unsub(raw),
        Some("presence") => Ok(Op::Later("presence")),
        Some("unpresence") => Ok(Op::Later("unpresence")),
        Some("tail") => Ok(Op::Later("tail")),
        Some("untail") => Ok(Op::Later("untail")),
        _ => Err(OpError::bad(
            "each op is {op: sub|unsub|presence|unpresence|tail|untail, ...}",
        )),
    }
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
