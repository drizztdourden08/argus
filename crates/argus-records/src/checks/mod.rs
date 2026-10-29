//! The check runs of Brock's local CI.

mod store;
mod view;

pub use store::{CheckRun, CheckStore, IdBook};
pub use view::run_json;
