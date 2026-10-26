//! The `argus` binary. `argus coord <verb>` is the coord CLI and is dispatched before anything
//! else starts; `argus serve` runs the server in the foreground and headless; `argus tray`
//! runs the tray icon that starts and quits it; `argus mcp` is the stdio MCP server Claude
//! spawns; `argus install-mcp` and `argus autostart` write configuration for this binary. The
//! other verbs of the Node `dashboard` bin arrive in later slices.
#![deny(clippy::cognitive_complexity)]

mod tray;

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use argus::install::{self, Env};
use argus::wake_stream::{self, Io};
use argus_core::Paths;
use argus_server::{ServeOptions, serve};
use clap::{Parser, Subcommand};
use tokio_util::sync::CancellationToken;
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
    /// Prints one line per wake of a project's wake stream (for a Monitor); see
    /// `--project <key> [--thread <id|*>] [--id <label>]`.
    WakeStream {
        /// The flags, read by the wake-stream command itself.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// The stdio MCP server Claude spawns: answers `initialize` and `tools/list` itself, forwards
    /// the rest to the running server and starts it when it is down.
    Mcp {
        /// The default project of every tool call.
        #[arg(long)]
        project: Option<String>,
        /// The dashboard home (config, database, logs); default `<COORD_HOME>/dashboard`.
        #[arg(long, alias = "home")]
        data_dir: Option<PathBuf>,
    },
    /// Prints, or with `--write` merges, the Claude Desktop entry for `argus mcp`; `--code`
    /// prints the `.mcp.json` entry for a folder: `[--desktop [--write] [--config <file>]]
    /// [--code [--dir <folder>] [--project <key>]]`.
    #[command(disable_help_flag = true)]
    InstallMcp {
        /// The flags, read by the install-mcp command itself.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Starts the dashboard at Windows login with a hidden launcher in the Startup folder:
    /// `install`, `remove` or `status`. Not recommended: the dashboard starts with Claude.
    #[command(disable_help_flag = true)]
    Autostart {
        /// The action, read by the autostart command itself.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Compares two dashboard servers by value, read-only: the shadow check of the switch-over.
    Parity(argus_parity::Args),
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
    Ok(wake_stream::run(&args, &paths_of(None), io, stop_on_interrupt()).await)
}

/// Runs an install verb on the real machine, its answer on stdout and the rest on stderr.
fn run_install(verb: fn(&[String], &mut install::Io<'_>) -> u8, args: &[String]) -> u8 {
    let mut out = |line: &str| println!("{line}");
    let mut err = |line: &str| eprintln!("{line}");
    let mut io = install::Io {
        out: &mut out,
        err: &mut err,
        env: Env::process(),
    };
    verb(args, &mut io)
}

/// The commands that need the async runtime: serve, wake-stream and the tray.
async fn run_cli() -> ExitCode {
    let cli = Cli::parse();
    let done = match cli.command {
        Command::Serve {
            port,
            data_dir,
            shadow,
        } => run_serve(port, data_dir, shadow).await,
        Command::WakeStream { args } => run_wake_stream(args).await,
        Command::Mcp { project, data_dir } => argus::mcp::run(paths_of(data_dir), project).await,
        Command::InstallMcp { args } => Ok(run_install(install::mcp_config::run, &args)),
        Command::Autostart { args } => Ok(run_install(install::autostart::run, &args)),
        Command::Parity(args) => argus_parity::run(args).await,
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

/// `argus coord ...`: the arguments after `coord`, when that is what was typed.
fn coord_args() -> Option<Vec<String>> {
    let mut args = std::env::args_os().skip(1);
    if args.next()? != "coord" {
        return None;
    }
    Some(args.map(|a| a.to_string_lossy().into_owned()).collect())
}

fn main() -> ExitCode {
    if let Some(argv) = coord_args() {
        std::process::exit(argus_coord::run_process(argv));
    }
    match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime.block_on(run_cli()),
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
    fn the_parity_flags_parse() {
        let line = "argus parity --left http://a --right http://b --project x --project y --json";
        assert!(Cli::try_parse_from(line.split(' ')).is_ok());
        assert!(Cli::try_parse_from(["argus", "parity", "--left", "http://a"]).is_err());
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

    #[test]
    fn the_mcp_install_and_autostart_verbs_parse() {
        for line in [
            "argus mcp",
            "argus mcp --project rotp",
            "argus mcp --project rotp --data-dir x",
            "argus install-mcp --desktop --write --config f.json",
            "argus install-mcp --code --dir d --project k",
            "argus install-mcp --help",
            "argus autostart status",
            "argus autostart bogus",
        ] {
            assert!(Cli::try_parse_from(line.split(' ')).is_ok(), "{line}");
        }
    }
}
