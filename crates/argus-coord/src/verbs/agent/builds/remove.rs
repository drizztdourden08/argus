//! `coord build rm <tag> [--force]`: delete the record. Refused while live tests carry the tag
//! (`--force` drops the record only). The launchers made for the bundle are retired with it.

use argus_board::json::{doc, map_mut, nullish, truthy};
use argus_launchers::launcher::retire_made_for;

use super::launch::{drop_legacy, retire_orphan};
use super::support::{audit, authorize_manage, carrying, tag_arg, thread_of};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::write;
use crate::verbs::agent::rules;

/// `coord build rm`.
///
/// # Errors
/// No such bundle, live tests that carry the tag, a thread that may not manage it.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let tag = tag_arg(&call.positional, "rm")?;
    write(call, |board, who, run| {
        let Some(rec) = doc(board, "builds", &tag) else {
            return Err(CoordError::refuse(format!("no bundle \"{tag}\".")));
        };
        let launcher = nullish(rec, "launcher");
        let legacy = truthy(rec, "cmd");
        authorize_manage(board, who, &[thread_of(board, &tag)])?;
        let carried = carrying(board, &tag).len();
        if carried > 0 && !run.options.flag("force") {
            return Err(CoordError::refuse(format!(
                "{carried} live test(s) still carry build {tag}: move or clear their tag first, or pass --force to drop the record only."
            )));
        }
        let mut notes = vec![retire_orphan(board, who, &tag, launcher.as_deref())?];
        let why = format!("bundle {tag} removed");
        let gone = rules::run(board, who, |b, w, at| {
            retire_made_for(b, w, &tag, (&why, false), at)
        })?;
        notes.extend(gone.iter().map(|n| format!("launcher {n} retired")));
        notes.push(if legacy {
            drop_legacy(board, &tag)
        } else {
            String::new()
        });
        notes.retain(|n| !n.is_empty());
        map_mut(board, "builds").shift_remove(&tag);
        audit(board, who, &format!("removed bundle {tag}"));
        let named = if notes.is_empty() {
            String::new()
        } else {
            format!(" ({})", notes.join(", "))
        };
        let kept = if carried > 0 {
            format!(" ({carried} test(s) keep the tag)")
        } else {
            String::new()
        };
        let handed = if board["handedBuild"].as_str() == Some(tag.as_str()) {
            "; it is still the handed build (coord build unhand)"
        } else {
            ""
        };
        run.say(&format!("{tag}: bundle removed{named}{kept}{handed}."));
        Ok(())
    })
}
