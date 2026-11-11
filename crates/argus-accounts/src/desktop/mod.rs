//! Claude Desktop as a program on this PC: [`app`] runs and closes it, [`vault`] keeps one saved
//! login per account and [`switch`] strings the steps of a switch together over the ports.

pub mod app;
pub mod journal;
pub mod switch;
pub mod vault;

pub use app::ClaudeDesktop;
pub use journal::{Entry, Journal, Repair, repair};
pub use switch::{Cancel, Done, Plan, Stages, Step, StepStatus, Stop, Timing, run};
pub use vault::{FsVault, last_known_account};
