//! Claude accounts of the owner: which accounts exist on this PC, their rules and their usage.
//! Later pieces add the usage poller, the chat sync, the switch and the handover on top of the
//! types, the config and the ports defined here. The outside world (the usage endpoint, the
//! login files, the Desktop app, the boards) is reached only through [`ports`], so tests fake it.

pub mod config;
pub mod ports;
pub mod registry;
pub mod types;

pub use config::{AccountsConfig, ArchiveRule, EstimateDefaults};
pub use ports::{
    Catalog, Clock, Credentials, DesktopApp, LoginVault, Resumer, SidebarStore, Threads,
    UsageSource,
};
pub use registry::{AuthProbe, ClaudeAuthProbe, discover, find_app_data, find_cli_exe};
pub use types::{
    Account, AccountKey, CliStatus, HandoverMode, Rules, Scope, Severity, Usage, UsageLimit,
};
