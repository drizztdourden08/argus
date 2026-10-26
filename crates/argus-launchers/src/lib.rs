//! Launchers and runs: the processes a lead defined, started, watched, logged and stopped
//! (DESIGN.md sections 5 and 13.7), and the launcher and bundle rules around them (13.1, 13.7).
//!
//! The run service never touches a database or a real process directly. Rows and logs go
//! through the [`Storage`] traits, pages are told through [`RunEvents`], and processes come
//! from a [`ProcessHost`] ([`SystemHost`] for real, [`FakeHost`] for tests).

#![warn(clippy::cognitive_complexity)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

mod agents;
mod ansi;
mod attach;
mod clock;
mod def;
mod defs_source;
mod error;
mod events;
mod fake_host;
mod host;
pub mod launcher;
mod live_run;
mod log_splitter;
mod log_window;
mod log_writer;
mod memory;
mod names;
mod record;
mod redact;
mod run_attach;
mod run_finalize;
mod run_input;
mod run_launch;
mod run_query;
mod run_recover;
mod run_retention;
mod run_service;
mod run_shutdown;
mod run_start;
mod run_stop;
mod run_supervise;
mod run_verdict;
mod store;
mod system_host;
mod time;
mod tree_kill;
mod utf8;

pub use agents::{AgentFilter, agents_with_runs};
pub use ansi::plain_text;
pub use attach::{AttachmentRow, SaveRequest, save_attachment, sniff_image};
pub use clock::{Clock, ManualClock, SystemClock};
pub use def::{RunDef, RunKind, coerce_def, defs_from_map, public_def};
pub use defs_source::{BoardFileDefs, DefsSource, MemoryDefs};
pub use error::RunError;
pub use events::{NullEvents, RecordingEvents, RunEvents};
pub use fake_host::FakeHost;
pub use host::{
    BoxRead, BoxWrite, ExitInfo, ProcessHost, SpawnOptions, SpawnRequest, SpawnedProcess,
    spawn_options,
};
pub use log_splitter::{LineSplitter, MAX_LINE};
pub use log_window::{LogWindow, PendingLine, Plan};
pub use log_writer::{LogContext, LogWriter};
pub use memory::MemoryStore;
pub use names::{check_agent, check_key, check_name, check_rid};
pub use record::{
    ListFilter, LogCommit, LogEvent, LogLine, LogRead, NewRun, Origin, ReadMode, RunLimits,
    RunPatch, RunRow, RunStatus, StoredLine, Stream,
};
pub use redact::{Redactor, secret_env_values};
pub use run_attach::AttachmentInfo;
pub use run_input::STDIN_MAX;
pub use run_query::{LogOptions, LogPage};
pub use run_service::{RunConfig, RunDeps, RunService};
pub use run_start::{StartOptions, StartOutcome};
pub use run_verdict::{EndInput, Verdict, verdict};
pub use store::{AttachmentStore, LogStore, RunStore, Storage};
pub use system_host::SystemHost;
pub use time::{DAY_MS, format_iso, parse_iso, system_now_ms};
