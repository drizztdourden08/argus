//! The `argus` binary. `argus serve` runs the server in the foreground and headless; `argus tray`
//! runs the tray icon that starts and quits it; the other verbs of the Node `dashboard` bin
//! arrive in later slices.
#![deny(clippy::cognitive_complexity)]

mod tray;

use std::path::PathBuf;
use std::process::ExitCode;

use argus_core::Paths;
use argus_server::{ServeOptions, serve};
use clap::{Parser, Subcommand};
use tray::TrayAction;

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
    /// The tray icon (Windows): run, start, stop or status; start is the default.
    Tray {
        #[command(subcommand)]
        action: Option<TrayAction>,
        /// The dashboard home (config, database, logs); default `<COORD_HOME>/dashboard`.
        #[arg(long, alias = "home", global = true)]
        data_dir: Option<PathBuf>,
    },
}

/// The folders of the dashboard: `COORD_HOME` from the environment, and the given home or the
/// default one.
fn paths_of(data_dir: Option<PathBuf>) -> Paths {
    let env = Paths::from_env();
    let home = data_dir.map(|d| std::path::absolute(&d).unwrap_or(d));
    Paths::new(env.coord_home, home.or(Some(env.home)))
}

async fn run_serve(port: u16, data_dir: Option<PathBuf>, shadow: bool) -> Result<u8, String> {
    let mut opts = ServeOptions::new(paths_of(data_dir));
    opts.port = Some(port);
    opts.shadow = shadow;
    opts.notify.system = true;
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
    Ok(0)
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
        Command::Tray { action, data_dir } => {
            let action = action.unwrap_or(TrayAction::Start);
            tray::command(action, paths_of(data_dir)).await
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
