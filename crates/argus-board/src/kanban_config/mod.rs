//! The Kanban config of a board: the columns the Kanban tab shows and the rules that watch them.
//! `board.json` holds it as the top-level `kanban` object; a board with none runs the default,
//! which is the four columns and the quiet hours of before. Here the config is only stored and
//! checked; the write-time rules are in `rules`, the sweep reads it elsewhere.

mod default;
mod rule_check;
mod rules;
mod types;
mod validation;

pub use default::{default_config, effective, export};
pub use rules::{Verdict, evaluate, has_write_rules};
pub use types::{Alert, Column, KanbanConfig, Limit, Match, RULE_KINDS, Rule};
pub use validation::{ConfigError, parse};
