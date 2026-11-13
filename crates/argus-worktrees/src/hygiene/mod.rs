//! Worktree hygiene: the link between an agent and its worktree is mandatory (the owner's rule of 2026-11-06). A
//! worktree, never a main checkout, that nothing is attached to is removed, or its lead approves its removal, or the
//! Overlord deletes it or keeps it as an exception.
//!
//! ATTACHED (`judge`): a worktree is attached while any of these holds:
//! - a running sub-agent record (working, waiting, stop-requested) owns it: its folder is
//!   `.claude/worktrees/agent-<claudeId>`, or its `worktree` names it. An agent that ended after the last scan counts
//!   until the rescan it asks for
//! - a live thread works in it: an item in progress of a live thread names it (a feature thread's session is seen
//!   through its agent record or such an item; the board has no folder per thread)
//! - a launcher, a bundle or a run (check runs among them) is in it
//! - a live process uses it
//!
//! A list Argus cannot read (the runs, the processes) counts as attached. An open pull request does not: its branch
//! outlives the folder.
//!
//! NOT ATTACHED, NOTHING LEFT (`removing`): clean, its HEAD kept by a branch or a ref, not locked (Brock's own lock
//! aside). Once nothing has been attached for `detach_debounce_ms` (1 minute) Argus removes it through the very same
//! `Service::remove` and its guards (links first, Brock's lock, the branch only when merged), as one `detached` request
//! per owning lead (`Caller::Hygiene` with no approval: nothing may be lost). The lead gets one inbox note of what went.
//! A leftover folder with no file in it goes the same way, through the leftover removal.
//!
//! NOT ATTACHED, SOMETHING LEFT (`awaiting-lead`, `ask`): uncommitted changes, commits no ref keeps, a lock that is not
//! Brock's, or a removal that failed. The owning lead (the last agent's thread while it lives, else the thread of the
//! item it belongs to, else the thread of its rule, else the board's lead) is asked ONCE: an inbox note with the
//! approve command and the way to ask the Overlord for an exception, and a `worktree-approve` wake.
//! `coord worktree approve <id>` (`approve`) files an `approved` request: a full backup, then the removal with what is
//! left (`Caller::Hygiene` with the approval, the branch kept). A lock still stops it: the ask says to unlock first. A
//! backup that would be partial (untracked files or a nested repository over the copy limits) refuses it too.
//!
//! REPORTED (`report`): `report_after_ms` (30 minutes) after the ask, still unattached and unanswered, the worktree is
//! reported ONCE to the Overlord (the lead of the `portfolio` project): an inbox note on his board with the worktree,
//! what is left, the backup path and the one delete command (`coord worktree approve <id> --project <p>`), and a
//! `worktree-report` wake.
//!
//! EXCEPTION (`exception`): `coord worktree keep <id> --note "why"` is the Overlord's word only; a thread that calls it
//! is refused and told to ask with `coord lead portfolio "..."`. An exception is never removed and never reported. It
//! ends at its `until`, when the Overlord clears it, or when its worktree is gone. The keeps threads wrote on the board
//! before 2026-11-06 do not hold a worktree here (they still snooze the `worktree-stale` signal).
//!
//! Anything attached again (an agent, a run) stops all of this: the mark goes back to `attached`, and a later detach
//! starts over (a new debounce, a new ask).
//!
//! FLOW (`step`, right after every scan and on the server's live tick): mark every worktree, file the removals that
//! are due, ask the leads, report the asks left unanswered, then the `worktree-unlinked` notice (`notices`: a worktree
//! with no item an hour after Argus first saw it, told once, never removed for that). The request worker judges each
//! worktree again right before removing it (`still_free`): a detached one must still be clean and unattached, an
//! approved one still unattached and not an exception.
//!
//! STATE: each worktree's mark (state, owning lead, since, asked, reported, request), what Argus could not remove, the
//! exception list and the last removals, in the key-value table (`worktrees:hygiene:<project>`), so a restart keeps
//! every time.

mod approve;
mod ask;
mod auto_plan;
mod exception;
mod keep;
mod gone;
mod judge;
mod memory;
mod notices;
mod release;
mod report;
mod step;
mod text;
mod unlock;

pub use approve::{ApproveInput, Approved, NotApproved};
pub use auto_plan::PlanRow;
pub use exception::{KeepInput, Kept};
pub use memory::{HygieneDoc, HygieneMeta, HygieneState, RemovedDoc, hygiene_key};
pub use step::Stepped;

pub(crate) use memory::Memories;
