//! The board items of the portfolio: one stored record per project on the `boards` collection
//! of `_portfolio`, shown with the code `BD`. It holds what the owner says about a board (goal
//! and note) and its lifecycle. Every live fact stays on the project card.
//!
//! This module is the one writer: [`reconcile`] creates, archives and unarchives items to match
//! the project list, and [`apply_owner_op`] applies what the owner posts from the Argus page.

mod owner_op;
mod reconcile;

pub use owner_op::apply_owner_op;
pub(super) use owner_op::import;
pub use reconcile::{needs_reconcile, reconcile};

/// The collection of board items on the portfolio board.
const COLL: &str = "boards";
