//! The one way Argus starts a child process: no console window on Windows.
//!
//! Clippy forbids `Command::new` everywhere else (see the root `clippy.toml`).

use std::ffi::OsStr;
use std::process::Command;

/// The Windows process creation flag that keeps a console child from opening a window.
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Keeps `command` from opening a console window. Does nothing off Windows.
fn hide(command: &mut Command) {
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(command, CREATE_NO_WINDOW);
}

/// A command for `program` that never opens a console window. Wrap it with
/// `tokio::process::Command::from` for the async flavour.
#[must_use]
#[allow(clippy::disallowed_methods)] // the one place a bare `Command::new` is allowed
pub fn hidden_command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    hide(&mut command);
    command
}
