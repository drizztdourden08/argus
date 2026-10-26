//! The tools that read: one arm per tool over the shapes of the other modules.

use argus_core::ids::PORTFOLIO;
use serde_json::{Value, json};

use crate::agents::list_agents;
use crate::backend::McpBackend;
use crate::call::Call;
use crate::error::ToolError;
use crate::launchers::list_launchers;
use crate::lists::{get_item, get_test, list_entries, list_items, list_tests, list_threads};
use crate::runs_view::{run_logs, run_status};
use crate::search::search;
use crate::summary::board_summary;

/// The portfolio cards: the docs, or the sync service's when the docs hold none yet.
#[must_use]
pub fn portfolio_cards(backend: &dyn McpBackend) -> Vec<Value> {
    let cards = backend.docs(PORTFOLIO, "cards");
    if cards.is_empty() {
        backend.cards()
    } else {
        cards
    }
}

/// Run a read tool, or none when `name` is not one.
pub async fn read(name: &str, call: &Call<'_>) -> Option<Result<Value, ToolError>> {
    let (b, k, a) = (call.backend, call.key(), call.args);
    let out = match name {
        "health" => Ok(b.health().await),
        "portfolio_overview" => Ok(json!({ "cards": portfolio_cards(b) })),
        "board_summary" => Ok(board_summary(b, k)),
        "list_threads" => Ok(list_threads(b, k, a)),
        "list_items" => Ok(list_items(b, k, a)),
        "get_item" => get_item(b, k, a),
        "list_tests" => Ok(list_tests(b, k, a)),
        "get_test" => get_test(b, k, a, &call.base),
        "list_agents" => Ok(list_agents(b, k, a)),
        "list_entries" => Ok(list_entries(b, k, a)),
        "search" => search(b, a, call.ctx),
        "run_status" => run_status(b, k, a),
        "list_launchers" => Ok(list_launchers(b, k, a)),
        "run_logs" => run_logs(b, k, a),
        _ => return None,
    };
    Some(out)
}
