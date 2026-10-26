//! A notifier that shows nothing and remembers what it was asked, for tests of every caller.

use std::sync::Mutex;

use crate::error::NotifyError;
use crate::notifier::Notifier;
use crate::payload::Payload;
use crate::toast::Toast;

/// Builds the payload of each toast like the real notifier does, so an invalid toast fails the
/// same way, and keeps the payloads in the order they were shown.
#[derive(Debug, Default)]
pub struct RecordingNotifier {
    shown: Mutex<Vec<Payload>>,
    failure: Mutex<Option<NotifyError>>,
}

impl RecordingNotifier {
    /// A notifier that accepts every valid toast.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A notifier whose platform refuses every valid toast with `error`.
    #[must_use]
    pub fn failing(error: NotifyError) -> Self {
        Self {
            shown: Mutex::default(),
            failure: Mutex::new(Some(error)),
        }
    }

    /// The payloads shown so far, oldest first.
    ///
    /// # Panics
    ///
    /// When another thread panicked while it held the record.
    #[must_use]
    pub fn shown(&self) -> Vec<Payload> {
        self.shown.lock().expect("record lock").clone()
    }
}

impl Notifier for RecordingNotifier {
    fn show(&self, toast: &Toast) -> Result<(), NotifyError> {
        let payload = Payload::build(toast)?;
        if let Some(error) = self.failure.lock().expect("failure lock").clone() {
            return Err(error);
        }
        self.shown.lock().expect("record lock").push(payload);
        Ok(())
    }
}
