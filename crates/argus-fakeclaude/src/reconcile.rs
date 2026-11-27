//! What Claude does with its sidebar at start, as far as the account switch cares: it decides
//! whether the local store or the server copy wins, then marks the store as its account's.

use std::fs;
use std::path::Path;

use argus_accounts::sidebar::SidebarKeys;
use serde_json::{Value, json};

use crate::storage::Storage;

/// The flag naming the account that last synced the store.
pub const OWNER: &str = "ccd-sync-owner";
/// The flag naming the organization that last synced the store.
pub const ORG: &str = "ccd-sync-org";
/// Every flag of a change waiting to go to the server starts with this.
pub const PENDING_PREFIX: &str = "ccd-sync-pending";
/// The flag that says the local `dframe-store` is newer than the server copy.
pub const PENDING_STORE: &str = "ccd-sync-pending:ccd/dframe-store";

/// Runs the start-up reconcile for `account` in `org` and returns the keys the app now holds.
///
/// A store last synced by another account loses its pending flags. Then a pending store goes up
/// to `<remote>/<org>.json`; otherwise the server copy, if any, replaces the local store and the
/// two folder keys are derived from it again.
pub fn run(
    store: &mut Storage,
    remote: &Path,
    (account, org): (&str, &str),
) -> Result<SidebarKeys, String> {
    if store.flag(OWNER).as_deref() != Some(account) {
        store.drop_flags(PENDING_PREFIX)?;
    }
    let mut keys = store.read_keys()?;
    let file = remote.join(format!("{org}.json"));
    if store.flag(PENDING_STORE).as_deref() == Some("1") {
        upload(&keys.dframe_store, &file)?;
        store.drop_flags(PENDING_STORE)?;
    } else if let Some(server) = download(&file)? {
        keys.dframe_store = server;
        derive(&mut keys);
    }
    store.set_flag(OWNER, account)?;
    store.set_flag(ORG, org)?;
    Ok(keys)
}

fn upload(store: &Value, file: &Path) -> Result<(), String> {
    let text = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(file, text).map_err(|e| format!("cannot write {}: {e}", file.display()))
}

fn download(file: &Path) -> Result<Option<Value>, String> {
    match fs::read_to_string(file) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| format!("{} is not JSON: {e}", file.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("cannot read {}: {e}", file.display())),
    }
}

/// Makes the folder keys follow the scopes the `dframe-store` holds.
fn derive(keys: &mut SidebarKeys) {
    let state = keys.dframe_store.get("state").cloned().unwrap_or_default();
    let by_scope = |name: &str| {
        state
            .get(name)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default()
    };
    for (scope, groups) in by_scope("customGroupsByScope") {
        set_scope(&mut keys.group_scopes, &scope, groups);
    }
    for (scope, entry) in by_scope("codeSidebarByScope") {
        let sections = entry.get("sections").cloned().unwrap_or_else(|| json!([]));
        set_scope(
            &mut keys.code_sections,
            &scope,
            json!({ "sections": sections }),
        );
    }
}

fn set_scope(wrapper: &mut Value, scope: &str, entry: Value) {
    if let Some(map) = wrapper.get_mut("value").and_then(Value::as_object_mut) {
        map.insert(scope.to_owned(), entry);
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn keys() -> SidebarKeys {
        SidebarKeys {
            code_sections: json!({"value": {"s": {"sections": []}}}),
            group_scopes: json!({"value": {"s": {"groups": []}}}),
            dframe_store: json!({"state": {
                "customGroupsByScope": {"s": {"groups": [{"id": "g", "name": "Work"}]}},
                "codeSidebarByScope": {"s": {"sections": [{"id": "g"}]}},
            }}),
            encodings: [1, 1, 1],
        }
    }

    #[test]
    fn the_folder_keys_follow_the_store() {
        let mut keys = keys();
        derive(&mut keys);
        assert_eq!(keys.group_scopes["value"]["s"]["groups"][0]["name"], "Work");
        assert_eq!(keys.code_sections["value"]["s"]["sections"][0]["id"], "g");
    }

    #[test]
    fn a_missing_server_copy_is_none() {
        let dir = std::env::temp_dir().join("argus-fakeclaude-no-such-file.json");
        assert_eq!(download(&dir), Ok(None));
    }
}
