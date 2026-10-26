//! `coord build ready <tag>` and `coord build draft <tag>`: ready is built and testable (Play
//! shown when a launcher is joined, hand allowed); draft is back to not built yet (no Play, it
//! cannot be handed, refused while it is the handed build).

use argus_board::json::doc;
use argus_launchers::launcher::{BundleStatus, resolve_bundle, set_bundle_status};

use super::support::{authorize_manage, tag_arg, thread_of};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::write;
use crate::verbs::agent::rules::{self, typed_board};

/// `coord build ready` (`ready`) and `coord build draft` (not `ready`).
///
/// # Errors
/// No such bundle, a closed one, the handed one going back to draft, a thread that may not
/// manage it.
pub fn run(call: &mut Call<'_>, ready: bool) -> Outcome {
    let (word, status) = if ready {
        ("ready", BundleStatus::Ready)
    } else {
        ("draft", BundleStatus::Draft)
    };
    let tag = tag_arg(&call.positional, word)?;
    write(call, |board, who, run| {
        if doc(board, "builds", &tag).is_none() {
            return Err(CoordError::refuse(format!(
                "no bundle \"{tag}\": coord build add {tag} makes it."
            )));
        }
        authorize_manage(board, who, &[thread_of(board, &tag)])?;
        let changed = rules::run(board, who, |b, w, at| {
            set_bundle_status(b, w, &tag, status, at)
        })?;
        let line = if !changed {
            format!("{tag}: already {word}.")
        } else if !ready {
            format!("{tag}: draft. No Play, and it cannot be handed until coord build ready {tag}.")
        } else {
            let typed = typed_board(board)?;
            let starts = resolve_bundle(&typed, typed.builds.get(&tag));
            let play = if starts.cmd.is_some_and(|c| !c.is_empty()) {
                " Play is shown.".to_owned()
            } else {
                format!(
                    " No launcher yet, so no Play: coord build set {tag} --launcher <name> (or --cmd \"...\")."
                )
            };
            format!("{tag}: ready. coord build hand {tag} is allowed.{play}")
        };
        run.say(&line);
        Ok(())
    })
}
