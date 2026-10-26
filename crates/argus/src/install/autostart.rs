//! `argus autostart install|remove|status`: start the dashboard at Windows login by dropping one
//! hidden `.vbs` launcher in the Startup folder. Explicit only: no scheduled task, no registry
//! edit, and no other command creates it. Not recommended and not advertised: the default route
//! is the tray (`argus tray start`) and the stdio MCP bridge, which start the dashboard when
//! Claude starts.

use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use super::env::{Env, Io, Platform};
use super::vbs::{content, decode, encode};

/// The name of the launcher file.
pub const VBS_NAME: &str = "coord-dashboard.vbs";

const USAGE: &str = "Usage: argus autostart install|remove|status\n(login autostart is not recommended: the dashboard starts with Claude, and `argus tray start` shows the tray icon)";

/// The Startup folder of the user.
#[must_use]
pub fn startup_dir(env: &Env) -> PathBuf {
    env.roaming()
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
}

fn status(file: &PathBuf, wanted: &str, io: &mut Io<'_>) -> u8 {
    let Ok(bytes) = fs::read(file) else {
        (io.out)(&format!("not installed ({})", file.display()));
        return 1;
    };
    let note = if decode(&bytes) == wanted {
        ""
    } else {
        " (but it points somewhere else; run install to refresh it)"
    };
    (io.out)(&format!("installed{note}: {}", file.display()));
    0
}

fn remove(file: &PathBuf, io: &mut Io<'_>) -> u8 {
    match fs::remove_file(file) {
        Ok(()) => (io.out)(&format!("removed {}", file.display())),
        Err(e) if e.kind() == ErrorKind::NotFound => (io.out)("nothing to remove"),
        Err(e) => {
            (io.err)(&format!("argus: {} cannot be removed: {e}", file.display()));
            return 1;
        }
    }
    0
}

fn install(file: &PathBuf, wanted: &str, io: &mut Io<'_>) -> u8 {
    let written = file
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(file, encode(wanted)));
    if let Err(e) = written {
        (io.err)(&format!("argus: {} cannot be written: {e}", file.display()));
        return 1;
    }
    (io.out)(&format!(
        "installed {}: the dashboard starts hidden at your next login. Remove it with: argus autostart remove. Login autostart is not recommended: the dashboard starts when Claude starts (MCP bridge) and the tray is `argus tray start`.",
        file.display()
    ));
    0
}

/// Runs the verb and answers the exit code: 0 done, 1 a failure or `status` of nothing, 2 usage.
pub fn run(args: &[String], io: &mut Io<'_>) -> u8 {
    let action = args.first().map(String::as_str);
    if !matches!(action, Some("install" | "remove" | "status")) {
        (io.err)(USAGE);
        return 2;
    }
    if io.env.platform != Platform::Windows {
        (io.err)("argus: autostart is Windows only (a Startup folder script).");
        return 1;
    }
    let file = startup_dir(&io.env).join(VBS_NAME);
    let wanted = content(&io.env.exe.display().to_string());
    match action {
        Some("status") => status(&file, &wanted, io),
        Some("remove") => remove(&file, io),
        _ => install(&file, &wanted, io),
    }
}
