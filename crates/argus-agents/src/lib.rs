//! Sub-agent liveness for Argus: the transcript watcher, the reaper and the reconciler.
//!
//! Three services read one data source, the sub-agent transcripts the session host keeps under
//! `~/.claude/projects/<slug>/<sessionId>/subagents/`, and the task outputs under the temp folder.
//!
//! | Service | Role |
//! |---|---|
//! | [`AgentWatcher`] | Tails transcripts, keeps live fields on the `agents` docs, never writes the board |
//! | [`AgentReaper`] | Closes a hook record whose transcript, task output and worktree stayed quiet for `agents.reapMs` |
//! | [`AgentReconciler`] | Compares the board with what Claude has: ids, links, duplicates, orphans |
//!
//! Everything the services read or write besides files goes through the small traits in
//! [`ports`], so the server wires the real board, wake engine and doc store, and tests use fakes.

pub mod board;
pub mod config;
pub mod error;
mod evidence;
pub mod fakes;
pub mod fs;
pub mod holds;
pub mod hook_logic;
pub mod iso;
pub mod model_rules;
pub mod ports;
pub mod reaper;
pub mod reconciler;
pub mod record;
pub mod schedule;
pub mod text;
pub mod tool_summary;
pub mod transcript;
pub mod watcher;

pub use argus_core::clock::{Clock, ManualClock, SystemClock};
pub use board::Board;
pub use config::{AgentsConfig, ReaperOptions, ReconcilerOptions, WatcherOptions};
pub use error::AgentsError;
pub use fs::{FileSystem, StdFs};
pub use ports::{ActivityLine, ActivitySink, BoardStore, LiveDocs, Ports, WakeEvent, WakeSink};
pub use reaper::AgentReaper;
pub use reconciler::{AgentReconciler, RunReport};
pub use schedule::{ServiceHandle, Tick, start};
pub use watcher::AgentWatcher;
