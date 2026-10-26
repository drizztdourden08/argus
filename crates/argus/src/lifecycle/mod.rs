//! The server lifecycle verbs of the Node `dashboard` bin: `serve`, `start`, `stop`, `status`,
//! `open`, `logs`, `resync` and `token`. Only `serve` runs the server; every other verb talks
//! HTTP to the one the files of the dashboard home name, so they are quick and work while the
//! server is down. Output lines and exit codes are the Node ones.

mod admin;
mod live;
mod logs;
mod open;
mod serve;
mod spawn;
mod start;
mod status;
mod stop;
mod tray_line;

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use argus_core::Paths;
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Command, CommandFactory, Subcommand};

pub use serve::paths_of;

/// The verbs.
#[derive(Subcommand)]
pub enum Verb {
    /// Runs the server in the foreground.
    Serve {
        /// The port (0 picks a free one); default `DASHBOARD_PORT`, else 47616.
        #[arg(long)]
        port: Option<u16>,
        /// The dashboard home (config, database, logs); default `<COORD_HOME>/dashboard`.
        #[arg(long, alias = "home")]
        data_dir: Option<PathBuf>,
        /// Shadow mode: read the boards, never write them.
        #[arg(long)]
        shadow: bool,
    },
    /// Starts the server hidden in the background and prints its address.
    Start {
        /// Runs the server in the foreground instead.
        #[arg(long)]
        foreground: bool,
        /// The port of the new server; default `DASHBOARD_PORT`, else the configured one.
        #[arg(long)]
        port: Option<u16>,
        /// Seconds to wait for the server to answer (default 8).
        #[arg(long, value_name = "SECONDS")]
        wait: Option<u64>,
    },
    /// Stops the server.
    Stop,
    /// Health, pid, port and runs; exit 1 when the server is down.
    Status {
        /// Prints JSON.
        #[arg(long)]
        json: bool,
    },
    /// Opens Argus (the overview) or a board in the browser.
    Open {
        /// A project key.
        key: Option<String>,
        /// Only prints the address.
        #[arg(long)]
        print: bool,
    },
    /// The server log; with `--run R-n --project <key>` a run's log.
    Logs {
        /// How many lines (default 50).
        #[arg(long)]
        lines: Option<String>,
        /// Keeps printing.
        #[arg(long)]
        follow: bool,
        /// A run id.
        #[arg(long)]
        run: Option<String>,
        /// The project of the run.
        #[arg(long)]
        project: Option<String>,
    },
    /// Asks the server to read its boards again.
    Resync {
        /// One project (default all).
        #[arg(long)]
        project: Option<String>,
        /// Rebuilds the docs from the boards.
        #[arg(long)]
        hard: bool,
    },
    /// A new token (it only matters with `requireToken` in config.json).
    Token {
        /// Rotates it.
        #[arg(long)]
        rotate: bool,
    },
}

async fn start_verb(
    paths: &Paths,
    (foreground, port, wait): (bool, Option<u16>, Option<u64>),
) -> Result<u8, String> {
    if foreground {
        return serve::run(port, None, false).await;
    }
    let options = start::Options {
        port,
        wait: wait.map_or(start::START_WAIT, Duration::from_secs),
        quiet: false,
    };
    start::start(paths, &options).await
}

/// Does what the verb asks and answers the exit code.
///
/// # Errors
///
/// The verb failed in a way it did not print itself.
pub async fn run(verb: Verb) -> Result<u8, String> {
    let paths = paths_of(None);
    match verb {
        Verb::Serve {
            port,
            data_dir,
            shadow,
        } => serve::run(port, data_dir, shadow).await,
        Verb::Start {
            foreground,
            port,
            wait,
        } => start_verb(&paths, (foreground, port, wait)).await,
        Verb::Stop => stop::stop(&paths).await,
        Verb::Status { json } => status::status(&paths, json).await,
        Verb::Open { key, print } => open::open(&paths, key.as_deref(), print).await,
        Verb::Logs {
            lines,
            follow,
            run,
            project,
        } => {
            let request = logs::Request {
                lines: lines.as_deref(),
                follow,
                run: run.as_deref(),
                project: project.as_deref(),
            };
            logs::logs(&paths, &request).await
        }
        Verb::Resync { project, hard } => admin::resync(&paths, project.as_deref(), hard).await,
        Verb::Token { rotate } => admin::token(&paths, rotate).await,
    }
}

/// The text of a parse failure in the Node form: an unknown command is named, and every other
/// refusal is clap's own.
fn unknown_command(error: &clap::Error) -> Option<String> {
    if error.kind() != ErrorKind::InvalidSubcommand {
        return None;
    }
    match error.get(ContextKind::InvalidSubcommand)? {
        ContextValue::String(name) => Some(name.clone()),
        _ => None,
    }
}

/// Reports a command line that did not parse and answers its exit code: 0 for help and version,
/// 2 for the rest.
#[must_use]
pub fn refuse<C: CommandFactory>(error: &clap::Error) -> ExitCode {
    let Some(name) = unknown_command(error) else {
        let _printed = error.print();
        return ExitCode::from(u8::try_from(error.exit_code()).unwrap_or(2));
    };
    let mut command: Command = C::command();
    let usage = command.render_usage();
    eprintln!("argus: unknown command \"{name}\"\n{usage}");
    ExitCode::from(2)
}
