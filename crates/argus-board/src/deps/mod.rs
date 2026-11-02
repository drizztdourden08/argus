//! Cross-board dependencies. A request from an item of one project to an item of another is
//! one record in the ledger (collection `deps` of the portfolio board) and two mirrors: `waitsOn`
//! on the requester item and `requestedBy` on the assignee item. Only this module writes them.
//!
//! The ledger is the truth. Three files change for one dependency, each under its own lock and
//! never two locks at once, so a crash leaves a state that [`reconcile`] finishes or cancels.
//! Every call answers the [`Transition`]s it made; the server turns them into wakes.
//!
//! * [`ask`], [`close`], [`reopen`], [`cancel`]: the verbs.
//! * [`reconcile`], [`reconcile_board`], [`sweep`]: follow what the boards say.
//! * [`list`], [`show`], [`by_item`]: read.

mod ask;
mod cycle;
mod decide;
mod error;
mod ledger;
mod mirror;
mod model;
mod notice;
mod plan;
mod query;
mod reconcile;
mod resolve;
mod roster;
mod rules;
mod sight;

pub use ask::ask;
pub use error::{DepError, off};
pub use model::{Dep, DepState, Done, End, Row, Transition, UnknownState, project_of};
pub use plan::AskRequest;
pub use query::{ListFilter, Shown, View, by_item, lines, list, show};
pub use reconcile::{Sweep, reconcile, reconcile_board, sweep};
pub use resolve::{CancelRequest, CloseRequest, ReopenRequest, cancel, close, reopen};
pub use roster::{BoardRoster, Roster};
