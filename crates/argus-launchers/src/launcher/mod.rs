//! The launcher rules (DESIGN.md 13.1 and 13.7) as functions over the launcher
//! side of a board: records, the three kinds of launcher, the usage window, Play
//! definitions, retire, and the stale and master wakes.

mod audit;
mod authorize;
mod every;
mod join;
mod link;
mod master_role;
mod naming;
mod need_link;
mod play;
mod record;
mod resolve;
mod retire;
mod shell;
mod stale;
mod stale_text;
mod usage;
mod write;

pub use audit::{audit, clip};
pub use authorize::authorize_launcher;
pub use every::{EVERY_HELP, EVERY_LEGACY, Every, duration_text, parse_every};
pub use join::{has_master, is_live, is_master, is_standalone, joined_bundle, require_master};
pub use link::{link_launcher, unlink_launcher};
pub use naming::{
    DEF_PREFIX, KINDS, is_launcher_name, launcher_def_name, launcher_slug, require_launcher_name,
};
pub use play::{project_root, resolve_path, sync_launcher_play};
pub use record::{
    AuditLine, Bundle, Launcher, LauncherBoard, LinkMark, MASTER, Role, RuleError, ThreadInfo, Who,
};
pub use resolve::{LinkState, Resolved, resolve_launcher};
pub use retire::{keep_launcher, retire_launcher};
pub use shell::{Platform, command_warnings, runs_note, shell_of};
pub use stale::{
    LauncherStale, LauncherUsage, REASON_MASTER, REASON_STALE, StaleConfig, StaleEnv, StaleNote,
    WakeEvent,
};
pub use stale_text::{MAX_TEXT, StaleItem, master_text, stale_text};
pub use usage::{Usage, last_used_ms, usage_of};
pub use write::{LauncherPatch, WriteContext, WriteResult, write_launcher};
