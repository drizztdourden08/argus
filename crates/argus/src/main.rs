//! The `argus` binary. `argus coord <verb>` is the coord CLI and is dispatched before anything
//! else starts; `argus serve` runs the server in the foreground and headless; `argus start`,
//! `stop`, `status`, `open`, `logs`, `resync` and `token` are the lifecycle verbs; `argus tray`
//! runs the tray icon that starts and quits it; `argus mcp` is the stdio MCP server Claude
//! spawns; `argus install-mcp` and `argus autostart` write configuration for this binary. The
//! other verbs of the Node `dashboard` bin arrive in later slices.
#![deny(clippy::cognitive_complexity)]

mod lifecycle;
mod node_form;
mod tray;

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use argus::install::{self, Env};
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
    /// Copies this binary to a stable folder (default `%LOCALAPPDATA%Programsrgus`) and prints
    /// the `"exe"` config line: `[--dir <folder>] [--print]`. Edits no config.
    #[command(disable_help_flag = true)]
    SelfInstall {
        /// The flags, read by the self-install command itself.
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
        /// Makes the home a shadow home for good (the `"shadow": true` marker in its
        /// config.json): the server behind the tray never writes the boards.
        #[arg(long, global = true)]
        shadow: bool,
    },
}

/// How long the exit waits for blocking work that is still running.
const EXIT_GRACE: Duration = Duration::from_millis(500);

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
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => return lifecycle::refuse::<Cli>(&error),
    };
    let done = match cli.command {
        Command::Lifecycle(verb) => lifecycle::run(verb).await,
        Command::WakeStream { args } => run_wake_stream(args).await,
        Command::Mcp { project, data_dir } => {
            argus::mcp::run(lifecycle::paths_of(data_dir), project).await
        }
        Command::InstallMcp { args } => Ok(run_install(install::mcp_config::run, &args)),
        Command::Autostart { args } => Ok(run_install(install::autostart::run, &args)),
        Command::SelfInstall { args } => Ok(run_install(install::self_install::run, &args)),
        Command::Parity(args) => argus_parity::run(args).await,
        Command::Tray {
            action,
            data_dir,
            shadow,
        } => {
            let action = action.unwrap_or(TrayAction::Start);
            tray::command(action, lifecycle::paths_of(data_dir), shadow).await
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

/// The Node `dashboard` output for a call the shim forwarded, when the command only prints.
fn shim_answer() -> Option<ExitCode> {
    if !node_form::forwarded() {
        return None;
    }
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let done = node_form::answer(&args, env!("CARGO_PKG_VERSION"))?;
    print!("{}", done.stdout);
    eprint!("{}", done.stderr);
    Some(ExitCode::from(done.code))
}

fn main() -> ExitCode {
    if let Some(argv) = coord_args() {
        std::process::exit(argus_coord::run_process(argv));
    }
    if let Some(done) = shim_answer() {
        return done;
    }
    on_runtime(run_cli())
}

/// Runs `work` on a fresh runtime and leaves without waiting long for the blocking work still
/// running: a worktree scan that measures the disk can take minutes, and a dropped runtime waits
/// for it, so a stopped server's process lingered. Node leaves the process 50 ms after its stop
/// in the same way.
fn on_runtime(work: impl Future<Output = ExitCode>) -> ExitCode {
    match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => {
            let code = runtime.block_on(work);
            runtime.shutdown_timeout(EXIT_GRACE);
            code
        }
        Err(e) => {
            eprintln!("argus: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use std::process::ExitCode;
    use std::time::{Duration, Instant};

    use clap::{CommandFactory, Parser};

    use super::{Cli, on_runtime};

    #[test]
    fn the_exit_does_not_wait_for_blocking_work() {
        let begun = Instant::now();
        let code = on_runtime(async {
            let _left = tokio::task::spawn_blocking(|| std::thread::sleep(Duration::from_secs(30)));
            ExitCode::SUCCESS
        });
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(
            begun.elapsed() < Duration::from_secs(5),
            "{:?}",
            begun.elapsed()
        );
    }

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
            "argus tray start --shadow --data-dir x",
            "argus tray --shadow",
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
            "argus self-install --dir d --print",
            "argus autostart status",
            "argus autostart bogus",
        ] {
            assert!(Cli::try_parse_from(line.split(' ')).is_ok(), "{line}");
        }
    }
}
