//! The shadow Overlord only reads (I-172): the one place where `coord` refuses a write by the
//! thread `coordinator-hosted`, whatever the project's allow rules let through the CLI.

use argus_board::store::read_board;
use argus_overlord::shadow::{THREAD, coord_words_allowed};

use super::args::Options;
use super::exit::CoordError;
use super::invocation::Invocation;
use crate::who::{caller, find_project};

/// What the shadow is told when it writes.
pub const REFUSAL: &str = "the shadow Overlord only reads";

/// Refuses a call of `thread` unless the thread is not the shadow or the words are one it may run.
/// `words` are the verb and its positional words; `role` is the `--role` option.
///
/// # Errors
/// Refused with [`REFUSAL`] for a write by the shadow.
pub fn check(thread: &str, words: &[&str], role: Option<&str>) -> Result<(), CoordError> {
    let reads = coord_words_allowed(words) && role != Some("coordinator");
    if thread != THREAD || reads {
        Ok(())
    } else {
        Err(CoordError::refuse(REFUSAL))
    }
}

/// The thread the call is made by, when the session is a hosted child and the board knows it.
fn hosted_thread(inv: &Invocation, options: &Options, project: Option<&str>) -> Option<String> {
    if inv.env.get("ARGUS_HOSTED") != Some("1") {
        return None;
    }
    let sid = inv.env.session_id();
    let project = match project {
        Some(project) => project.to_owned(),
        None => find_project(&inv.env, &inv.cwd, options, sid).ok()?,
    };
    let paths = inv.env.paths(&inv.cwd);
    let board = read_board(&paths.coord_home, &project).ok().flatten()?;
    caller(&board, &inv.cwd, &inv.env, options)
        .ok()
        .map(|who| who.thread)
}

/// The guard of the router: runs before the verb.
///
/// # Errors
/// Refused with [`REFUSAL`] for a write by the shadow.
pub fn guard(
    inv: &Invocation,
    options: &Options,
    project: Option<&str>,
    words: &[String],
) -> Result<(), CoordError> {
    let Some(thread) = hosted_thread(inv, options, project) else {
        return Ok(());
    };
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    check(&thread, &words, options.text("role"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-tag: critical
    #[test]
    fn a_write_by_the_shadow_is_refused_and_by_another_thread_passes() {
        let write = ["item", "status", "I-1", "done"];
        assert!(check(THREAD, &write, None).is_err());
        assert!(check(THREAD, &["entry", "add"], None).is_err());
        assert!(check(THREAD, &["inbox", "send", "x"], None).is_err());
        assert!(check("coordinator", &write, None).is_ok());
        assert!(check("brock", &write, None).is_ok());
    }

    #[test]
    fn the_shadow_may_read_register_note_and_ack() {
        assert!(check(THREAD, &["item", "list"], None).is_ok());
        assert!(check(THREAD, &["inbox"], None).is_ok());
        assert!(check(THREAD, &["thread", "register", THREAD], None).is_ok());
        assert!(check(THREAD, &["thread", "note", "seen it"], None).is_ok());
    }

    #[test]
    fn the_shadow_never_registers_as_the_coordinator() {
        let register = ["thread", "register", THREAD];
        assert!(check(THREAD, &register, Some("coordinator")).is_err());
        assert!(check(THREAD, &["thread", "register", "coordinator"], None).is_err());
    }
}
