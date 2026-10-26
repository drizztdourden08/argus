//! Changes that go through the shared ops of `argus-board` (`applyOp`): the soft verbs of an
//! agent and the hand of a build. The op of the CLI is last-writer unless `--if-rev` asks for the
//! revision check.

use argus_board::Board;
use argus_board::json::as_integer;
use argus_board::ops::{Status, Who, apply_op};
use serde_json::{Map, Value, json};

use crate::cli::Options;
use crate::cli::exit::CoordError;
use crate::who::Caller;

/// The caller as the ops know it.
#[must_use]
pub fn ops_who(who: &Caller) -> Who {
    Who {
        thread: Some(who.thread.clone()),
        role: who.role,
        actor: Some(who.actor.clone()),
    }
}

/// An op on one record: `{ kind, target, ...rest }` with `force`, or `baseRev` for `--if-rev`.
///
/// # Errors
/// `--if-rev` is not a whole number.
pub fn op(
    kind: &str,
    coll: &str,
    id: &str,
    rest: Map<String, Value>,
    options: &Options,
) -> Result<Value, CoordError> {
    let mut out = Map::new();
    out.insert("kind".to_owned(), json!(kind));
    out.insert("target".to_owned(), json!({ "c": coll, "id": id }));
    out.extend(rest);
    let Some(raw) = options.json("if-rev") else {
        out.insert("force".to_owned(), json!(true));
        return Ok(Value::Object(out));
    };
    let rev = as_integer(Some(&raw)).ok_or_else(|| {
        CoordError::refuse("--if-rev takes the revision number you read (rev in `show`).")
    })?;
    out.insert("baseRev".to_owned(), json!(rev));
    Ok(Value::Object(out))
}

/// `apply`: runs the op as the caller; a refusal or a conflict is the verb's refusal. Returns the
/// outcome sentence.
///
/// # Errors
/// The op was refused, or the record changed since the revision the caller read.
pub fn apply(board: &mut Board, who: &Caller, operation: &Value) -> Result<String, CoordError> {
    let result = apply_op(board, operation, &ops_who(who), None);
    match result.status {
        Status::Applied => Ok(result.outcome),
        Status::Refused => Err(CoordError::refuse(result.outcome)),
        Status::Conflict => {
            let current = result.current.unwrap_or_else(|| json!({}));
            Err(CoordError::refuse(format!(
                "{} changed since rev {} (now {current}). Read it again and retry.",
                operation["target"]["id"].as_str().unwrap_or_default(),
                operation["baseRev"]
            )))
        }
    }
}
