//! The `argus` binary. `argus serve` runs the server in the foreground and headless; `argus start`,
//! `stop`, `status`, `open`, `logs`, `resync` and `token` are the lifecycle verbs of the Node
//! `dashboard` bin; `argus tray` runs the tray icon that starts and quits it; the other verbs
//! arrive in later slices.
#![deny(clippy::cognitive_complexity)]

mod lifecycle;
mod tray;

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use argus::wake_stream::{self, Io};
use clap::{Parser, Subcommand};
use tokio_util::sync::CancellationToken;
use tray::TrayAction;

#[derive(Parser)]
#[command(
    name = "argus",
    version,
    about = "The coordination dashboard of every project on this machine."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(flatten)]
    Lifecycle(lifecycle::Verb),
    /// Prints one line per wake of a project's wake stream (for a Monitor); see
    /// `--project <key> [--thread <id|*>] [--id <label>]`.
    WakeStream {
        /// The flags, read by the wake-stream command itself.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// The tray icon (Windows): run, start, stop or status; start is the default.
    Tray {
        #[command(subcommand)]
        action: Option<TrayAction>,
        /// The dashboard home (config, database, logs); default `<COORD_HOME>/dashboard`.
        #[arg(long, alias = "home", global = true)]
        data_dir: Option<PathBuf>,
    },
}

/// Ends the command on Ctrl-C.
fn stop_on_interrupt() -> CancellationToken {
    let stop = CancellationToken::new();
    let ring = stop.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            ring.cancel();
        }
    });
    stop
}

/// Writes one wake line to stdout, flushed, and reports a closed reader instead of panicking.
fn print_line(line: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    writeln!(out, "{line}")?;
    out.flush()
}

async fn run_wake_stream(args: Vec<String>) -> Result<u8, String> {
    let io = Io {
        out: Box::new(print_line),
        err: Box::new(|line| eprintln!("{line}")),
    };
    Ok(wake_stream::run(&args, &lifecycle::paths_of(None), io, stop_on_interrupt()).await)
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => return lifecycle::refuse::<Cli>(&error),
    };
    let done = match cli.command {
        Command::Lifecycle(verb) => lifecycle::run(verb).await,
        Command::WakeStream { args } => run_wake_stream(args).await,
        Command::Tray { action, data_dir } => {
            let action = action.unwrap_or(TrayAction::Start);
            tray::command(action, lifecycle::paths_of(data_dir)).await
        }
    };
    match done {
        Ok(0) => ExitCode::SUCCESS,
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("argus: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::Cli;

    #[test]
    fn the_command_line_is_well_formed() {
        Cli::command().debug_assert();
    }

    #[test]
    fn the_tray_verbs_and_its_home_parse() {
        for line in [
            "argus tray",
            "argus tray run --data-dir x",
            "argus tray --data-dir x stop",
            "argus tray status",
            "argus tray start",
        ] {
            assert!(Cli::try_parse_from(line.split(' ')).is_ok(), "{line}");
        }
        assert!(Cli::try_parse_from(["argus", "tray", "quit"]).is_err());
    }
}
