//! Claude Desktop as a program on this PC: [`app`] runs and closes it, [`vault`] keeps one saved
//! login per account, [`process_app`] drives any program as the app (a test double) and
//! [`switch`] strings the steps of a switch together over the ports.

pub mod app;
pub mod journal;
pub mod process_app;
pub mod switch;
pub mod vault;

pub use app::ClaudeDesktop;
pub use journal::{Entry, Journal, Repair, repair};
pub use process_app::{Launch, Limits, ProcessApp};
pub use switch::{Cancel, Done, Plan, Stages, Step, StepStatus, Stop, Timing, WAITING_PHASE, run};
pub use vault::{FsVault, last_known_account};
