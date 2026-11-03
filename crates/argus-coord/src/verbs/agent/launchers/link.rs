//! `coord launcher link <name> <I-n>`: the item a build serves. The build goes with the item and
//! closes when it does. It also answers a build that `coord build migrate-links` could not match.

use argus_board::json::{doc, truthy};
use argus_board::served::{check_for, for_of, is_master};
use argus_launchers::launcher::link_launcher;

use super::name::name_arg;
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::opt::word;
use crate::verbs::agent::base::write;
use crate::verbs::agent::rules;

/// `coord launcher link`.
///
/// # Errors
/// No name or no item, a launcher that does not exist or is retired, an item a build may not
/// serve, a launcher of another thread.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let name = name_arg(&call.positional, "link")?;
    let item = word(&call.positional, 1).trim().to_owned();
    if item.is_empty() {
        return Err(CoordError::refuse(
            "name the item the build serves: coord launcher link <name> I-n.",
        ));
    }
    write(call, |board, who, run| {
        let Some(rec) = doc(board, "launchers", &name) else {
            return Err(CoordError::refuse(format!("no launcher \"{name}\".")));
        };
        if for_of(rec) == Some(item.as_str()) && !truthy(rec, "retiredAt") {
            run.say(&format!("{name}: already serves {item}."));
            return Ok(());
        }
        let master = is_master(board, &name);
        check_for(board, master, Some(item.as_str()))?;
        rules::run(board, who, |b, w, at| {
            link_launcher(b, w, (&name, &item), at)
        })?;
        let ends = if master {
            String::new()
        } else {
            format!(" It closes when {item} does.")
        };
        run.say(&format!("{name}: serves {item} now.{ends}"));
        Ok(())
    })
}
