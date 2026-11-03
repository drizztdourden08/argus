//! `coord launcher unlink <name>`: drop the item a build serves. Only the master build may go
//! without one; every other build needs the item it serves, so it is linked to another item
//! (`coord launcher link`) or retired instead.

use argus_launchers::launcher::unlink_launcher;

use super::name::name_arg;
use crate::cli::Call;
use crate::cli::exit::Outcome;
use crate::verbs::agent::base::write;
use crate::verbs::agent::rules;

/// `coord launcher unlink`.
///
/// # Errors
/// No name, a launcher that does not exist or is retired, a build that is not the master, a
/// launcher of another thread.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let name = name_arg(&call.positional, "unlink")?;
    write(call, |board, who, run| {
        let gone = rules::run(board, who, |b, w, at| unlink_launcher(b, w, &name, at))?;
        match gone {
            Some(item) => run.say(&format!("{name}: no longer serves {item}.")),
            None => run.say(&format!("{name}: serves no item.")),
        }
        Ok(())
    })
}
