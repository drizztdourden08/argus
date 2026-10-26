//! The handle `argus serve` and `argus tray` own: the tray runs on its own thread and is
//! controlled from here.

use std::sync::Arc;
use std::sync::mpsc::{Sender, channel};
use std::thread::{Builder, JoinHandle};

use super::event_loop::{Control, ExitReason, run};
use super::handlers::Handlers;
use crate::tray::balloon::Balloon;
use crate::tray::error::TrayError;
use crate::tray::menu::Command;
use crate::tray::paths::TrayFiles;
use crate::tray::state::TrayState;

/// A running tray icon.
pub struct TrayHandle {
    controls: Sender<Control>,
    handlers: Arc<Handlers>,
    thread: Option<JoinHandle<ExitReason>>,
}

impl TrayHandle {
    /// Starts the tray for a dashboard home and returns once the icon is up. The icon shows
    /// "checking" until the first `set_state`.
    ///
    /// # Errors
    ///
    /// `AlreadyRunning` when another tray holds the icon for this home, `Io` when `tray.json`
    /// cannot be written, `Platform` when Windows or the icon library refuses.
    pub fn start(files: TrayFiles) -> Result<Self, TrayError> {
        let (controls, inbox) = channel();
        let (ready_tx, ready_rx) = channel();
        let handlers = Arc::new(Handlers::default());
        let shared = Arc::clone(&handlers);
        let thread = Builder::new()
            .name("argus-tray".to_owned())
            .spawn(move || run(files, shared, &inbox, &ready_tx))?;
        let started = ready_rx
            .recv()
            .unwrap_or_else(|_| Err(TrayError::Platform("the tray thread ended".to_owned())));
        let handle = Self {
            controls,
            handlers,
            thread: Some(thread),
        };
        match started {
            Ok(()) => Ok(handle),
            Err(error) => {
                let _ended = handle.wait();
                Err(error)
            }
        }
    }

    /// Shows a new state: tooltip, icon and menu. Ignored once Quit is being carried out.
    pub fn set_state(&self, state: TrayState) {
        let _sent = self.controls.send(Control::SetState(state));
    }

    /// Shows a balloon tip on the icon.
    pub fn show_balloon(&self, balloon: Balloon) {
        let _sent = self.controls.send(Control::Balloon(balloon));
    }

    /// Registers what happens when the owner chooses Quit (or `tray.quit` appears). `quit.flag`
    /// is already written when it runs and the icon says "closing". The handler stops the server
    /// and then calls `shutdown`. It runs on the tray thread, so it must return quickly.
    pub fn on_quit(&self, handler: impl Fn() + Send + 'static) {
        self.handlers.set_quit(Box::new(handler));
    }

    /// Registers what happens for the other menu items and for a click on the icon: `Open`,
    /// `OpenNotifications`, `OpenLogs`, `Restart` and `Start`. It runs on the tray thread, so it
    /// must return quickly.
    pub fn on_command(&self, handler: impl Fn(Command) + Send + 'static) {
        self.handlers.set_command(Box::new(handler));
    }

    /// Removes the icon and ends the thread.
    #[must_use]
    pub fn shutdown(self) -> ExitReason {
        let _sent = self.controls.send(Control::Shutdown);
        self.wait()
    }

    /// Blocks until the tray ends: after `shutdown`, or when another process left `tray.stop`.
    #[must_use]
    pub fn wait(mut self) -> ExitReason {
        self.thread.take().map_or(ExitReason::Shutdown, |thread| {
            thread.join().unwrap_or(ExitReason::Crashed)
        })
    }
}

impl Drop for TrayHandle {
    fn drop(&mut self) {
        if self.thread.is_some() {
            let _sent = self.controls.send(Control::Shutdown);
        }
    }
}
