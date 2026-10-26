//! `coord build hand <tag>` and `coord build unhand`: the coordinator says which build the owner
//! is handed now. A missing record is made (as ready), a draft or closed bundle is refused, and so
//! is a project with no master launcher.

use argus_launchers::launcher::{check_hand, resolve_bundle};
use serde_json::json;

use super::support::carrying;
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::apply::apply;
use crate::verbs::agent::base::write;
use crate::verbs::agent::rules::{refused, typed_board};

/// `coord build hand`.
///
/// # Errors
/// No tag, a closed or draft bundle, no master launcher, a thread that is not the coordinator.
pub fn hand(call: &mut Call<'_>) -> Outcome {
    let Some(tag) = call.positional.first().filter(|t| !t.is_empty()).cloned() else {
        return Err(CoordError::refuse(
            "name the build: coord build hand <tag>.",
        ));
    };
    write(call, |board, who, run| {
        check_hand(&typed_board(board)?, &tag).map_err(refused)?;
        let outcome = apply(board, who, &json!({ "kind": "build", "handed": tag }))?;
        let handed = board["handedBuild"].as_str().map(str::to_owned);
        let carried = handed.as_deref().map_or(0, |h| carrying(board, h).len());
        let none = if carried == 0 {
            " No test carries this tag yet: tag the tests with coord test add|set --build."
        } else {
            ""
        };
        run.say(&format!("{outcome}.{none}"));
        if let Some(handed) = handed {
            let typed = typed_board(board)?;
            let starts = resolve_bundle(&typed, typed.builds.get(&handed));
            if starts.cmd.is_none_or(|c| c.is_empty()) {
                run.say(&format!(
                    "tip: give the owner something to start: coord build set {handed} --launcher <name> (coord launcher list), or --cmd \"<terminal command>\"."
                ));
            }
        }
        Ok(())
    })
}

/// `coord build unhand`.
///
/// # Errors
/// A thread that is not the coordinator.
pub fn unhand(call: &mut Call<'_>) -> Outcome {
    write(call, |board, who, run| {
        let outcome = apply(board, who, &json!({ "kind": "build", "handed": null }))?;
        run.say(&format!("{outcome}."));
        Ok(())
    })
}
