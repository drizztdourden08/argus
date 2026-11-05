//! `coord demo seed [--project <key>]`: fills a preview board with an example of every record
//! kind the dashboard shows, linked to each other, so the owner can test the whole board.
//!
//! It writes through the real verbs and board ops (the steps are data in `argus_board::demo`),
//! never by hand. It refuses unless `COORD_HOME` is set to a home that is not the live one, and
//! refuses a board that already exists: delete the board folder to seed again.

mod across;
mod guard;
mod runner;

use argus_board::demo::{steps, write_releases};

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use runner::Runner;

const USAGE: &str = "Usage: COORD_HOME=<preview home> coord demo seed [--project <key>]\n  Fills the board <key> (default demo) of a preview home with one example of every record kind:\n  epics, stories, tasks, a bug and a debt, tests (AI and manual), plans and runs in every stage,\n  builds, agents with worktrees, asks, a decision, notes, pull requests, requests to the tessera,\n  brock and archipelia boards, a process definition and releases.\n  Refuses when COORD_HOME is unset or is the live home (~/.coord), and when the board exists.";

fn summary(run: &Runner, releases: usize) -> Vec<String> {
    vec![
        format!(
            "demo board {} seeded in {}",
            run.key(),
            run.home().display()
        ),
        format!(
            "{} calls passed, {} releases written, {} failed",
            run.passed,
            releases,
            run.failed.len()
        ),
        format!(
            "open it: COORD_HOME={} coord dashboard start",
            run.home().display()
        ),
    ]
}

fn plant(run: &mut Runner) -> Result<usize, CoordError> {
    across::folders(run).map_err(|e| CoordError::refuse(format!("demo folders: {e}")))?;
    across::targets(run);
    run.run_all(&steps());
    across::dependencies(run);
    across::merge(run);
    write_releases(run.home(), run.key())
        .map_err(|e| CoordError::refuse(format!("demo releases: {e}")))
}

/// The verb `coord demo seed`.
///
/// # Errors
/// Refused for a live or unset `COORD_HOME`, a bad key, an existing board, or when a step failed
/// (the others still ran; each failure is printed).
pub fn seed(call: &mut Call<'_>) -> Outcome {
    match call.positional.first().map(String::as_str) {
        Some("seed") => {}
        None | Some("help") => {
            call.inv.say(&[USAGE]);
            return Ok(());
        }
        Some(other) => {
            return Err(CoordError::refuse(format!(
                "coord demo {other}: unknown. {USAGE}"
            )));
        }
    }
    let home = guard::preview_home(&call.inv.env, &call.inv.cwd)?;
    let key = guard::board_key(call.options.text("project"))?;
    guard::fresh(&home, &key)?;
    std::fs::create_dir_all(&home).map_err(|e| CoordError::refuse(format!("preview home: {e}")))?;
    let mut run = Runner::new(&home, &key, &home);
    let releases = plant(&mut run)?;
    let lines = summary(&run, releases);
    let shown: Vec<&str> = lines.iter().map(String::as_str).collect();
    call.inv.say(&shown);
    if run.failed.is_empty() {
        return Ok(());
    }
    let failures = run.failed.join("\n  ");
    call.inv
        .warn(&format!("these steps did not apply:\n  {failures}"));
    Err(CoordError::Exit(1))
}
