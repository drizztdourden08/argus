//! The write-time rules of the Kanban config: when an item is written, the column it lands in
//! checks it. The kinds are `require-parent`, `require-assignee`, `require-field` and `limit`,
//! and the column's own limit. A rule that breaks either refuses the write or only warns.
//!
//! * `column`: which column an item sits in, and who is in it.
//! * `parent`, `assignee`, `field`, `limit`: one kind each, answering the sentence of the break
//!   or nothing.
//! * `evaluate`: the rules of a column run over the items a write touched.

mod assignee;
mod column;
mod evaluate;
mod field;
mod limit;
mod parent;

pub use evaluate::{Verdict, evaluate, has_write_rules};
