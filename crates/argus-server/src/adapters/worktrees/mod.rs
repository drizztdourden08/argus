//! The worktree service's adapters: its storage over the boards and the key-value table, its
//! notifier over the activity log and the wake bridge, and the machine's processes.

mod auto;
mod cwds;
mod facts;
mod links;
mod machine;
mod notifier;
mod store;

pub use auto::{AutoSetting, extras_of};
pub use machine::MachineProcs;
pub use notifier::TreeNotifier;
pub use store::BoardWorktrees;
