//! `argus migrate board-ids`: turns every board key into an opaque id and keeps the old keys as
//! aliases. The steps run in a fixed order under a marker and the board locks, after a checked
//! backup, and every step can be run again. `--rollback` reverses the run from its journal and
//! keeps what was written since; `--restore` puts the backup back.
#![deny(clippy::cognitive_complexity)]

mod backup;
mod error;
mod fault;
mod files;
mod journal;
mod jsonio;
mod locks;
mod map;
mod marker;
mod options;
mod plan;
mod precheck;
mod relocate;
mod remaining;
mod restore;
mod rewrite;
mod rollback;
mod rollcheck;
mod run;
mod sql;
mod steps;
mod table;
mod verify;

pub use error::Error;
pub use options::{Fault, Mode, Options};
pub use run::run;
