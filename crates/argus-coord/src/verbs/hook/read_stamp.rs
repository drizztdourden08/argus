//! The read stamp of a hook. A lead reads its board messages through the hooks, not through
//! `coord inbox`, so a hook that shows messages, the notice digest or a `WAKE` line tells Argus
//! the thread read them: the same `POST /api/p/:key/inbox/read` that `coord inbox` makes, so the
//! stall watch counts the wakes as read and the dependency requests are acknowledged. One call,
//! a short limit, every failure ignored: a hook is never held up by it.

use std::time::Duration;

use serde_json::Value;

use super::ctx::Hook;
use crate::verbs::work::inbox::ack_post::{Read, dep_refs, tell};

/// The longest a hook waits for the stamp.
const STAMP_TIMEOUT: Duration = Duration::from_millis(800);

/// The thread `thread` of `project` was shown `messages`: stamp its read.
pub fn stamp(hook: &Hook<'_>, project: &str, thread: &str, messages: &[Value]) {
    let read = Read::new(thread, thread, dep_refs(messages));
    tell(hook.env, &hook.paths, project, &read, Some(STAMP_TIMEOUT));
}

/// The thread `thread` of `project` was woken: stamp its read, with the dependency requests
/// that wait in its inbox.
pub fn stamp_inbox(hook: &Hook<'_>, project: &str, thread: &str) {
    let waiting = hook
        .board(project)
        .ok()
        .flatten()
        .and_then(|board| board["inbox"][thread].as_array().cloned())
        .unwrap_or_default();
    stamp(hook, project, thread, &waiting);
}
