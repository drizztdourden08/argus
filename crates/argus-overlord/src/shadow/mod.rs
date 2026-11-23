//! Shadow mode of the hosted Overlord (I-172): a child that forks the desktop Overlord's session,
//! hears a copy of every wake the desktop Overlord hears, may only read, and writes what it would
//! do. All pure; the server wires it to the child, the wake engine and the permission endpoint.

mod allow;
mod argv;
mod intro;
mod log;
#[cfg(test)]
mod tests;

pub(crate) use allow::SHELL;
pub use allow::{DENIED, Verdict, coord_words_allowed, verdict};
pub use argv::{shadow_args, shadow_argv};
pub use intro::intro;
pub use log::{Seen, Turn, denial_entry, seen};

/// The thread the shadow registers as, with the role `thread`, so `overlord_only` refuses it.
pub const THREAD: &str = "coordinator-hosted";

/// The thread of the desktop Overlord, whose wakes the shadow mirrors.
pub const OVERLORD_THREAD: &str = "coordinator";

/// The mirror rule: a wake is copied to the shadow when the desktop Overlord got it, and only then.
#[must_use]
pub fn mirrors(thread: &str) -> bool {
    thread == OVERLORD_THREAD
}
