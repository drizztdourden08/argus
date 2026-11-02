//! Fixtures for the tests of this crate and of the server slice: boards, a manual clock, listeners that
//! record what they receive, and the engine and the service on in-memory storage.

mod fixtures;
mod listen;
mod rig;

pub use crate::stalls::MemoryKv;
pub use fixtures::{
    DAY, HOUR, MIN, MemoryBusy, SEC, START, SentLookup, applied, board, failed_run, merge,
    never_sent, presence_config, registry_boards, registry_config, thread, wake, wake_in,
};
pub use listen::Tap;
pub use rig::{EngineRig, Rig};
