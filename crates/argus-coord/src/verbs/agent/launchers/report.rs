//! The sentences `coord launcher set` adds after its line about links and the master role.

use argus_launchers::launcher::{Launcher, MASTER};

/// Why a `--use-every` was not kept: the build has no usage window.
pub(super) fn ignored_every(name: &str, rec: &Launcher) -> String {
    if rec.link.master || name == MASTER {
        return format!("--use-every ignored: {name} has no usage window (it is never stale).");
    }
    match rec.for_item.as_deref() {
        Some(item) => format!(
            "--use-every ignored: {name} serves {item}, so it has no usage window (it goes with the item)."
        ),
        None => format!(
            "--use-every ignored: {name} is joined to a bundle, so it has no usage window (it goes with the bundle)."
        ),
    }
}

/// What happens to the build that held the master role.
pub(super) fn handed_over(name: &str, old: &str) -> String {
    format!(
        "{name} is the master build now; {old} no longer is. Link {old} to the item it serves (coord launcher link {old} I-n) or retire it (coord launcher retire {old})."
    )
}
