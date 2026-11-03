//! The tray icon of Argus: what it shows, the files other processes use to talk to it, and on
//! Windows the icon itself on its own thread.

mod balloon;
mod downscale;
mod error;
mod fallback;
mod gate;
mod icon;
mod launch;
mod log;
mod menu;
#[cfg(windows)]
mod native;
mod paths;
mod quit_flag;
mod record;
mod state;
mod status;
mod stopping;
mod timestamp;
mod watch;

pub use balloon::{Balloon, write as write_balloon};
pub use error::TrayError;
pub use fallback::{Delivery, FallbackNotifier};
pub use icon::{ARGUS_ICO, IconKind, Ink, Rgba, render as render_icon};
pub use launch::{Launch, prepare as prepare_launch, wait_for_tray};
pub use menu::{Command, Entry, entries as menu_entries};
#[cfg(windows)]
pub use native::{ExitReason, TrayHandle, open_url, process_alive};
pub use paths::TrayFiles;
pub use quit_flag::{
    Boot, clear as clear_quit_flag, is_active as quit_flag_active, write as write_quit_flag,
};
pub use record::{TrayRecord, read_live as read_live_tray};
pub use state::{Phase, TrayState};
pub use status::{TrayStatus, icon_line, status as tray_status, status_text};
pub use stopping::{GRACE as STOP_GRACE, Stopped, request as request_stop, stop_tray};
