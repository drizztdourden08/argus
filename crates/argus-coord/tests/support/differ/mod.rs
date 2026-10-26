//! The differential harness: runs the same `coord` steps on the Node CLI and on `argus coord`,
//! each in its own temp home seeded alike, and compares the exit code, stdout, stderr and every
//! file under the homes after each step. A verb that answers 120 on the Argus side is reported
//! as "rust: not ported (120)" and is not a mismatch.
//!
//! ```text
//! let Some(engines) = Engines::or_skip("my_test") else { return };   // skips cleanly
//! let differ = Differ::standard(engines)?;                           // twin homes, fixture boards
//! let report = differ.run(&[
//!     Step::new(&["item", "list", "--project", "alpha"]).session(LEAD_SESSION),
//!     Step::new(&["inbox", "--project", "alpha", "--as", "worker"]),
//! ])?;
//! report.assert_no_mismatch();                                       // panics with the report
//! ```
//!
//! # API
//!
//! - [`Engines::locate`] and [`Engines::or_skip`]: node, `coord.mjs` (`COORD_NODE_CLI`) and the
//!   `argus` binary (`ARGUS_EXE`, else next to the test executables). Absent means skip.
//! - [`FixtureBoard`]: builds a schema 2 board (`thread`, `item`, `inbox_message`, `to_json`);
//!   [`fixture::standard_boards`] is the `alpha` and `beta` pair.
//! - [`Seed`]: `Fixture(boards)` or `CopyOf(dir)`, a copy of a board tree into the temp homes.
//! - [`Twin`]: two seeded homes (`node`, `rust`), each with `coord`, `dashboard` and `work`
//!   folders, and a closed dashboard port that both `config.json` files name.
//! - [`Step`]: argv, stdin, session id, extra env, cwd. Every call also gets `COORD_ENGINE=node`
//!   (Node side), `COORD_DASHBOARD_NO_AUTOSTART=1`, `DASHBOARD_PORT` and the temp homes.
//! - [`Differ`]: `new(engines, seed, stable_ids)` or `standard(engines)`, `run(steps)` giving a
//!   [`Report`] (`mismatches`, `not_ported`, `assert_no_mismatch`, `Display`).
//! - [`Normaliser`]: the rules (a table in its module); [`judge`]: the comparison.
//! - A step the Argus side answers 120 on runs on Node in the Argus home too, as the shim does,
//!   so the homes stay alike for the steps after it.

pub mod engines;
pub mod exec;
pub mod fixture;
pub mod homes;
pub mod normalise;
pub mod scenario;
pub mod snapshot;
pub mod step;
pub mod verdict;

pub use engines::Engines;
pub use fixture::FixtureBoard;
pub use homes::{Seed, Twin};
pub use normalise::Normaliser;
pub use scenario::{Differ, LEAD_SESSION, Report, WORKER_SESSION};
pub use step::Step;
pub use verdict::{Verdict, judge};
