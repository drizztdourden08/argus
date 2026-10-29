//! The tray icon on Windows: a thread with its own message loop, and the handle that controls it.

mod event_loop;
mod handle;
mod handlers;
mod opener;
mod session;
mod taskbar;
mod view;
mod win32;

pub use event_loop::ExitReason;
pub use handle::TrayHandle;
pub use opener::open_url;
pub use win32::process_alive;

use super::quit_flag::Boot;

impl Boot {
    /// The clock readings of this machine now.
    #[must_use]
    pub fn current() -> Self {
        Self {
            now_ms: super::timestamp::epoch_ms(std::time::SystemTime::now()),
            uptime_ms: win32::uptime_ms(),
        }
    }
}
