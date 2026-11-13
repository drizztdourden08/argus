//! `argus migrate board-ids`: the one-off migration of every board key to an opaque id. The
//! server must be stopped first; the lead stops it and starts it again.

use std::path::PathBuf;

use argus_migrate::{Mode, Options};
use clap::{Args, Subcommand};

use crate::lifecycle;

/// The migrations.
#[derive(Subcommand)]
pub enum Verb {
    /// Turns every board key into an id and keeps the old keys as aliases. Without a flag it
    /// migrates (or finishes an interrupted run); the server must be stopped.
    BoardIds(BoardIds),
}

/// The flags of `argus migrate board-ids`.
#[derive(Args)]
pub struct BoardIds {
    /// Writes `board-ids.plan.json` (ids, names, the strings left for review) and changes
    /// nothing else.
    #[arg(long, conflicts_with_all = ["rollback", "restore"])]
    dry_run: bool,
    /// Reverses the migration from its journal; writes made since are kept.
    #[arg(long, conflicts_with = "restore")]
    rollback: bool,
    /// Puts a backup folder back; the current home is renamed aside, never deleted.
    #[arg(long, value_name = "BACKUP")]
    restore: Option<PathBuf>,
    /// The coord home; default the one Argus uses (`COORD_HOME`, else `~/.coord`).
    #[arg(long)]
    home: Option<PathBuf>,
}

fn mode_of(flags: &BoardIds) -> Mode {
    if let Some(from) = &flags.restore {
        Mode::Restore(from.clone())
    } else if flags.rollback {
        Mode::Rollback
    } else if flags.dry_run {
        Mode::DryRun
    } else {
        Mode::Migrate
    }
}

fn options_of(flags: &BoardIds) -> Options {
    let home = flags
        .home
        .clone()
        .unwrap_or_else(|| lifecycle::paths_of(None).coord_home);
    Options {
        home,
        mode: mode_of(flags),
        pid_alive: argus_server::instance::pid_alive,
        fault: None,
    }
}

/// Runs the migration and prints its report; the exit code.
pub async fn run(verb: Verb) -> Result<u8, String> {
    let Verb::BoardIds(flags) = verb;
    let opts = options_of(&flags);
    let done = tokio::task::spawn_blocking(move || argus_migrate::run(&opts))
        .await
        .map_err(|e| e.to_string())?;
    for line in done.map_err(|e| e.to_string())? {
        println!("{line}");
    }
    Ok(0)
}
