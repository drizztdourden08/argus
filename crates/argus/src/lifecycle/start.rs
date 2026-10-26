//! `argus start`: the server hidden in the background, its address printed once it answers.
//! A server that answers already is left alone. The server's standard error goes to
//! `logs/start.err` until it is up, so a start that fails says why.

use std::ffi::OsString;
use std::fs::{self, File};
use std::process::Child;
use std::time::Duration;

use argus_core::Paths;
use argus_core::config::{env_port, load_config};
use argus_native::{TrayFiles, clear_quit_flag};
use tokio::time::sleep;

use super::live::{Up, up};
use super::spawn;

/// How long `start` waits for the server by default.
pub const START_WAIT: Duration = Duration::from_secs(8);
const STEP: Duration = Duration::from_millis(100);

/// What the caller asked of `start`.
pub struct Options {
    /// The port of the new server; none leaves it to the config.
    pub port: Option<u16>,
    /// How long to wait for it.
    pub wait: Duration,
    /// Print nothing on success (the caller prints its own line).
    pub quiet: bool,
}

enum Outcome {
    Up(Up),
    Exited,
    TimedOut,
}

/// Waits for the server to answer, to exit or for the time to pass.
async fn wait_for(paths: &Paths, child: &mut Child, limit: Duration) -> Outcome {
    let mut waited = Duration::ZERO;
    while waited < limit {
        sleep(STEP).await;
        waited += STEP;
        if let Some(up) = up(paths).await {
            return Outcome::Up(up);
        }
        if child.try_wait().ok().flatten().is_some() {
            return Outcome::Exited;
        }
    }
    Outcome::TimedOut
}

fn serve_args(port: Option<u16>) -> Vec<OsString> {
    let mut args = vec![OsString::from("serve")];
    if let Some(port) = port {
        args.push("--port".into());
        args.push(port.to_string().into());
    }
    args
}

fn failure(err_file: &std::path::Path, limit: Duration) -> String {
    let why = fs::read_to_string(err_file).unwrap_or_default();
    let why = why.trim();
    let reason = if why.is_empty() {
        format!(" within {} s", limit.as_secs())
    } else {
        format!(": {why}")
    };
    format!("dashboard: the server did not come up{reason}. See: argus logs")
}

/// Starts the server unless one answers, and waits for it.
///
/// # Errors
///
/// The config is refused, or the server could not be started at all.
pub async fn start(paths: &Paths, options: &Options) -> Result<u8, String> {
    // Starting the server is how the owner reopens a dashboard quit from the tray.
    let _cleared = clear_quit_flag(&TrayFiles::new(&paths.home));
    let env = env_port().map_err(|e| e.to_string())?;
    load_config(paths, env).map_err(|e| e.to_string())?;
    if let Some(live) = up(paths).await {
        if !options.quiet {
            let line = format!(
                "dashboard already running at {}/ (pid {})",
                live.at.base(),
                live.at.pid_text()
            );
            println!("{line}");
        }
        return Ok(0);
    }
    fs::create_dir_all(&paths.logs).map_err(|e| e.to_string())?;
    let err_file = paths.logs.join("start.err");
    let sink = File::create(&err_file).map_err(|e| e.to_string())?;
    let mut child = spawn::detached(&serve_args(options.port), &sink)
        .map_err(|e| format!("the server could not be started: {e}"))?;
    drop(sink);
    match wait_for(paths, &mut child, options.wait).await {
        Outcome::Up(live) => {
            if !options.quiet {
                let pid = live.at.record.as_ref().map_or(child.id(), |r| r.pid);
                println!("dashboard running at {}/ (pid {pid})", live.at.base());
            }
            let _removed = fs::remove_file(&err_file);
            Ok(0)
        }
        Outcome::Exited | Outcome::TimedOut => {
            eprintln!("{}", failure(&err_file, options.wait));
            Ok(1)
        }
    }
}
