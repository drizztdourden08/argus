//! The balloon fallback of the notifications: when a toast fails and a tray holds the icon, the
//! tray shows the notification as a balloon tip instead.
// test-tag: critical

use super::balloon::{self, Balloon};
use super::paths::TrayFiles;
use super::record::read_live;
use crate::error::NotifyError;
use crate::notifier::Notifier;
use crate::toast::Toast;

/// How a notification reached the owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    /// The toast was shown.
    Toast,
    /// The toast failed and the tray was asked to show a balloon tip.
    Balloon {
        /// Why the toast failed.
        reason: String,
    },
}

type Alive = Box<dyn Fn(u32) -> bool + Send + Sync>;

/// A notifier that tries the toast first and the tray balloon second.
pub struct FallbackNotifier<N> {
    toast: N,
    files: TrayFiles,
    alive: Alive,
}

impl<N: Notifier> FallbackNotifier<N> {
    /// Wraps `toast`. `alive` says whether a process id is a running process.
    pub fn new(
        toast: N,
        files: TrayFiles,
        alive: impl Fn(u32) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            toast,
            files,
            alive: Box::new(alive),
        }
    }

    /// Shows the toast, or the balloon when the platform refused the toast and a tray runs.
    ///
    /// # Errors
    ///
    /// The toast is invalid, or it failed and no tray could take the balloon.
    pub fn show_reported(&self, toast: &Toast) -> Result<Delivery, NotifyError> {
        let error = match self.toast.show(toast) {
            Ok(()) => return Ok(Delivery::Toast),
            Err(error @ NotifyError::Platform(_)) => error,
            Err(other) => return Err(other),
        };
        if read_live(&self.files, &self.alive).is_none() {
            return Err(error);
        }
        let balloon = Balloon::new(&toast.title, &toast.text);
        match balloon::write(&self.files, &balloon) {
            Ok(()) => Ok(Delivery::Balloon {
                reason: error.to_string(),
            }),
            Err(_) => Err(error),
        }
    }
}

impl<N: Notifier> Notifier for FallbackNotifier<N> {
    fn show(&self, toast: &Toast) -> Result<(), NotifyError> {
        self.show_reported(toast).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::{Delivery, FallbackNotifier};
    use crate::error::NotifyError;
    use crate::notifier::Notifier;
    use crate::recording::RecordingNotifier;
    use crate::toast::Toast;
    use crate::tray::balloon;
    use crate::tray::paths::TrayFiles;
    use crate::tray::record;

    fn failing() -> RecordingNotifier {
        RecordingNotifier::failing(NotifyError::Platform("no toast host".into()))
    }

    #[test]
    fn a_shown_toast_leaves_no_balloon() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        let notifier = FallbackNotifier::new(RecordingNotifier::new(), files.clone(), |_| true);
        let delivery = notifier.show_reported(&Toast::new("Hi", "there")).unwrap();
        assert_eq!(delivery, Delivery::Toast);
        assert!(!files.balloon_file().exists());
    }

    #[test]
    fn a_failed_toast_becomes_a_balloon_when_a_tray_runs() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        record::write(&files, 50).unwrap();
        let notifier = FallbackNotifier::new(failing(), files.clone(), |pid| pid == 50);
        let delivery = notifier
            .show_reported(&Toast::new("Build", "failed on main"))
            .unwrap();
        let Delivery::Balloon { reason } = delivery else {
            panic!("expected a balloon");
        };
        assert!(reason.contains("no toast host"));
        let pending = balloon::take(&files).unwrap();
        assert_eq!(
            (pending.title.as_str(), pending.text.as_str()),
            ("Build", "failed on main")
        );
    }

    #[test]
    fn without_a_tray_the_failure_stands() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        let notifier = FallbackNotifier::new(failing(), files.clone(), |_| false);
        let error = notifier.show(&Toast::new("Build", "x")).unwrap_err();
        assert!(matches!(error, NotifyError::Platform(_)));
        assert!(!files.balloon_file().exists());
    }

    #[test]
    fn an_invalid_toast_is_not_retried_as_a_balloon() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        record::write(&files, 50).unwrap();
        let notifier = FallbackNotifier::new(RecordingNotifier::new(), files.clone(), |_| true);
        assert_eq!(notifier.show(&Toast::new("", "")), Err(NotifyError::Empty));
        assert!(!files.balloon_file().exists());
    }
}
