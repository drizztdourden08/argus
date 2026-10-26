//! The seam between Argus and the operating system: `Notifier` shows a toast.

use crate::error::NotifyError;
use crate::toast::Toast;

/// Shows a toast to the owner. The real one talks to Windows; tests use `RecordingNotifier`.
pub trait Notifier: Send + Sync {
    /// Shows the toast and returns once the platform accepted it. The call blocks only for the
    /// platform call, so async callers run it on a blocking thread.
    ///
    /// # Errors
    ///
    /// The toast is invalid (`Empty`, `InvalidUrl`, `InvalidTag`) or the platform refused it
    /// (`Platform`).
    fn show(&self, toast: &Toast) -> Result<(), NotifyError>;
}
