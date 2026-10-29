//! `argus tray run | start | stop | status` (DESIGN 7.6): the icon in the notification area, on
//! its own process. `run` is that process; `start` launches it detached; `stop` closes the icon
//! only; `status` says whether it runs. The tray starts the server unless the owner quit the
//! dashboard from it during this boot. Windows only.

use argus_core::Paths;
use clap::Subcommand;

#[cfg(windows)]
mod log;
#[cfg(windows)]
mod menu;
#[cfg(windows)]
mod open;
#[cfg(windows)]
mod run;
#[cfg(windows)]
mod server;
#[cfg(windows)]
mod start;
#[cfg(windows)]
mod status;
#[cfg(windows)]
mod stop;

/// What to do with the tray.
#[derive(Subcommand, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    /// Runs the tray icon in this process until it is closed.
    Run,
    /// Starts the tray icon in the background, and with it the server (the default).
    Start,
    /// Closes the tray icon only; the server keeps running.
    Stop,
    /// Says whether the tray icon runs.
    Status,
}

/// `--shadow`: the home becomes a shadow for good (seeded from the Node home when it has no
/// database yet, and marked in its `config.json`), before the tray or the server comes up.
#[cfg(windows)]
fn make_shadow(paths: &Paths) -> Result<(), String> {
    let node_home = Paths::from_env().home;
    argus_server::shadow::prepare(paths, Some(&node_home)).map_err(|e| e.to_string())
}

/// Does what `argus tray` was asked and answers the exit code.
///
/// # Errors
///
/// The tray could not start or stop, or this is not Windows.
#[cfg(windows)]
pub async fn command(action: TrayAction, paths: Paths, shadow: bool) -> Result<u8, String> {
    if shadow && matches!(action, TrayAction::Run | TrayAction::Start) {
        make_shadow(&paths)?;
    }
    match action {
        TrayAction::Run => run::run(paths).await,
        TrayAction::Start => start::start(&paths).await,
        TrayAction::Stop => stop::stop(&paths),
        TrayAction::Status => Ok(status::status(&paths)),
    }
}

/// Does what `argus tray` was asked and answers the exit code.
///
/// # Errors
///
/// This is not Windows.
#[cfg(not(windows))]
pub fn command(
    _action: TrayAction,
    _paths: Paths,
    _shadow: bool,
) -> std::future::Ready<Result<u8, String>> {
    let why = "the tray icon is Windows only; use `argus serve` for the server";
    std::future::ready(Err(why.to_owned()))
}
