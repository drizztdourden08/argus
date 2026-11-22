//! The session host: the child command line, the child env, the state machine of `state.json`, the
//! billing guard, the inbox of held messages and the supervisor that runs one long-lived child. The
//! supervisor spawns nothing itself; the server adapter does, through the
//! [`ChildHost`](crate::ports::ChildHost) port.

pub mod activity;
pub mod args;
pub mod env;
pub mod guard;
pub mod inbox;
pub mod owner;
pub mod state;
pub mod supervisor;
