//! The processes the lifecycle verbs start: the server detached and hidden, the browser, and the
//! end of a server's process tree by its pid (never by image name).

use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::process::{Child, Command, Stdio};

use argus_core::process::hidden_command;

#[cfg(windows)]
mod flags {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    use argus_core::process::CREATE_NO_WINDOW;

    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;

    /// Apart from this process: no console, no window, outside the job it runs in.
    pub fn detached(command: &mut Command, breakaway: bool) {
        let more = if breakaway {
            CREATE_BREAKAWAY_FROM_JOB
        } else {
            0
        };
        command.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW | more);
    }

    /// No window.
    pub fn hidden(command: &mut Command) {
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

#[cfg(not(windows))]
mod flags {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    /// Its own process group, so a Ctrl-C of the caller does not reach it.
    pub fn detached(command: &mut Command, _breakaway: bool) {
        command.process_group(0);
    }

    /// Nothing to hide off Windows.
    pub fn hidden(_command: &mut Command) {}
}

fn command(args: &[OsString], breakaway: bool, stderr: &File) -> io::Result<Command> {
    let mut command = hidden_command(std::env::current_exe()?);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr.try_clone()?);
    flags::detached(&mut command, breakaway);
    Ok(command)
}

/// Starts `argus <args>` apart from this process, with its standard error in `stderr`. A job
/// that does not allow breakaway gets the process without it.
///
/// # Errors
///
/// The program could not be started.
pub fn detached(args: &[OsString], stderr: &File) -> io::Result<Child> {
    argus_native::keep_stdio_private();
    command(args, true, stderr)?
        .spawn()
        .or_else(|_| command(args, false, stderr)?.spawn())
}

/// Opens an address in the default browser, with no window of its own.
///
/// # Errors
///
/// The opener could not be started.
pub fn browser(url: &str) -> io::Result<()> {
    let (program, args): (&str, Vec<&str>) = if cfg!(windows) {
        ("rundll32", vec!["url.dll,FileProtocolHandler", url])
    } else if cfg!(target_os = "macos") {
        ("open", vec![url])
    } else {
        ("xdg-open", vec![url])
    };
    let mut command = hidden_command(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    flags::hidden(&mut command);
    command.spawn().map(drop)
}

/// Ends the process `pid` and its children. Only for a pid that was a confirmed dashboard a
/// moment ago.
pub fn kill_tree(pid: u32) {
    let pid = pid.to_string();
    let mut command = if cfg!(windows) {
        let mut c = hidden_command("taskkill");
        c.args(["/PID", &pid, "/T", "/F"]);
        c
    } else {
        let mut c = hidden_command("kill");
        c.args(["-9", &pid]);
        c
    };
    command.stdout(Stdio::null()).stderr(Stdio::null());
    flags::hidden(&mut command);
    let _ended = command.status();
}
