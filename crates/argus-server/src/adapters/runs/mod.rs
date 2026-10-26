//! The run service's adapters: its storage over the dashboard database, its events into the
//! doc store and the log hub, its process host, and the cards' view of it. Its warnings go to
//! the server log only.

mod attachments;
mod events;
mod host;
mod logs;
mod patch;
mod rows;
mod store;
mod view;

pub use events::DocRunEvents;
pub use host::RunHost;
pub use store::SqliteRuns;
pub use view::RunsCards;
