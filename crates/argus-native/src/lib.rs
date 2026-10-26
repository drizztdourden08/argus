//! Native helpers of Argus for the operating system: Windows toasts today, the tray icon next.
//!
//! Callers talk to the `Notifier` trait. `WindowsNotifier` is the real one (Windows only);
//! `RecordingNotifier` is the fake for tests.

mod error;
mod identity;
mod notifier;
mod payload;
mod recording;
mod text;
mod toast;
#[cfg(windows)]
mod windows_notifier;

pub use error::NotifyError;
pub use identity::{ARGUS_AUMID, ARGUS_NAME, Identity};
pub use notifier::Notifier;
pub use payload::{GROUP, Payload};
pub use recording::RecordingNotifier;
pub use text::{TEXT_MAX, TITLE_MAX, plain, xml_escape};
pub use toast::{Level, Toast};
#[cfg(windows)]
pub use windows_notifier::WindowsNotifier;
