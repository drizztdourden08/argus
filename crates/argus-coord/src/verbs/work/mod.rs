//! WP4, the work documents and messages: `item`, `test`, `entry` (with `ask`, `decide`, `notify`,
//! `message` and `answer`), `action` (alias `todo`), `lock`, `log` and `inbox`. Each module
//! prints inside its locked board write, before the save, as the Node verbs do.

mod action;
mod crud;
mod entry;
mod group;
pub(crate) mod inbox;
mod item;
mod keys;
mod listing;
mod lock;
mod log;
mod place;
mod run;
mod soft;
mod test;
mod types;

pub use action::handle as action;
pub use entry::handle as entry;
pub use inbox::handle as inbox;
pub use item::handle as item;
pub use lock::handle as lock;
pub use log::handle as log;
pub use test::handle as test;
pub use types::handle as types;
