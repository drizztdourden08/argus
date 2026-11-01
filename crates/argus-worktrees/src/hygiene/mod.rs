//! Worktree hygiene: the worktree of a sub-agent that ended goes by itself, with no human click.
//!
//! RELEASED: a worktree is released when every condition holds (`judge`):
//! - not needed: every sub-agent record that owns it (its `worktree` names it, or it is the agent's own
//!   `.claude/worktrees/agent-<claudeId>`) is done or stopped, and the last scan looked after it ended
//! - nothing is linked: no unexpired keep, no open item, pull request, launcher, bundle or run (check runs among them)
//!   names its folder or branch, and no live process uses it
//! - safe: not a main checkout, inside its rule, not locked (Brock's own lock aside), clean, and its HEAD kept by a
//!   branch or a ref (or merged)
//!
//! FLOW (`step`, after every scan and on the server's live tick): a newly released worktree is told to its thread with
//! ONE `worktree-review` wake per thread, once nothing new was released for `release_debounce_ms` (a burst of agents
//! ending makes one wake). `release_grace_ms` after the wake, the released worktrees of a thread go as one removal
//! request of mode `released`, by that thread: the request worker scans afresh, judges each one again right before
//! removing it (`still_released`), and removes it through the very same `Service::remove` and its guards (links
//! first, backups, Brock's lock, the branch only when merged). The thread gets one inbox note of what went.
//! `coord worktree release <id>` removes one at once through the same removal (`release`); `coord worktree keep`
//! stops it. Anything else (dirty, commits no ref keeps, locked, in use, a main checkout, outside a rule) is never
//! removed automatically: it shows `review` with the reason and stays on the `worktree-stale` path.
//!
//! STATE: what each worktree is (released, review, kept), when it was told, its request, what Argus could not
//! remove, and the last removals, in the key-value table (`worktrees:hygiene:<project>`), so a restart keeps the grace.

mod judge;
mod memory;
mod release;
mod step;
mod text;

pub use memory::{HygieneDoc, HygieneMeta, HygieneState, RemovedDoc, hygiene_key};
pub use step::Stepped;
pub use text::REASON;

pub(crate) use memory::Memories;
