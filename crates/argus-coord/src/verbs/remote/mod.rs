//! WP6, the server clients: the verbs that talk to the dashboard (`run`, `ping`, `activity`,
//! `proc`, `stall`, `wake-fault`, `worktree`, `lead`, `portfolio`, `dashboard`). Each is a module here
//! with its handler, and one row of [`PORTED`](crate::verbs::PORTED).

pub mod activity;
pub mod ci;
pub mod dashboard;
pub mod lead;
pub mod mode;
pub mod ping;
pub mod portfolio;
pub mod pr;
pub mod proc;
pub mod run;
pub mod stall;
mod support;
pub mod wakefault;
pub mod worktree;
