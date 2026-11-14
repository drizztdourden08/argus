//! The running supervisor of the hosted Overlord: its link, its state and its task, kept so a
//! stop can wait for the end and a start can follow it.

use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_overlord::session::state::State;
use argus_overlord::session::supervisor::{Handle, Link};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use super::live::StartFault;

struct Entry {
    link: Link,
    state: watch::Receiver<State>,
    task: Option<JoinHandle<()>>,
}

impl Entry {
    fn running(&self) -> bool {
        self.task.as_ref().is_some_and(|task| !task.is_finished())
    }
}

/// The supervisor of the child, if one was started.
#[derive(Default)]
pub(crate) struct Running {
    entry: Mutex<Option<Entry>>,
    /// Held by a control action from its start to its end, so two never overlap.
    pub(crate) control: tokio::sync::Mutex<()>,
}

impl Running {
    fn entry(&self) -> MutexGuard<'_, Option<Entry>> {
        self.entry.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The way in for messages; none before the first start.
    pub(crate) fn link(&self) -> Option<Link> {
        self.entry().as_ref().map(|e| e.link.clone())
    }

    /// The state of the session; the default (stopped) when none was started.
    pub(crate) fn state(&self) -> State {
        self.entry()
            .as_ref()
            .map(|e| e.state.borrow().clone())
            .unwrap_or_default()
    }

    /// Keeps the supervisor `begin` starts, unless one already runs.
    ///
    /// # Errors
    /// `StartFault::Running` when a supervisor task is still up.
    pub(crate) fn install(&self, begin: impl FnOnce() -> Handle) -> Result<(), StartFault> {
        let mut slot = self.entry();
        if slot.as_ref().is_some_and(Entry::running) {
            return Err(StartFault::Running);
        }
        let handle = begin();
        *slot = Some(Entry {
            link: handle.link,
            state: handle.state,
            task: Some(handle.task),
        });
        Ok(())
    }

    /// Asks the child to stop and hands out the task to wait on. None when nothing runs.
    pub(crate) fn begin_stop(&self) -> Option<JoinHandle<()>> {
        let mut slot = self.entry();
        let entry = slot.as_mut().filter(|e| e.running())?;
        entry.link.stop();
        entry.task.take()
    }

    /// Puts back a task that did not end in time.
    pub(crate) fn return_task(&self, task: JoinHandle<()>) {
        if let Some(entry) = self.entry().as_mut() {
            entry.task = Some(task);
        }
    }

    /// Forgets the ended supervisor.
    pub(crate) fn clear(&self) {
        *self.entry() = None;
    }
}
