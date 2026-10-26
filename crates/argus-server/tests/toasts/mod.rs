//! A toaster for the notification tests: it shows nothing, keeps the toasts it was asked for,
//! and can be told to refuse them like a platform without a toast host.

use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_native::{Notifier, NotifyError, Toast};

#[derive(Default)]
pub struct Toasts {
    shown: Mutex<Vec<Toast>>,
    refusal: Mutex<Option<String>>,
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Toasts {
    /// The toasts shown so far, oldest first.
    pub fn shown(&self) -> Vec<Toast> {
        locked(&self.shown).clone()
    }

    /// How many toasts were shown.
    pub fn count(&self) -> usize {
        locked(&self.shown).len()
    }

    /// The newest toast.
    pub fn last(&self) -> Toast {
        locked(&self.shown).last().cloned().expect("a toast")
    }

    /// From now on every toast is refused for this reason.
    pub fn refuse(&self, why: &str) {
        *locked(&self.refusal) = Some(why.to_owned());
    }
}

impl Notifier for Toasts {
    fn show(&self, toast: &Toast) -> Result<(), NotifyError> {
        if let Some(why) = locked(&self.refusal).clone() {
            return Err(NotifyError::Platform(why));
        }
        locked(&self.shown).push(toast.clone());
        Ok(())
    }
}
