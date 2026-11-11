//! Claude accounts of the owner: which accounts exist on this PC, their rules and their usage.
//! Later pieces add the usage poller, the chat sync, the switch and the handover on top of the
//! types, the config and the ports defined here. The outside world (the usage endpoint, the
//! login files, the Desktop app, the boards) is reached only through [`ports`], so tests fake it.

// The tests assert emptiness with `assert!(x.is_empty())`, as `argus-launchers` does.
#![cfg_attr(test, allow(clippy::assert_is_empty))]
#![allow(clippy::missing_errors_doc)]

pub mod alerts;
pub mod catalog;
pub mod config;
pub mod credentials;
pub mod desktop;
pub mod handover;
pub mod ports;
pub mod registry;
pub mod resume;
pub mod rules;
pub mod sidebar;
pub mod sync;
pub mod types;
pub mod usage;

#[cfg(test)]
mod fixtures;

pub use config::{AccountsConfig, ArchiveRule, EstimateDefaults};
pub use credentials::CliCredentials;
pub use ports::{
    Catalog, Clock, Credentials, DesktopApp, HandoverStore, LoginVault, Resumer, SidebarStore,
    Threads, UsageSource,
};
pub use registry::{AuthProbe, ClaudeAuthProbe, discover, find_app_data, find_cli_exe};
pub use types::{
    Account, AccountKey, CliStatus, HandoverMode, Rules, Scope, Severity, Usage, UsageLimit,
};
pub use usage::HttpUsage;
pub use usage::poller::{BurnHistory, Poll, PollResult, binding_limit, next_interval, poll_all};
