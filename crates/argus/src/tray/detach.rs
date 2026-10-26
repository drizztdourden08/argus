//! Processes the tray starts: hidden, without a console, not waited for.

use std::ffi::OsStr;
use std::io;
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

const DETACHED_PROCESS: u32 = 0x0000_0008;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;

fn quiet(program: impl AsRef<OsStr>, args: &[&OsStr], flags: u32) -> io::Result<()> {
    Command::new(program)
        .args(args)
        .creation_flags(flags)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(drop)
}

/// Starts `argus <args>` apart from this process: it outlives the tray and the job it runs in.
/// A job that does not allow breakaway gets the process without it.
///
/// # Errors
///
/// The program could not be started.
pub fn argus(args: &[&OsStr]) -> io::Result<()> {
    argus_native::keep_stdio_private();
    let exe = std::env::current_exe()?;
    let flags = DETACHED_PROCESS | CREATE_NO_WINDOW;
    quiet(&exe, args, flags | CREATE_BREAKAWAY_FROM_JOB).or_else(|_| quiet(&exe, args, flags))
}

/// Starts a program with no window and does not wait for it.
///
/// # Errors
///
/// The program could not be started.
pub fn hidden(program: &str, args: &[&OsStr]) -> io::Result<()> {
    quiet(program, args, CREATE_NO_WINDOW)
}

/// Starts a program that shows its own window (the folder of the logs) and does not wait.
///
/// # Errors
///
/// The program could not be started.
pub fn windowed(program: &str, args: &[&OsStr]) -> io::Result<()> {
    quiet(program, args, 0)
}

/// Ends the process `pid` and its children.
pub fn kill(pid: u32) {
    let pid = pid.to_string();
    let args = ["/PID", &pid, "/T", "/F"].map(OsStr::new);
    let _ended = Command::new("taskkill")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
