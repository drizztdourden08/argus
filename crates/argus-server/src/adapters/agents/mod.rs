//! The sub-agent services' ports: the boards they read and write, the `agents` docs and the
//! wake engine. The activity sink is in `activity`.

pub mod docs;
pub mod store;

pub use docs::{AgentDocs, AgentWakes};
pub use store::AgentBoards;
