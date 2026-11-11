//! The sync of chats and sidebar folders between the Claude accounts: a [`job`] that scans,
//! plans and (on request) applies, with a [`backup`] made before any write, the [`steps`] it
//! reports, the [`compare`] rows the web shows, the [`drift`] counts of a dry run and the
//! [`review`] where the owner decides each difference.

pub mod backup;
pub mod compare;
pub mod counts;
pub mod drift;
pub mod job;
pub mod review;
pub mod steps;

pub use backup::{Manifest, UndoReport, undo};
pub use compare::{Compare, build_compare};
pub use counts::{ChatCounts, chat_counts};
pub use drift::{Drift, drift_of};
pub use job::{Env, Mode, Phase, Progress, Summary, lock_running, run, scan_and_plan};
pub use steps::{StepKind, StepSink, StepState, SyncStep};
