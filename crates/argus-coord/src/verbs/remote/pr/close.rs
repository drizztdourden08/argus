//! `coord pr close PR-n`: an open pull request ends without a merge.

use serde_json::json;

use super::apply::apply;
use super::need_pr;
use crate::cli::exit::Outcome;
use crate::cli::route::Call;

/// `coord pr close PR-n`.
pub(super) fn close(call: &mut Call<'_>) -> Outcome {
    let id = need_pr(call.positional.first())?;
    let op = json!({ "kind": "pr-close", "target": { "c": "prs", "id": id }, "set": {} });
    apply(call, &op)?;
    call.inv.say(&[&format!("{id} closed")]);
    Ok(())
}
