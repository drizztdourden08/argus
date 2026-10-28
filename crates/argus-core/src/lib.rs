//! Types and helpers every Argus crate shares: collections and doc changes, ids and scopes,
//! the clock, JavaScript-compatible JSON helpers, API errors, paths, config, the event bus
//! and the server log.
#![deny(clippy::cognitive_complexity)]

pub mod bus;
pub mod change;
pub mod clock;
pub mod coll;
pub mod collate;
pub mod config;
pub mod error;
#[cfg(feature = "testkit")]
pub mod fixture;
pub mod fs;
pub mod ids;
pub mod js;
pub mod json_text;
pub mod launch;
pub mod log;
pub mod names;
pub mod paths;
pub mod redact;
pub mod shadow_mark;
pub mod storage;
pub mod unique;

pub use bus::{Bus, Subscription};
pub use change::{AppliedChange, DocChange, DocOp, DocsEvent, ResetEvent};
pub use coll::Coll;
pub use config::{Config, ConfigError};
pub use error::{ApiError, ErrorCode};
pub use paths::Paths;
pub use storage::StorageError;
