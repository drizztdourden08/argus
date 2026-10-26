//! The launcher and bundle rules (DESIGN.md 13.1 and 13.7) as functions over the launcher
//! side of a board: records, the three kinds of launcher, the usage window, Play
//! definitions, bundle status, retire and restore, and the stale and master wakes.

mod audit;
mod authorize;
mod bundle;
mod bundle_ops;
mod every;
mod join;
mod naming;
mod play;
mod record;
mod retire;
mod shell;
mod stale;
mod stale_text;
mod usage;
mod write;

pub use audit::{audit, clip};
pub use authorize::authorize_launcher;
pub use bundle::{BundleStatus, LinkState, Resolved, bundle_status, listed_status, resolve_bundle};
pub use bundle_ops::{
    CloseOutcome, CloseRequest, check_hand, close_bundle, ensure_bundle, reopen_bundle,
    set_bundle_status,
};
pub use every::{EVERY_HELP, EVERY_LEGACY, Every, duration_text, parse_every};
pub use join::{has_master, is_live, is_standalone, joined_bundle, linked_bundles, made_for};
pub use naming::{
    DEF_PREFIX, KINDS, is_launcher_name, launcher_def_name, launcher_slug, require_launcher_name,
};
pub use play::{project_root, resolve_path, sync_launcher_play};
pub use record::{
    AuditLine, Bundle, Launcher, LauncherBoard, MASTER, Role, RuleError, ThreadInfo, Who,
};
pub use retire::{keep_launcher, restore_made_for, retire_launcher, retire_made_for};
pub use shell::{Platform, command_warnings, runs_note, shell_of};
pub use stale::{
    LauncherStale, LauncherUsage, REASON_MASTER, REASON_STALE, StaleConfig, StaleEnv, StaleNote,
    WakeEvent,
};
pub use stale_text::{MAX_TEXT, StaleItem, master_text, stale_text};
pub use usage::{Usage, last_used_ms, usage_of};
pub use write::{LauncherPatch, WriteContext, WriteResult, write_launcher};
