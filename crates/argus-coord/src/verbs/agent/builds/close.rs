//! `coord build close <tag> [--note "why"] [--force]` and `coord build reopen <tag>`: the bundle is
//! finished and landed, its tests leave the owner's Tests tab until "Show closed"; the launchers
//! made for it go with it, and come back on a reopen.

use argus_board::json::{doc, shown, truthy};
use argus_board::limits::optional_text;
use argus_launchers::launcher::{CloseRequest, close_bundle, reopen_bundle};

use super::support::{authorize_manage, carrying, tag_arg, thread_of};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out::ago;
use crate::verbs::agent::base::opt::{json_or_null, not_one_text};
use crate::verbs::agent::base::write;
use crate::verbs::agent::rules;

/// The manual tests of the tag that nobody has run yet: the ones that make a close premature.
fn still_to_run(board: &argus_board::Board, tag: &str) -> Vec<String> {
    carrying(board, tag)
        .into_iter()
        .filter(|t| t["mode"] == "manual" && t["status"] == "pending")
        .map(|t| shown(t.get("id")))
        .collect()
}

/// `coord build close`.
///
/// # Errors
/// A note that is too long, manual tests still to run (without `--force`), a thread that may not
/// manage the bundle.
pub fn close(call: &mut Call<'_>) -> Outcome {
    let tag = tag_arg(&call.positional, "close")?;
    write(call, |board, who, run| {
        if not_one_text(run.options, "note") {
            return Err(CoordError::refuse("--note takes one value."));
        }
        let why = optional_text("buildnote", &json_or_null(run.options, "note"))?;
        authorize_manage(board, who, &[thread_of(board, &tag)])?;
        if let Some(have) = doc(board, "builds", &tag).filter(|b| truthy(b, "closedAt")) {
            let line = format!(
                "{tag}: already closed {} ago by {}.",
                ago(have["closedAt"].as_str(), run.now_ms),
                shown(have.get("closedBy"))
            );
            run.say(&line);
            return Ok(());
        }
        let pending = still_to_run(board, &tag);
        let request = CloseRequest {
            note: why.as_deref(),
            force: run.options.flag("force"),
            pending_tests: &pending,
        };
        let done = rules::run(board, who, |b, w, at| {
            close_bundle(b, w, &tag, &request, at)
        })?;
        let unhanded = if done.unhanded { " and unhanded" } else { "" };
        let gone = if done.retired.is_empty() {
            String::new()
        } else {
            format!(
                "; launcher {} made for it retired with it",
                done.retired.join(", ")
            )
        };
        run.say(&format!(
            "{tag}: closed{unhanded}{gone}. Its tests leave the owner's Tests tab until Show closed; coord build reopen {tag} undoes it."
        ));
        Ok(())
    })
}

/// `coord build reopen`.
///
/// # Errors
/// No such bundle, a thread that may not manage it.
pub fn reopen(call: &mut Call<'_>) -> Outcome {
    let tag = tag_arg(&call.positional, "reopen")?;
    write(call, |board, who, run| {
        let Some(rec) = doc(board, "builds", &tag) else {
            return Err(CoordError::refuse(format!("no bundle \"{tag}\".")));
        };
        let closed = truthy(rec, "closedAt");
        authorize_manage(board, who, &[thread_of(board, &tag)])?;
        if !closed {
            run.say(&format!("{tag}: is not closed."));
            return Ok(());
        }
        let cwd = run.cwd.to_string_lossy().into_owned();
        let back = rules::run(board, who, |b, w, at| {
            reopen_bundle(b, w, &tag, Some(&cwd), at)
        })?
        .unwrap_or_default();
        let named = if back.is_empty() {
            String::new()
        } else {
            format!("; launcher {} made for it is back", back.join(", "))
        };
        run.say(&format!("{tag}: reopened{named}."));
        Ok(())
    })
}
