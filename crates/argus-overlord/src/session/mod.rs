//! The session host's pure parts: the child command line, the child env, the state machine of
//! `state.json` and the billing guard. Nothing here spawns a process; the server adapter does,
//! through the [`ChildHost`](crate::ports::ChildHost) port.

pub mod args;
pub mod env;
pub mod guard;
pub mod state;
