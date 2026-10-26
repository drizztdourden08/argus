//! `coord launcher retire <name> [--note "why"]`: no Play, under Retired on the page. `master` is
//! never retired.

use argus_board::json::{doc, shown, truthy};
use argus_launchers::launcher::retire_launcher;

use super::name::name_arg;
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out::ago;
use crate::verbs::agent::base::links::linked;
use crate::verbs::agent::base::opt::one_text;
use crate::verbs::agent::base::write;
use crate::verbs::agent::rules;

/// `coord launcher retire`.
///
/// # Errors
/// A note that is too long, no such launcher, master, a launcher of another thread.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let name = name_arg(&call.positional, "retire")?;
    let why = one_text(&call.options, "note", "launchernote", false)?.flatten();
    write(call, |board, who, run| {
        let Some(rec) = doc(board, "launchers", &name) else {
            return Err(CoordError::refuse(format!("no launcher \"{name}\".")));
        };
        if truthy(rec, "retiredAt") {
            run.say(&format!(
                "{name}: already retired {} by {}.",
                ago(rec["retiredAt"].as_str(), run.now_ms),
                shown(rec.get("retiredBy"))
            ));
            return Ok(());
        }
        let play = rules::run(board, who, |b, w, at| {
            retire_launcher(b, w, &name, why.as_deref(), at)
        })?;
        let links = linked(board, &name, None);
        let play = if play.is_empty() {
            String::new()
        } else {
            format!(" {play}")
        };
        let still = if links.is_empty() {
            String::new()
        } else {
            format!(
                " Bundle {} still link{} it: their Play is off until linked to another (coord build set <tag> --launcher <name>).",
                links.join(", "),
                if links.len() > 1 { "" } else { "s" }
            )
        };
        run.say(&format!(
            "{name}: retired.{play}{still} coord launcher set {name} brings it back."
        ));
        Ok(())
    })
}
