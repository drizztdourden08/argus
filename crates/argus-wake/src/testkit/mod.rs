//! Fixtures for the tests of this crate and of the server slice: boards, a manual clock, listeners that
//! record what they receive, and the engine and the service on in-memory storage.

mod fixtures;
mod listen;
mod rig;

pub use fixtures::{
    DAY, HOUR, MIN, SEC, START, applied, board, failed_run, merge, presence_config,
    registry_boards, registry_config, thread, wake, wake_in,
};
pub use listen::Tap;
pub use rig::{EngineRig, Rig};
