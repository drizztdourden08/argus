//! What the three verb groups share: how a verb reaches the board, the options it reads, the
//! ownership rules and the sort order of ids.

pub mod apply;
pub mod links;
pub mod opt;
pub mod scope;
pub mod serves;
pub mod show;

pub use scope::{Run, read, write};
