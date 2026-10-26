//! The parent watch. Under the shim a Node parent waits on this listener for days; it passes
//! its pid in `ARGUS_COORD_PARENT`, and the listener ends (exit 0, silent) within five seconds
//! of that process going away, so a killed hook leaves no orphan holding a wake stream.

use std::thread;
use std::time::Duration;

use super::super::alive::pid_alive;
use crate::cli::Env;

const EVERY: Duration = Duration::from_secs(5);

/// Starts the watch when `ARGUS_COORD_PARENT` names a process; nothing otherwise.
pub fn watch(env: &Env) {
    let Some(parent) = env
        .get("ARGUS_COORD_PARENT")
        .and_then(|p| p.trim().parse::<u32>().ok())
    else {
        return;
    };
    thread::spawn(move || {
        loop {
            thread::sleep(EVERY);
            if !pid_alive(parent) {
                std::process::exit(0);
            }
        }
    });
}
