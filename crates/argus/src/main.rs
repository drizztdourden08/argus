//! The `argus` binary. `argus serve` runs the server in the foreground; the other verbs of the
//! Node `dashboard` bin arrive in later slices.
#![deny(clippy::cognitive_complexity)]

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use argus::wake_stream::{self, Io};
use argus_core::Paths;
use argus_server::{ServeOptions, serve};
use clap::{Parser, Subcommand};
use tokio_util::sync::CancellationToken;

/// The port of `argus serve` while the Node server keeps 47615.
const DEFAULT_PORT: u16 = 47616;

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
    /// Runs the server in the foreground.
    Serve {
        /// The port (0 picks a free one).
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
        /// The dashboard home (config, database, logs); default `<COORD_HOME>/dashboard`.
        #[arg(long, alias = "home")]
        data_dir: Option<PathBuf>,
        /// Shadow mode: read the boards, never write them.
        #[arg(long)]
        shadow: bool,
    },
    /// Prints one line per wake of a project's wake stream (for a Monitor); see
    /// `--project <key> [--thread <id|*>] [--id <label>]`.
    WakeStream {
        /// The flags, read by the wake-stream command itself.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

async fn run_serve(port: u16, data_dir: Option<PathBuf>, shadow: bool) -> Result<(), String> {
    let env = Paths::from_env();
    let home = data_dir.map(|d| std::path::absolute(&d).unwrap_or(d));
    let paths = Paths::new(env.coord_home, home.or(Some(env.home)));
    let mut opts = ServeOptions::new(paths);
    opts.port = Some(port);
    opts.shadow = shadow;
    let server = serve(opts).await.map_err(|e| e.to_string())?;
    println!(
        "argus listening on {}{}",
        server.url,
        if shadow { " (shadow)" } else { "" }
    );
    let svc = std::sync::Arc::clone(&server.svc);
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            svc.stop.cancel();
        }
    });
    server.wait().await;
    Ok(())
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

async fn run_wake_stream(args: Vec<String>) -> ExitCode {
    let io = Io {
        out: Box::new(print_line),
        err: Box::new(|line| eprintln!("{line}")),
    };
    let code = wake_stream::run(&args, &Paths::from_env(), io, stop_on_interrupt()).await;
    ExitCode::from(code)
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let done = match cli.command {
        Command::Serve {
            port,
            data_dir,
            shadow,
        } => run_serve(port, data_dir, shadow).await,
        Command::WakeStream { args } => return run_wake_stream(args).await,
    };
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("argus: {e}");
            ExitCode::FAILURE
        }
    }
}
