//! `coord launcher keep <name> [--days n] [--note "why"]`: counts as a use without running it, one
//! more full window from now, or `--days n` (1 to 365) held fresh. Who and why are recorded.

use argus_board::json::{as_integer, doc};
use argus_launchers::launcher::{EVERY_LEGACY, keep_launcher, usage_of};

use super::name::name_arg;
use crate::cli::Options;
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::opt::{json_or_null, not_one_text, one_text};
use crate::verbs::agent::base::write;
use crate::verbs::agent::rules::{self, typed_board};

fn days_of(options: &Options) -> Result<Option<i64>, CoordError> {
    if options.get("days").is_none() {
        return Ok(None);
    }
    let days = as_integer(Some(&json_or_null(options, "days")));
    match days {
        Some(d) if !not_one_text(options, "days") && (1..=365).contains(&d) => Ok(Some(d)),
        _ => Err(CoordError::refuse(
            "--days takes a whole number of days, 1 to 365.",
        )),
    }
}

/// `coord launcher keep`.
///
/// # Errors
/// A note that is too long, days out of range, no such launcher, a launcher that has no window to
/// keep or belongs to another thread.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let name = name_arg(&call.positional, "keep")?;
    let note = one_text(&call.options, "note", "launchernote", false)?.flatten();
    let days = days_of(&call.options)?;
    write(call, |board, who, run| {
        if doc(board, "launchers", &name).is_none() {
            return Err(CoordError::refuse(format!("no launcher \"{name}\".")));
        }
        rules::run(board, who, |b, w, at| {
            keep_launcher(b, w, &name, (days, note.as_deref()), at)
        })?;
        let typed = typed_board(board)?;
        let every = typed
            .launchers
            .get(&name)
            .and_then(|rec| usage_of(&typed, rec, run.now_ms, 0))
            .map_or_else(|| EVERY_LEGACY.to_owned(), |u| u.use_every);
        let span = match days {
            Some(d) => format!(" for {d} day{}", if d == 1 { "" } else { "s" }),
            None => format!(": it counts as used now, so the next {every} are quiet"),
        };
        let why = note.as_ref().map_or_else(String::new, |n| format!(" ({n})"));
        run.say(&format!(
            "{name}: kept{span}{why}. coord launcher set {name} --use-every <window> changes how often it is expected to be used; coord launcher retire {name} drops it."
        ));
        Ok(())
    })
}
