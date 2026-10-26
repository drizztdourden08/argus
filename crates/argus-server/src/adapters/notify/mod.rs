//! The notifications' ports over the dashboard: the table and its docs, the live config, the
//! project names, the page events, the tray balloon, and the board facts the automatic pings
//! read. The activity sink is in `activity`.

mod balloon;
mod config;
mod events;
mod facts;
mod names;
mod store;

pub use balloon::TrayBalloon;
pub use config::LiveConfig;
pub use events::DocStream;
pub use facts::facts_of;
pub use names::BoardNames;
pub use store::NotifyDocs;
