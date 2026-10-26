//! `run_status` and `run_logs`: the launches a lead defined, the runs started from them and
//! their output.

use serde_json::{Map, Value, json};

use crate::args::{arg_str, arg_u64};
use crate::backend::{LogQuery, McpBackend, RunFilter};
use crate::error::ToolError;
use crate::json::pick;
use crate::project::need;

/// The message of a call that needs the runs service while it is down.
pub const RUNS_DOWN: &str = "the runs service is not running";

/// A run definition without its env values, listing only the env keys.
fn public_def(def: &Value) -> Value {
    let mut out = def.clone();
    let mut env_keys: Vec<Value> = Vec::new();
    if let Some(map) = out.as_object_mut() {
        if let Some(Value::Object(env)) = map.shift_remove("env") {
            env_keys = env.keys().map(|k| json!(k)).collect();
        }
        map.insert("envKeys".into(), Value::Array(env_keys));
    }
    out
}

/// `run_status`: one run, or the definitions plus recent runs.
pub fn run_status(backend: &dyn McpBackend, key: &str, args: &Value) -> Result<Value, ToolError> {
    if let Some(id) = arg_str(args, "run") {
        let run = need(backend.run_get(key, id), &format!("run {id} in {key}"))?;
        return Ok(json!({ "run": run }));
    }
    let defs = backend
        .run_defs(key)
        .unwrap_or_else(|| backend.docs(key, "launches"));
    let filter = RunFilter {
        status: arg_str(args, "status").map(str::to_owned),
        name: arg_str(args, "name").map(str::to_owned),
        limit: Some(arg_u64(args, "limit").unwrap_or(20).min(500)),
    };
    let defs: Vec<Value> = defs.iter().map(public_def).collect();
    Ok(json!({ "defs": defs, "runs": backend.run_list(Some(key), &filter) }))
}

/// `run_logs`: the lines of a run, from `after` or the last `tail`.
pub fn run_logs(backend: &dyn McpBackend, key: &str, args: &Value) -> Result<Value, ToolError> {
    if !backend.runs_up() {
        return Err(ToolError::unavailable(RUNS_DOWN));
    }
    let id = arg_str(args, "run").unwrap_or_default();
    let run = need(backend.run_get(key, id), &format!("run {id} in {key}"))?;
    let after = arg_u64(args, "after");
    let tail = arg_u64(args, "tail").or_else(|| after.is_none().then_some(200));
    let query = LogQuery {
        after,
        tail,
        stream: arg_str(args, "stream").map(str::to_owned),
    };
    let out = backend.run_logs(key, id, &query);
    let mut result = Map::new();
    pick(&mut result, &out, &["lines", "next", "dropped"]);
    pick(&mut result, &run, &["status"]);
    Ok(Value::Object(result))
}
