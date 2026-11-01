//! Removal REQUESTS: a lead whose own session refuses `coord worktree remove` (irreversible local destruction) asks
//! Argus instead, and Argus's own server does the removing.
//!
//! A request is only a record. Deciding and deleting stay in `remove`: every target of a request goes through the very
//! same `Service::remove` the CLI route calls, with the same caller, so every guard, caller rule, backup rule, age limit
//! and junction rule applies unchanged. A request carries no extra power: it can never discard work unless the thread
//! names the owner's page entry.
//!
//! - modes: named targets, `finished` (every worktree merged, or stale and clean: the guards then decide each) and
//!   `leftovers` (every leftover folder old enough, each through `remove` on its own)
//! - caller: checked when the request is made (a stranger project or thread: forbidden; the page may not ask, it removes
//!   directly) and again by `remove` for every target
//! - worker: one per project, one target at a time, oldest request first; each request starts with a fresh scan. A
//!   long request of one project (88 leftovers of 10 minutes each) never holds up the requests of another; the
//!   removals themselves still run one at a time across every project (`Service::one_at_a_time`, the job queue)
//! - state: kept in the key-value table after every target (the last 100 per project); a restart makes the queued and
//!   the running ones queued again and goes on with the targets that have no outcome yet
//! - limit: a target that does not come back within `target_limit_ms` (30 minutes) fails with a sentence and the
//!   request goes on; while one runs, the record's `current` names it with the seconds it has been on it
//! - outcome: every target ends with a sentence; the request is `done` (nothing refused), `partial`, or `refused`
//! - follow-up: when a request finishes, ONE short note goes to the inbox of the requesting thread (from Argus) with a
//!   `thread-message` wake, and an activity line `worktree-request`

mod create;
mod deliver;
mod record;
mod resolve;
mod run;
mod store;
mod text;
mod watch;
mod worker;

use crate::queue::Queue;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

pub use create::{Queued, RequestInput};
pub use record::{
    Brief, Current, Mode, Outcome, RequestRec, RequestStatus, SkippedSummary, Target,
};
pub use store::requests_key;
pub use text::{queued_text, summary_text, what_text};

/// Finished requests kept per project.
pub const KEPT: usize = 100;
/// Named worktrees one request may carry.
pub const MAX_TARGETS: usize = 200;
/// Targets one `finished` or `leftovers` request takes.
pub const MAX_FINISHED: usize = 500;
/// The shortest `--idle` (one hour), in minutes.
pub const IDLE_MIN_MINUTES: f64 = 60.0;
/// The longest `--idle` (30 days), in minutes.
pub const IDLE_MAX_MINUTES: f64 = 43_200.0;
/// Requests of a project that may wait at once.
pub const MAX_ACTIVE: usize = 20;
/// How long a sentence of an outcome may be.
const SENTENCE_MAX: usize = 400;

pub(crate) struct State {
    pub stores: HashMap<String, store::Store>,
    /// The worker does not run until the service starts (and again after a stop).
    pub halted: bool,
    /// The projects whose worker runs.
    pub looping: HashSet<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            stores: HashMap::new(),
            halted: true,
            looping: HashSet::new(),
        }
    }
}

/// The requests of every project and the worker of each project that carries them out.
#[derive(Default)]
pub struct Requests {
    pub(crate) state: Mutex<State>,
    /// One worker per project, by project.
    pub(crate) queues: Mutex<HashMap<String, Queue>>,
}
