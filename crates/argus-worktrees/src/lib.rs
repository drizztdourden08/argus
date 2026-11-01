//! The Worktrees service of Argus: the Worktrees tab's scan, statuses, stale signal and safe removal.
//!
//! The registered thread tells Argus where its worktrees live (rules on the board). This crate looks there with
//! read-only git, judges what it finds, tells the thread when a worktree needs a decision, and removes one behind
//! guards that never follow a junction out of the worktree and never discard work without a backup and the owner's say.
//! Git, the file system, the board, the clock and the process list come in through small traits, so the tests run on
//! temporary folders and fakes.

#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

mod approval;
mod backup;
mod batch;
mod board;
mod branch;
mod clock;
mod deleting;
mod derive;
mod doc;
mod error;
mod exec;
mod fact;
mod finish;
mod fs;
mod gh;
mod git;
mod guard;
mod guards;
mod holders;
mod hygiene;
mod inspect;
mod job_run;
mod jobs;
mod leftover_remove;
mod made;
mod measure;
mod modules;
mod parse;
mod paths;
mod ports;
mod probe;
mod probing;
mod progress;
mod prune;
mod queue;
mod remove;
mod request;
mod requests;
mod rule;
mod scan;
mod scanning;
mod service;
mod status;
mod sync;
mod tree;
mod turns;
mod waking;

#[cfg(test)]
mod specs;
#[cfg(test)]
mod testkit;

pub use approval::{approval_of, mentions, names_folder};
pub use argus_core::StorageError;
pub use backup::Backup;
pub use batch::{BatchInput, BatchOutcome, Skipped};
pub use board::{
    AgentFact, Answer, BoardFacts, Caller, EntryWorktree, Keep, Link, OwnerEntry, ThreadInfo,
};
pub use branch::BranchState;
pub use clock::{Clock, ManualClock, SystemClock, iso_from_ms};
pub use doc::{Derived, LeftoverDoc, Meta, RuleSummary, Status, WorktreeDoc};
pub use error::{ErrorCode, Result, WtError};
pub use exec::GitOutput;
pub use fact::Fact;
pub use fs::{Fs, RealFs};
pub use gh::{GhRunner, SystemGh};
pub use git::{Git, GitExec, REMOVE_TIMEOUT, SystemExec};
pub use guard::{assert_read_only, assert_write_ok};
pub use hygiene::{
    HygieneDoc, HygieneMeta, HygieneState, REASON as REVIEW_REASON, RemovedDoc, Stepped,
    hygiene_key,
};
pub use jobs::{JobDoc, JobState};
pub use paths::{clean_path, path_key, worktree_id};
pub use ports::{
    ActivityLine, LiveRun, Notifier, ProcInfo, ProcessLister, WakeEvent, WorktreeStore,
};
pub use progress::Step as RemovalStep;
pub use request::{Discarded, RemoveInput, RemoveOutcome};
pub use requests::{
    Brief, Current as RequestCurrent, Mode as RequestMode, Outcome as RequestOutcome, Queued,
    RequestInput, RequestRec, RequestStatus, Requests, SkippedSummary, Target as RequestTarget,
    queued_text, requests_key, summary_text, what_text,
};
pub use rule::{Rule, RuleKind};
pub use scanning::LiveResult;
pub use service::{Deps, Options, Service};
pub use status::wake_text;
pub use waking::{REASON, sent_key};
