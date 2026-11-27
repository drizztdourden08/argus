//! A [`DesktopApp`] over a child process, for tests that run the real switch without the real
//! Claude. `start` spawns the given program, `close` writes a quit file the program watches and
//! waits for the app's lock to go, a forced close ends only the child it spawned, and the
//! running check is the same lock check the real app gets.

use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use argus_core::process::hidden_command;

use crate::desktop::app::{await_quit, config_written_after, wait_until};
use crate::ports::{DesktopApp, Quit};
use crate::sync::lock_running;

/// The program to run as the app.
#[derive(Debug, Clone)]
pub struct Launch {
    /// The program.
    pub exe: PathBuf,
    /// Its arguments.
    pub args: Vec<String>,
    /// The file `close` writes to ask it to quit; removed before each start.
    pub quit_file: PathBuf,
    /// The version `version` answers.
    pub version: Option<String>,
}

/// The waits of a close and a start, short for tests.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// How long a close waits for the quit before it ends the child (with `force`).
    pub grace: Duration,
    /// How long a close waits in all before it gives up.
    pub limit: Duration,
    /// How long a start waits for the lock.
    pub start: Duration,
    /// How often the waits look.
    pub poll: Duration,
}

/// A child process standing in for Claude Desktop.
pub struct ProcessApp {
    app: PathBuf,
    launch: Launch,
    limits: Limits,
    child: Mutex<Option<Child>>,
}

impl ProcessApp {
    /// The app whose data lives in `app`, run as `launch` says.
    #[must_use]
    pub fn new(app: &Path, launch: Launch, limits: Limits) -> Self {
        ProcessApp {
            app: app.to_path_buf(),
            launch,
            limits,
            child: Mutex::new(None),
        }
    }

    fn held(&self) -> MutexGuard<'_, Option<Child>> {
        self.child.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether the child ended on its own (or was never started).
    fn exited(&self) -> bool {
        self.held()
            .as_mut()
            .is_none_or(|child| child.try_wait().ok().flatten().is_some())
    }

    /// Ends the child this app spawned, by its own handle and never by name, and reaps it.
    pub fn kill(&self) {
        if let Some(child) = self.held().as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl DesktopApp for ProcessApp {
    fn is_running(&self) -> bool {
        lock_running(&self.app)
    }

    fn turn_in_progress(&self) -> bool {
        false
    }

    fn close(&self, quit: &mut Quit) -> Result<(), String> {
        if !self.is_running() {
            return Ok(());
        }
        std::fs::write(&self.launch.quit_file, b"quit")
            .map_err(|e| format!("cannot write the quit file: {e}"))?;
        let limits = (self.limits.grace, self.limits.limit, self.limits.poll);
        await_quit(quit, limits, &|| !self.is_running(), &mut || {
            self.kill();
        })
    }

    fn start(&self) -> Result<(), String> {
        let _ = std::fs::remove_file(&self.launch.quit_file);
        let child = hidden_command(&self.launch.exe)
            .args(&self.launch.args)
            .spawn()
            .map_err(|e| format!("the app did not start: {e}"))?;
        *self.held() = Some(child);
        let up = wait_until(self.limits.start, self.limits.poll, || {
            self.is_running() || self.exited()
        });
        if up && self.is_running() {
            Ok(())
        } else {
            Err("the app did not start".to_owned())
        }
    }

    fn config_touched_since(&self, since_ms: i64) -> bool {
        config_written_after(&self.app, since_ms)
    }

    fn version(&self) -> Option<String> {
        self.launch.version.clone()
    }
}
