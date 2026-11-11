//! The Desktop app's sidebar folders. They live in three keys of the app's Local Storage
//! LevelDB: [`store`] reads and writes them, [`folders`] works out, as pure functions, which
//! folders and memberships each account scope is missing.

pub mod folders;
pub mod store;

use serde_json::{Map, Value, json};

pub use folders::{FolderPlan, FolderStep, Presence, fresh_folder_id, plan_folders};
pub use store::LevelDbSidebar;

/// The three keys that hold the sidebar folders, decoded, with the encoding byte each value
/// had in the store (0x00 UTF-16LE, 0x01 Latin-1) so a write uses the same one.
#[derive(Debug, Clone, PartialEq)]
pub struct SidebarKeys {
    /// `LSS-persisted.dframe-code-sections` as `{value, tabId, timestamp}`.
    pub code_sections: Value,
    /// `LSS-persisted.dframe-group-scopes` as `{value, tabId, timestamp}`.
    pub group_scopes: Value,
    /// `dframe-store` as `{state, version, shared}`.
    pub dframe_store: Value,
    /// The encoding bytes of the three values, in the same order.
    pub encodings: [u8; 3],
}

impl SidebarKeys {
    /// The shape [`crate::SidebarStore`] passes around: the three values and their encodings.
    #[must_use]
    pub fn to_value(&self) -> Value {
        json!({
            "codeSections": self.code_sections,
            "groupScopes": self.group_scopes,
            "dframeStore": self.dframe_store,
            "encodings": {
                "codeSections": self.encodings[0],
                "groupScopes": self.encodings[1],
                "dframeStore": self.encodings[2],
            },
        })
    }

    /// The inverse of [`SidebarKeys::to_value`]. A missing `encodings` entry defaults to
    /// Latin-1 (0x01), the one this store uses.
    ///
    /// # Errors
    /// When one of the three values is missing.
    pub fn from_value(v: &Value) -> Result<Self, String> {
        let part = |name: &str| {
            v.get(name)
                .cloned()
                .ok_or_else(|| format!("sidebar keys: `{name}` is missing"))
        };
        let empty = Map::new();
        let encs = v
            .get("encodings")
            .and_then(Value::as_object)
            .unwrap_or(&empty);
        let enc = |name: &str| {
            encs.get(name)
                .and_then(Value::as_u64)
                .and_then(|n| u8::try_from(n).ok())
                .unwrap_or(1)
        };
        Ok(Self {
            code_sections: part("codeSections")?,
            group_scopes: part("groupScopes")?,
            dframe_store: part("dframeStore")?,
            encodings: [enc("codeSections"), enc("groupScopes"), enc("dframeStore")],
        })
    }
}
