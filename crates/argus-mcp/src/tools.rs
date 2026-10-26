//! Which handler answers which tool.

use serde_json::Value;

use crate::call::Call;
use crate::error::ToolError;
use crate::proof::attach_proof;
use crate::read_tools::read;
use crate::write_tools::{
    answer, decision, notify_owner, post_entry, record_test_result, run_start, run_stop,
    send_to_lead,
};

/// The tools that take no project: they read across projects or name their own target.
pub const NO_PROJECT: [&str; 5] = [
    "health",
    "portfolio_overview",
    "search",
    "send_to_lead",
    "notify_owner",
];

/// Run one tool.
pub async fn run(name: &str, call: &Call<'_>) -> Result<Value, ToolError> {
    if let Some(done) = read(name, call).await {
        return done;
    }
    write(name, call).await
}

async fn write(name: &str, call: &Call<'_>) -> Result<Value, ToolError> {
    match name {
        "post_entry" => post_entry(call).await,
        "answer" => answer(call).await,
        "decision" => decision(call).await,
        "record_test_result" => record_test_result(call).await,
        _ => act(name, call).await,
    }
}

async fn act(name: &str, call: &Call<'_>) -> Result<Value, ToolError> {
    match name {
        "run_start" => run_start(call).await,
        "run_stop" => run_stop(call).await,
        "attach_proof" => attach_proof(call).await,
        "send_to_lead" => send_to_lead(call).await,
        "notify_owner" => notify_owner(call).await,
        _ => Err(ToolError::new(
            "unknown_tool",
            format!("no tool named \"{name}\""),
        )),
    }
}
