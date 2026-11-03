//! Worktree hygiene: a worktree nobody works in goes by itself, with no human click.
//!
//! RELEASED: a worktree is released when every condition holds (`judge`), whatever its age:
//! - no running agent owns it: no sub-agent record that is working, waiting or stop-requested has it as its folder
//!   (`.claude/worktrees/agent-<claudeId>`) or in its `worktree`. No record at all, or only done, stopped or failed
//!   ones, is no running owner
//! - nothing holds it: no unexpired keep, no pull request of its branch, no item in progress of a live thread that
//!   names it, no launcher, bundle or run (check runs among them) in it, and no live process uses it
//! - safe: not a main checkout, inside its rule, not locked (Brock's own lock aside), clean, and its HEAD kept by a
//!   branch or a ref (or merged)
//!
//! A leftover folder with no file in it is released the same way and goes through the leftover removal.
//!
//! STATES (the Worktrees tab): in use (a running agent owns it, or something holds it), released (and when it goes),
//! review (dirty, commits no ref keeps, locked, a failed removal: the thread decides), kept. The 3 day "active" of the
//! staleness check is not a state.
//!
//! FLOW (`step`, right after every scan, so after the first scan at the start and after the scan an agent that ended
//! asks for, and on the server's live tick): a newly released worktree is told to its thread with ONE
//! `worktree-review` wake per thread, once nothing new was released for `release_debounce_ms` (a burst makes one
//! wake). `release_grace_ms` after the wake, the released worktrees of a thread go as one removal request of mode
//! `released`, by that thread: the request worker scans afresh, judges each one again right before removing it
//! (`still_released`), and removes it through the very same `Service::remove` and its guards (links first, backups,
//! Brock's lock, the branch only when merged). The thread gets one inbox note of what went. `coord worktree release
//! <id>` removes one at once through the same removal (`release`); `coord worktree keep` stops it. Anything else
//! (dirty, commits no ref keeps, locked, in use, a main checkout, outside a rule) is never removed automatically: it
//! shows `review` or `in use` with the reason and stays on the `worktree-stale` path.
//!
//! STATE: what each worktree is, when it was told, its request, what Argus could not remove, and the last removals,
//! in the key-value table (`worktrees:hygiene:<project>`), so a restart keeps the grace.

mod judge;
mod memory;
mod release;
mod step;
mod text;

pub use memory::{HygieneDoc, HygieneMeta, HygieneState, RemovedDoc, hygiene_key};
pub use step::Stepped;
pub use text::REASON;

pub(crate) use memory::Memories;
