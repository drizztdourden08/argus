//! Types and helpers every Argus crate shares: collections and doc changes, ids and scopes,
//! the clock, JavaScript-compatible JSON helpers, API errors, paths, config, the event bus
//! and the server log.
#![deny(clippy::cognitive_complexity)]

pub mod boards;
pub mod build_link;
pub mod bus;
pub mod change;
pub mod clock;
pub mod coll;
pub mod collate;
pub mod config;
pub mod error;
#[cfg(any(test, feature = "testkit"))]
pub mod fixture;
pub mod fs;
mod home_mark;
pub mod ids;
pub mod js;
pub mod json_text;
pub mod launch;
pub mod lead;
pub mod lockfile;
pub mod log;
pub mod names;
pub mod op_kinds;
pub mod page_url;
pub mod paths;
pub mod preview_mark;
pub mod process;
pub mod redact;
pub mod remote;
pub mod remote_state;
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
