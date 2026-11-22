//! The timed rules of the hosted Overlord (I-190): prompts the owner or the Overlord adds, each
//! delivered to the child every N minutes. All pure; the rows live in the server's database and the
//! tick that delivers them is in the server crate.

mod model;
mod parse;
mod schedule;
mod seed;
#[cfg(test)]
mod tests;
mod validate;

pub use model::{By, Rule, RuleInput, RulePatch};
pub use parse::{parse_input, parse_patch};
pub use schedule::{after_run, due, next_due_at, prompt_text};
pub use seed::seed;
pub use validate::{create, patch};

/// The shortest title, in characters.
pub const TITLE_MIN: usize = 1;
/// The longest title, in characters.
pub const TITLE_MAX: usize = 80;
/// The longest prompt, in characters.
pub const PROMPT_MAX: usize = 4000;
/// The shortest period, in minutes.
pub const EVERY_MIN: u32 = 5;
/// The longest period, in minutes: one day.
pub const EVERY_MAX: u32 = 1440;
