//! Reads one op of the list from its JSON object.

use argus_core::ids::is_scope;
use argus_core::{ApiError, Coll};
use serde_json::{Value, json};

use super::presence::{Claim, claim_of, release_of};

/// What an op did wrong: a wire code and a message.
pub struct OpError {
    pub(super) code: &'static str,
    pub(super) message: String,
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

    /// The per-op answer: `{error: {code, message}}`.
    pub(super) fn to_value(&self) -> Value {
        json!({ "error": { "code": self.code, "message": self.message } })
    }
}

/// One checked op.
pub enum Op {
    /// Follow collections of a scope.
    Sub {
        scope: String,
        colls: Vec<Coll>,
        since: Option<i64>,
    },
    /// Stop following collections (all when `None`) of a scope.
    Unsub {
        scope: String,
        colls: Option<Vec<Coll>>,
    },
    /// The tab says which thread has its board open.
    Presence(Claim),
    /// The tab no longer has its board open.
    Unpresence { scope: String, tab: String },
    /// A known op whose piece is not built yet.
    Later(&'static str),
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

/// Checks one op.
///
/// # Errors
/// `bad_request` for a missing or unknown `op`, a bad scope or collection list.
pub fn parse(raw: &Value) -> Result<Op, OpError> {
    match raw["op"].as_str() {
        Some("sub") => parse_sub(raw),
        Some("unsub") => parse_unsub(raw),
        Some("presence") => claim_of(raw).map(Op::Presence),
        Some("unpresence") => release_of(raw).map(|(scope, tab)| Op::Unpresence { scope, tab }),
        Some("tail") => Ok(Op::Later("tail")),
        Some("untail") => Ok(Op::Later("untail")),
        _ => Err(OpError::bad(
            "each op is {op: sub|unsub|presence|unpresence|tail|untail, ...}",
        )),
    }
}
