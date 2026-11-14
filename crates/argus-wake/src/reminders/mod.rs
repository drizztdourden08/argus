//! Open things the Overlord must resolve are not left to a single passive notice:
//!
//! * a wake fault wakes the Overlord at once (`fault-open`, raised by the registry);
//! * while any fault or any open notice (`worktree-stale`, `remote-sync-due`) stays open, the Overlord
//!   hears ONE coalesced line every `resend_ms` (15 minutes): `2 faults open, oldest 3h` (`open-resend`);
//! * a fault open for `owner_ms` (1 hour) also reaches the owner, once, as a toast and a notification.
//!
//! Nothing here reads a clock: the service passes `now`, so the manual clock of the tests drives it.
//! The owner of the service plugs in the notice sources and the owner sink.

mod config;
mod owner;
mod resend;
mod source;
mod text;

pub use config::ReminderConfig;
pub use owner::{
    HOSTED_DOWN, OwnerAlerts, OwnerNotice, OwnerSink, TOPIC_ASSIGNED, TOPIC_FAULT, TOPIC_RUN,
    TOPIC_STALL, TOPIC_VERIFY,
};
pub use resend::Resend;
pub use source::{NoticeSource, OpenNotice};
pub use text::open_line;
