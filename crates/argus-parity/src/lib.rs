//! `argus parity`: a read-only comparison of two dashboard servers by value, the check of the
//! switch-over. It reads every route both servers serve for every scope (the db shim's
//! collections, health, projects, activity, processes, notifications, runs, wakes, presence,
//! agent reconcile, the MCP `tools/list`) and one doc stream, sets the volatile fields aside
//! (see [`kind`]), and tells per route whether it is equal, differs or is missing on one side.
//!
//! It never writes: a probe is a GET or one of the two MCP reads (see [`probe`]), and the only
//! file it writes is the report named by `--out`.
#![deny(clippy::cognitive_complexity)]

mod cli;
mod diff;
mod findings;
mod got;
mod kind;
mod normalize;
mod pattern;
mod probe;
mod probes;
mod report;
mod rules;
mod run;
mod side;
mod stream;
mod verdict;

pub use cli::{Args, run};
pub use diff::Change;
pub use got::Got;
pub use kind::Kind;
pub use report::{Counts, Report, RouteResult};
pub use run::{Options, compare};
pub use verdict::{Verdict, Which};
