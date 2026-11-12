//! The `--agent A-n` label of a lock claim. Sub-agents of one thread share its identity, so the
//! label tells two of them apart: the holder stays the thread, and queue entries read `T#A-n`.

use serde_json::Value;

use super::holder_of;
use crate::verbs::work::run::Run;

/// The agent label the caller passed, `None` when absent or blank.
pub(super) fn label(r: &Run<'_>) -> Option<String> {
    r.text_opt("agent")
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .map(str::to_owned)
}

/// A queue entry: the thread id, then `#` and the agent when there is one.
pub(super) fn entry_of(thread: &str, agent: Option<&str>) -> String {
    agent.map_or_else(|| thread.to_owned(), |a| format!("{thread}#{a}"))
}

/// The thread of a queue entry.
pub(super) fn thread_of(entry: &str) -> &str {
    entry.split('#').next().unwrap_or(entry)
}

/// The agent of a queue entry, `None` for a plain thread id.
pub(super) fn agent_of(entry: &str) -> Option<&str> {
    entry.split_once('#').map(|(_, agent)| agent)
}

/// The agent label of the holder, `None` when the thread holds it itself.
pub(super) fn holder_agent(lock: &Value) -> Option<String> {
    lock["holderAgent"].as_str().map(str::to_owned)
}

/// The holder as shown: `T#A-n` when it has an agent, `None` when the lock is free.
pub(super) fn holder_text(lock: &Value) -> Option<String> {
    holder_of(lock).map(|h| entry_of(&h, holder_agent(lock).as_deref()))
}
