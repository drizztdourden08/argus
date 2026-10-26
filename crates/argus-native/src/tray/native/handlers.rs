//! The two callbacks the owner of the tray registers. They run on the tray thread, so they
//! return quickly and hand slow work to another thread.

use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::tray::menu::Command;

type QuitFn = Box<dyn Fn() + Send>;
type CommandFn = Box<dyn Fn(Command) + Send>;

#[derive(Default)]
pub struct Handlers {
    quit: Mutex<Option<QuitFn>>,
    command: Mutex<Option<CommandFn>>,
}

fn locked<T>(slot: &Mutex<T>) -> MutexGuard<'_, T> {
    slot.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Handlers {
    pub fn set_quit(&self, handler: QuitFn) {
        *locked(&self.quit) = Some(handler);
    }

    pub fn set_command(&self, handler: CommandFn) {
        *locked(&self.command) = Some(handler);
    }

    pub fn fire_quit(&self) {
        if let Some(handler) = locked(&self.quit).as_ref() {
            handler();
        }
    }

    pub fn fire_command(&self, command: Command) {
        if let Some(handler) = locked(&self.command).as_ref() {
            handler(command);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::Handlers;
    use crate::tray::menu::Command;

    #[test]
    fn handlers_fire_once_registered_and_the_last_registration_wins() {
        let handlers = Handlers::default();
        handlers.fire_quit();
        handlers.fire_command(Command::Open);
        let quits = Arc::new(AtomicU32::new(0));
        let seen = Arc::clone(&quits);
        handlers.set_quit(Box::new(move || {
            seen.fetch_add(1, Ordering::SeqCst);
        }));
        let opened = Arc::new(AtomicU32::new(0));
        let seen = Arc::clone(&opened);
        handlers.set_command(Box::new(move |c| {
            seen.fetch_add(u32::from(c == Command::Open), Ordering::SeqCst);
        }));
        handlers.fire_quit();
        handlers.fire_command(Command::Open);
        handlers.fire_command(Command::Restart);
        assert_eq!(quits.load(Ordering::SeqCst), 1);
        assert_eq!(opened.load(Ordering::SeqCst), 1);
    }
}
