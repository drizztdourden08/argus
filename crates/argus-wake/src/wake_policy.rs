//! The wake policy: which reasons wake a thread (a model turn) and which only leave a notice on the
//! board, and how often one thread may be woken.
//!
//! A thread stays idle until Argus signals a reason on the board. [`WAKE_CLASSES`] is the one table:
//! the engine reads it where every wake is queued ([`crate::WakeEngine::submit`]), so every producer
//! (the policy, the registry faults, the item, agent and sync reminders, the worktree hygiene, the
//! reaper, the process monitor and the messages) obeys it. A passive reason is recorded and shown,
//! never sent to a listener. See `docs/wake-policy.md`.

use serde_json::Value;

use crate::config::ms_at_least;

/// What a reason does to its thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeClass {
    /// A model turn: the event goes to the thread's listener.
    Wake,
    /// A notice only: the board header counts it and the next prompt hook prints a digest line.
    Passive,
}

/// Every reason of [`crate::reasons::WAKE_REASONS`] and its class.
pub const WAKE_CLASSES: [(&str, WakeClass); 47] = [
    ("test-fail", WakeClass::Wake),
    ("test-partial", WakeClass::Wake),
    ("test-blocked", WakeClass::Wake),
    ("run-complete", WakeClass::Wake),
    ("owner-answer", WakeClass::Wake),
    ("owner-note", WakeClass::Wake),
    ("owner-decision", WakeClass::Wake),
    ("item-assigned", WakeClass::Wake),
    ("resume", WakeClass::Wake),
    ("portfolio-decision", WakeClass::Wake),
    ("portfolio-note", WakeClass::Wake),
    ("lead-message", WakeClass::Wake),
    ("lead-info", WakeClass::Passive),
    ("thread-message", WakeClass::Wake),
    ("wake-fault", WakeClass::Passive),
    ("fault-open", WakeClass::Wake),
    ("open-resend", WakeClass::Wake),
    ("agent-stale", WakeClass::Passive),
    ("agent-checkin", WakeClass::Passive),
    ("agent-unassigned", WakeClass::Passive),
    ("run-failed", WakeClass::Wake),
    ("agent-orphan", WakeClass::Wake),
    ("agent-idle", WakeClass::Wake),
    ("kanban-stale", WakeClass::Passive),
    ("process-alert", WakeClass::Passive),
    ("process-reaped", WakeClass::Wake),
    ("worktree-stale", WakeClass::Passive),
    ("worktree-review", WakeClass::Passive),
    ("worktree-unlinked", WakeClass::Passive),
    ("launcher-stale", WakeClass::Passive),
    ("launcher-master-missing", WakeClass::Passive),
    ("remote-sync-due", WakeClass::Passive),
    ("dep-request", WakeClass::Wake),
    ("dep-resolved", WakeClass::Wake),
    ("dep-blocked", WakeClass::Wake),
    ("dep-cancelled", WakeClass::Passive),
    ("dep-reopened", WakeClass::Passive),
    ("ack-nudge", WakeClass::Wake),
    ("stall", WakeClass::Wake),
    ("item-unassigned", WakeClass::Passive),
    ("todo-assigned", WakeClass::Wake),
    ("item-status", WakeClass::Passive),
    ("item-ask", WakeClass::Passive),
    ("item-unblocked", WakeClass::Wake),
    ("children-done", WakeClass::Wake),
    ("plan-run-done", WakeClass::Wake),
    ("todo-quiet", WakeClass::Passive),
];

/// The class of a reason. A reason the table does not list wakes, so nothing new is ever lost.
#[must_use]
pub fn class_of(reason: &str) -> WakeClass {
    WAKE_CLASSES
        .iter()
        .find(|(name, _)| *name == reason)
        .map_or(WakeClass::Wake, |(_, class)| *class)
}

/// Is this reason a notice only?
#[must_use]
pub fn is_passive(reason: &str) -> bool {
    class_of(reason) == WakeClass::Passive
}

/// At most one wake per thread in this window, when `config.json` does not say.
pub const DEFAULT_DEBOUNCE_MS: i64 = 5 * 60_000;

/// The digest of the wakes the debounce held goes out this long after the last wake.
pub const DEFAULT_DIGEST_MS: i64 = 20 * 60_000;

/// How often one thread may be woken (`wakePolicy` of `config.json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WakePolicy {
    /// At most one wake per thread in this window (`wakePolicy.debounceMs`); 0 turns it off.
    pub debounce_ms: i64,
    /// A wake inside the window joins one digest, sent this long after the last wake
    /// (`wakePolicy.digestMs`). An owner reason in the digest goes at the end of the window.
    pub digest_ms: i64,
}

impl Default for WakePolicy {
    fn default() -> Self {
        Self {
            debounce_ms: DEFAULT_DEBOUNCE_MS,
            digest_ms: DEFAULT_DIGEST_MS,
        }
    }
}

impl WakePolicy {
    /// No debounce: every wake goes when its window closes.
    #[must_use]
    pub fn off() -> Self {
        Self {
            debounce_ms: 0,
            digest_ms: 0,
        }
    }

    /// The `wakePolicy` section (anything missing or invalid keeps its default).
    #[must_use]
    pub fn from_json(section: &Value) -> Self {
        let ms = |key: &str, default: i64| ms_at_least(section.get(key), default, 0);
        Self {
            debounce_ms: ms("debounceMs", DEFAULT_DEBOUNCE_MS),
            digest_ms: ms("digestMs", DEFAULT_DIGEST_MS),
        }
    }

    /// When a wake held by the debounce goes out: an owner reason at the end of the window, any
    /// other one with the digest. `None` when the thread may be woken now.
    #[must_use]
    pub fn held_until(&self, last_wake: Option<i64>, now: i64, immediate: bool) -> Option<i64> {
        let last = last_wake.filter(|_| self.debounce_ms > 0)?;
        let free_at = last + self.debounce_ms;
        if now >= free_at {
            return None;
        }
        Some(if immediate {
            free_at
        } else {
            last + self.digest_ms.max(self.debounce_ms)
        })
    }
}
