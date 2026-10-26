//! The board shortcut: one configuration in a checkout's `.claude/launch.json` that the Browser
//! pane of Claude Desktop lists on a new tab, so Play opens the right board. The alias urls and
//! the file merge live here; the verb `coord board shortcut` is the board group's.

pub mod alias;
pub mod launch;

pub use alias::{alias_url, is_overlord, shortcut_config, shortcut_configs};
pub use launch::{Written, launch_file, needs_shortcut, read_launch, write_shortcuts};
