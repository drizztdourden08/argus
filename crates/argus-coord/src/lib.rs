//! Argus coord: the `coord` CLI (verbs and hooks) in Rust, served as `argus coord <verb>` and
//! called by the `coord` shim. The specification is the Node CLI; every stdout and stderr line,
//! every refusal, every exit code and every byte of `board.json` is the Node one. The board
//! semantics exist once, in `argus-board`; this crate adds only the command line layer.
//!
//! [`run`] is the in-process entry: it takes an [`Invocation`] (arguments, environment, working
//! directory, streams, clock) and returns the exit code. A verb group that is not ported yet
//! returns exit 120 before it reads stdin or touches any file, so the shim runs the Node verb.
//!
//! # Layout
//!
//! The foundation (frozen once it lands):
//!
//! ```text
//! src/cli/       args (parseArgs), route (the verb table and the router), usage, out (say,
//!                table, ago, clip), exit (error to exit code), env, invocation, capture
//! src/who/       project lookup, the caller of a verb, the session registry scan
//! src/client/    dashboard config, blocking HTTP client, health, autostart, SSE reader
//! src/shortcut/  alias urls and launch.json merge (`coord board shortcut`)
//! src/verbs/     the table of ported verb groups (`PORTED`)
//! src/verbs/shared/ what every group uses and none rebuilds: `caller` (`who_of`,
//!                `is_coordinator`) and `lock` (`board_refusal`, the Node lock sentence)
//! ```
//!
//! The verb groups, one folder each so parallel work packages never edit the same file. A
//! group's folder is added with its first module, and its row joins `verbs::PORTED`:
//!
//! ```text
//! src/verbs/hook/     WP2: the ten hook events, visit and adopt, the context texts, guard,
//!                     the sub-agent bindings, the listener, the autostart
//! src/verbs/thread/   WP3: thread, dispatch, board, purge, restore, trash, check, tick, sync
//! src/verbs/work/     WP4: item, test, entry and its aliases, action, lock, log, inbox
//! src/verbs/agent/    WP5: agent, build, launcher
//! src/verbs/remote/   WP6: run, ping, activity, proc, wake-fault, worktree, lead, portfolio,
//!                     dashboard
//! tests/              each package ports its Node cases here, calling `run` with a temp
//!                     `COORD_HOME` and captured streams; WP7 spawns the binary
//! ```

#![deny(clippy::cognitive_complexity)]

pub mod cli;
pub mod client;
pub mod shortcut;
pub mod verbs;
pub mod who;

use std::io::Write;

pub use cli::{Capture, CoordError, Env, Invocation, NOT_PORTED, Outcome};

/// Runs one call of `coord` and returns the exit code of the process.
pub fn run(inv: &mut Invocation) -> i32 {
    cli::dispatch(inv, verbs::PORTED)
}

/// Runs `coord` for this process with its arguments (the words after `coord`): the entry of the
/// `argus coord` fast path. Nothing but the standard library is started.
#[must_use]
pub fn run_process(argv: Vec<String>) -> i32 {
    let mut inv = Invocation::from_process(argv);
    let code = run(&mut inv);
    inv.stdout.flush().ok();
    inv.stderr.flush().ok();
    code
}
