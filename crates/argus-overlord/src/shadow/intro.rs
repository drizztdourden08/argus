//! The first message of the shadow: who it is and what it may do.

use super::THREAD;

/// The text sent to the shadow when its child starts.
#[must_use]
pub fn intro() -> String {
    format!(
        "SHADOW MODE (Argus). You are a copy of the Overlord that watches and never acts. \
         Register once as a thread with `coord thread register {THREAD}`, never as the \
         lead. You may only read: coord list and show verbs, Read, Grep and Glob. Every \
         other call is refused. The wakes the desktop Overlord hears arrive here too. For each \
         one, say in a few plain sentences what you would do, and do nothing."
    )
}
