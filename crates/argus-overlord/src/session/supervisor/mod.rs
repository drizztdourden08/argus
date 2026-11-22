//! The supervisor of the hosted child: it starts one long-lived `claude` through the
//! [`ChildHost`] port, reads its stdout, writes the messages it is given to its stdin and starts it
//! again after a crash.
//!
//! Messages arrive on a channel, never by polling. While the child is idle the first message is
//! written at once. While a turn runs they are held, and after the `result` they go as one
//! combined user message. A dead child keeps what is held for the next start.
//!
//! The `init` event arrives only after the first user message (probe of 2026-11-14), so the billing
//! guard runs on the first turn, not at spawn.

mod child;
#[cfg(test)]
mod fake;
mod flight;
mod restart;
mod run;
#[cfg(test)]
mod specs;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use self::flight::Flight;
use super::guard::GuardFault;
use super::inbox::{Held, HeldView, Inbox};
use super::state::State;
use crate::ports::{ChildHost, ChildSpec, HostFault};

/// What the supervisor is told.
pub enum Command {
    /// A message for the child.
    Message(String),
    /// End the child and the supervisor.
    Stop,
}

/// What the server hears about the child's life.
pub trait Observer: Send + Sync {
    /// A child process exists: wakes may flow to it through `link`.
    fn started(&self, link: &Link);
    /// The child is gone: wakes must stay pending.
    fn stopped(&self);
    /// The first turn failed the billing guard; the child was ended and is not restarted.
    fn guard_failed(&self, fault: &GuardFault);
    /// One raw stdout line of the child, before it is read. Most observers ignore it.
    fn line(&self, _line: &str) {}
}

/// Builds the spec of the next child from the session id to resume, read fresh at every start so a
/// changed key or budget takes effect.
pub type SpecFactory = Box<dyn Fn(Option<&str>) -> Result<ChildSpec, HostFault> + Send + Sync>;

/// How a supervisor runs.
pub struct Config {
    pub spec: SpecFactory,
    /// Crashes in a row after which the session stays crashed.
    pub restart_max: u32,
    /// The session to resume at the first start.
    pub resume_from: Option<String>,
}

/// The way in for messages: cheap to clone, and honest about whether a child is up.
#[derive(Clone)]
pub struct Link {
    commands: UnboundedSender<Command>,
    alive: Arc<AtomicBool>,
    flight: Flight,
    held: HeldView,
}

impl Link {
    /// Gives the child a message. False when no child is up (or the supervisor ended), so the
    /// caller keeps the message.
    #[must_use]
    pub fn send(&self, text: String) -> bool {
        self.alive.load(Ordering::SeqCst) && self.commands.send(Command::Message(text)).is_ok()
    }

    /// Is a tool call of the child waiting for its result?
    #[must_use]
    pub fn tool_in_flight(&self) -> bool {
        self.flight.in_flight()
    }

    /// The messages waiting for the child, oldest first.
    #[must_use]
    pub fn held(&self) -> Vec<Held> {
        self.held.snapshot()
    }

    /// Asks the supervisor to end the child by pid and finish.
    pub fn stop(&self) {
        let _ = self.commands.send(Command::Stop);
    }
}

/// A running supervisor.
pub struct Handle {
    pub link: Link,
    /// The state of `state.json`, as it changes.
    pub state: watch::Receiver<State>,
    pub task: JoinHandle<()>,
}

/// Starts the supervisor on the current tokio runtime.
#[must_use]
pub fn start<H: ChildHost>(host: Arc<H>, config: Config, observer: Arc<dyn Observer>) -> Handle {
    let (commands, rx) = unbounded_channel();
    let (state_tx, state) = watch::channel(State::default());
    let alive = Arc::new(AtomicBool::new(false));
    let (inbox, flight) = (Inbox::default(), Flight::default());
    let link = Link {
        commands,
        alive: Arc::clone(&alive),
        flight: flight.clone(),
        held: inbox.view(),
    };
    let wiring = child::Wiring {
        rx,
        config,
        observer,
        alive,
        link: link.clone(),
    };
    let task = tokio::spawn(restart::drive(run::Run::new(host, state_tx, (inbox, flight)), wiring));
    Handle { link, state, task }
}
