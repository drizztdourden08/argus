//! The launcher side of a board: the records in `board.launchers` and `board.builds`, the
//! threads the rules look at, the Play definitions in `board.runDefs`, and the audit lines.
//! Everything serialises in the camelCase shape of the board file.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::def::RunDef;

pub const MASTER: &str = "master";

/// A plain sentence the CLI prints as the reason a write was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleError(pub String);

impl RuleError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RuleError {}

fn app_kind() -> String {
    "app".to_owned()
}

/// How a build stands with the link rule, kept flat in the JSON (`master`, `relink`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkMark {
    /// The master build of the board: needs no item and never closes with one.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub master: bool,
    /// A live build with no clear item, left by `coord build migrate-links` for its lead to link.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub relink: bool,
}

/// One startable build of a project (`board.launchers[name]`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Launcher {
    pub name: String,
    pub title: Option<String>,
    pub cmd: Option<String>,
    pub cwd: Option<String>,
    #[serde(default = "app_kind")]
    pub kind: String,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub note: Option<String>,
    pub current: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub window: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_every: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kept_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kept_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kept_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kept_until: Option<String>,
    pub retired_at: Option<String>,
    pub retired_by: Option<String>,
    pub retire_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retired_by_bundle: Option<String>,
    /// The work item this build serves (`for`): the build closes when the item does.
    #[serde(rename = "for", skip_serializing_if = "Option::is_none")]
    pub for_item: Option<String>,
    /// The link marks: `master` and `relink`.
    #[serde(flatten)]
    pub link: LinkMark,
    /// The item whose closing retired this build; a reopen of it brings the build back.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retired_by_item: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub by: Option<String>,
}

impl Default for Launcher {
    fn default() -> Self {
        Self {
            name: String::new(),
            title: None,
            cmd: None,
            cwd: None,
            kind: app_kind(),
            branch: None,
            commit: None,
            note: None,
            current: false,
            window: false,
            bundle: None,
            use_every: None,
            kept_at: None,
            kept_by: None,
            kept_note: None,
            kept_until: None,
            retired_at: None,
            retired_by: None,
            retire_note: None,
            retired_by_bundle: None,
            for_item: None,
            link: LinkMark::default(),
            retired_by_item: None,
            created_at: String::new(),
            created_by: None,
            updated_at: String::new(),
            by: None,
        }
    }
}

/// A bundle: a build tag with its tests and the launcher that starts it
/// (`board.builds[tag]`). A record written before launchers carries `cmd`, `cwd` and
/// `window` itself.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Bundle {
    pub tag: String,
    pub title: Option<String>,
    pub note: Option<String>,
    pub launcher: Option<String>,
    pub thread: Option<String>,
    pub status: Option<String>,
    pub closed_at: Option<String>,
    pub closed_by: Option<String>,
    pub close_note: Option<String>,
    pub cmd: Option<String>,
    pub cwd: Option<String>,
    pub window: bool,
    pub last_handed_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub created_by: Option<String>,
}

/// A registered thread, as far as the rules look at it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ThreadInfo {
    pub id: String,
    pub role: String,
    pub status: String,
    pub root: Option<String>,
    pub placeholder: bool,
    pub deleted_at: Option<String>,
}

/// One line of the board's audit log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLine {
    pub at: String,
    pub thread: String,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LauncherBoard {
    pub project: Option<String>,
    pub kind: Option<String>,
    pub created_at: Option<String>,
    pub launchers: BTreeMap<String, Launcher>,
    pub builds: BTreeMap<String, Bundle>,
    pub threads: BTreeMap<String, ThreadInfo>,
    pub run_defs: BTreeMap<String, RunDef>,
    pub handed_build: Option<String>,
    pub handed_at: Option<String>,
    pub audit: Vec<AuditLine>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Lead,
    Human,
    Thread,
}

/// Who is writing: the actor name for the audit log, the role, and the calling thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Who {
    pub actor: String,
    pub role: Role,
    pub thread: Option<ThreadInfo>,
}

impl Who {
    pub fn is_lead(&self) -> bool {
        matches!(self.role, Role::Lead | Role::Human)
    }
}
