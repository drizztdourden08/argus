//! Native helpers of Argus for the operating system: Windows toasts and the tray icon.
//!
//! Callers talk to the `Notifier` trait. `WindowsNotifier` is the real one (Windows only);
//! `RecordingNotifier` is the fake for tests. The tray is `TrayHandle` (Windows only) over a model
//! (`TrayState`, `menu_entries`) and a set of protocol files (`TrayFiles`) that build everywhere.

mod error;
mod handles;
mod identity;
mod notifier;
mod payload;
mod recording;
mod text;
mod toast;
mod tray;
#[cfg(windows)]
mod windows_notifier;

pub use error::NotifyError;
pub use handles::keep_stdio_private;
pub use identity::{ARGUS_AUMID, ARGUS_NAME, Identity};
pub use notifier::Notifier;
pub use payload::{GROUP, Payload};
pub use recording::RecordingNotifier;
pub use text::{TEXT_MAX, TITLE_MAX, plain, xml_escape};
pub use toast::{Level, Toast};
pub use tray::{
    ARGUS_ICO, Balloon, Boot, Command, Delivery, Entry, FallbackNotifier, IconKind, Launch, Phase,
    Rgba, STOP_GRACE, Stopped, TrayError, TrayFiles, TrayRecord, TrayState, TrayStatus,
    clear_quit_flag, menu_entries, prepare_launch, quit_flag_active, read_live_tray, render_icon,
    request_stop, status_text, stop_tray, tray_status, wait_for_tray, write_balloon,
    write_quit_flag,
};
#[cfg(windows)]
pub use tray::{ExitReason, TrayHandle, process_alive};
#[cfg(windows)]
pub use windows_notifier::WindowsNotifier;
