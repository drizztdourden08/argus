//! The Argus wake engine: which owner actions wake which thread, the coalescing windows, the
//! wake-stream listeners, the registry of who can be woken, and the reminders for quiet agents and items.
//!
//! The core is pure logic over small traits ([`WakeStore`], [`DocSink`], [`FaultStore`], [`Clock`]) so the
//! server slice plugs SQLite in and the tests run on [`memory`] without I/O.

pub mod activity;
pub mod agent_wakes;
pub mod board;
pub mod config;
pub mod dep_wakes;
pub mod docs;
pub mod engine;
pub mod external;
pub mod faults;
pub mod item_wakes;
pub mod lenient;
pub mod memory;
pub mod mirror;
pub mod notices;
pub mod policy;
pub mod presence;
pub mod reasons;
pub mod registry;
pub mod reminders;
pub mod service;
pub mod source;
pub mod sse;
pub mod store;
pub mod sync_wakes;
pub mod testkit;
pub mod text;
pub mod time;
pub mod wake;
pub mod wake_policy;
pub mod wake_stream;

pub use argus_core::clock::{Clock, ManualClock, SystemClock};
pub use docs::{Doc, DocSink};
pub use engine::{EngineConfig, WakeEngine};
pub use faults::FaultStore;
pub use store::WakeStore;
