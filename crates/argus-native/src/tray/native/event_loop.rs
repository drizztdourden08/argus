//! The tray thread: pumps Windows messages, takes the controls of the handle, and looks at the
//! request files twice a second.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

use super::handlers::Handlers;
use super::session::Session;
use super::win32;
use crate::tray::balloon::Balloon;
use crate::tray::error::TrayError;
use crate::tray::paths::TrayFiles;
use crate::tray::state::TrayState;

const PUMP_MS: u32 = 100;
const TICK: Duration = Duration::from_millis(500);

/// A message from the handle to the tray thread.
pub enum Control {
    SetState(TrayState),
    Balloon(Balloon),
    Shutdown,
}

/// Why the tray thread ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    /// The handle asked for it, or was dropped, or Windows ended the message loop.
    Shutdown,
    /// Another process left `tray.stop`.
    StopRequested,
    /// The tray thread panicked.
    Crashed,
}

/// Runs the tray until it ends. Tells `ready` once the icon is up, or why it could not be.
pub fn run(
    files: TrayFiles,
    handlers: Arc<Handlers>,
    controls: &Receiver<Control>,
    ready: &Sender<Result<(), TrayError>>,
) -> ExitReason {
    match Session::start(files, handlers) {
        Ok(mut session) => {
            let _told = ready.send(Ok(()));
            drive(&mut session, controls)
        }
        Err(error) => {
            let _told = ready.send(Err(error));
            ExitReason::Shutdown
        }
    }
}

fn drive(session: &mut Session, controls: &Receiver<Control>) -> ExitReason {
    let mut next_tick = Instant::now();
    loop {
        if !win32::pump(PUMP_MS) {
            return ExitReason::Shutdown;
        }
        session.drain_events();
        if let Some(reason) = take_controls(session, controls) {
            return reason;
        }
        if Instant::now() >= next_tick {
            next_tick = Instant::now() + TICK;
            if let Some(reason) = session.tick() {
                return reason;
            }
        }
    }
}

fn take_controls(session: &mut Session, controls: &Receiver<Control>) -> Option<ExitReason> {
    loop {
        match controls.try_recv() {
            Ok(Control::SetState(state)) => session.apply_state(state),
            Ok(Control::Balloon(balloon)) => session.balloon(&balloon),
            Ok(Control::Shutdown) | Err(TryRecvError::Disconnected) => {
                return Some(ExitReason::Shutdown);
            }
            Err(TryRecvError::Empty) => return None,
        }
    }
}
