//! The saved Desktop logins, one folder per account under `<DASHBOARD_HOME>\accounts\logins`.
//!
//! A Desktop login is three keys of `<app>\config.json` and the cookie files in
//! `<app>\Network`. The values are opaque: they are copied, never decoded, never logged and
//! never part of an error text. The folder is restricted to the current user.

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::clock::now_iso;
use argus_core::process::hidden_command;
use serde_json::{Map, Value, json};

use crate::catalog::read::write_atomic;
use crate::ports::LoginVault;
use crate::sync::lock_running;
use crate::types::Account;

/// The keys of `config.json` that make up the login.
pub const LOGIN_KEYS: [&str; 3] = [
    "oauth:tokenCache",
    "oauth:tokenCacheV2",
    "lastKnownAccountUuid",
];
const ACCOUNT_KEY: &str = "lastKnownAccountUuid";
const COOKIES: &str = "Cookies";
const JOURNAL: &str = "Cookies-journal";
const LOGIN_FILE: &str = "login.json";
const LOGIN_PREV: &str = "login.prev.json";
const COOKIES_PREV: &str = "Cookies.prev";

/// The Desktop account that is logged in now, from `<app>\config.json`.
#[must_use]
pub fn last_known_account(app: &Path) -> Option<String> {
    let text = fs::read_to_string(app.join("config.json")).ok()?;
    let config: Value = serde_json::from_str(&text).ok()?;
    config
        .get(ACCOUNT_KEY)
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn io_err(what: &str, path: &Path, e: &std::io::Error) -> String {
    format!("{what} {}: {e}", path.display())
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|e| io_err("cannot read", path, &e))?;
    serde_json::from_str(&text).map_err(|_| format!("{} is not valid JSON", path.display()))
}

/// The saved logins on disk.
pub struct FsVault {
    app: PathBuf,
    root: PathBuf,
    restrict: bool,
}

impl FsVault {
    /// The vault of the app data folder `app`, kept in `root` (`...\accounts\logins`).
    #[must_use]
    pub fn new(app: &Path, root: &Path) -> Self {
        FsVault {
            app: app.to_path_buf(),
            root: root.to_path_buf(),
            restrict: true,
        }
    }

    /// The folder of an account. The key names a folder, so it must be a plain name.
    fn folder(&self, account: &Account) -> Result<PathBuf, String> {
        let plain = !account.key.is_empty()
            && account
                .key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'));
        if plain {
            Ok(self.root.join(&account.key))
        } else {
            Err(format!(
                "the account key {:?} is not a plain name",
                account.key
            ))
        }
    }

    fn cookie_dir(&self) -> PathBuf {
        self.app.join("Network")
    }

    fn config_path(&self) -> PathBuf {
        self.app.join("config.json")
    }

    /// Makes the folder and keeps everyone but the current user out of it.
    fn prepare(&self, folder: &Path) -> Result<(), String> {
        fs::create_dir_all(folder).map_err(|e| io_err("cannot create", folder, &e))?;
        if !self.restrict {
            return Ok(());
        }
        let user = std::env::var("USERNAME").map_err(|_| "no USERNAME to grant".to_owned())?;
        let status = hidden_command("icacls")
            .arg(folder)
            .args(["/inheritance:r", "/grant:r"])
            .arg(format!("{user}:(OI)(CI)F"))
            .output()
            .map_err(|e| format!("icacls did not run: {e}"))?;
        if status.status.success() {
            Ok(())
        } else {
            Err(format!(
                "the login folder {} could not be restricted",
                folder.display()
            ))
        }
    }

    /// The saved record of an account: its `login.json`.
    fn record(&self, account: &Account) -> Result<Value, String> {
        let path = self.folder(account)?.join(LOGIN_FILE);
        read_json(&path).map_err(|_| format!("no login is saved for {}", account.label))
    }

    fn copy_cookies(from: &Path, to: &Path) -> Result<(), String> {
        let main = from.join(COOKIES);
        let bytes = fs::read(&main).map_err(|e| io_err("cannot read", &main, &e))?;
        write_atomic(&to.join(COOKIES), &bytes)?;
        let journal = to.join(JOURNAL);
        match fs::read(from.join(JOURNAL)) {
            Ok(bytes) => write_atomic(&journal, &bytes),
            Err(_) => fs::remove_file(&journal)
                .or_else(ignore_missing)
                .map_err(|e| io_err("cannot remove", &journal, &e)),
        }
    }

    /// Keeps the entry that is about to be overwritten as `login.prev.json` and `Cookies.prev`,
    /// one generation back.
    fn keep_previous(folder: &Path) -> Result<(), String> {
        for (name, prev) in [(LOGIN_FILE, LOGIN_PREV), (COOKIES, COOKIES_PREV)] {
            let from = folder.join(name);
            if from.exists() {
                let to = folder.join(prev);
                fs::copy(&from, &to).map_err(|e| io_err("cannot keep", &to, &e))?;
            }
        }
        Ok(())
    }
}

/// One file of a staged replacement: `Some` bytes are written, `None` removes the file.
type Replacement = (PathBuf, Option<Vec<u8>>);

/// Replaces several files so a failure while writing leaves every one of them as it was: all new
/// contents are written to temp files next to their targets first, and only then renamed over.
fn replace_all(files: Vec<Replacement>) -> Result<(), String> {
    let mut staged: Vec<(PathBuf, Option<PathBuf>)> = Vec::new();
    for (dest, bytes) in files {
        let temp = bytes.map(|bytes| stage(&dest, &bytes)).transpose();
        match temp {
            Ok(temp) => staged.push((dest, temp)),
            Err(e) => {
                discard(&staged);
                return Err(e);
            }
        }
    }
    for (dest, temp) in &staged {
        let moved = match temp {
            Some(temp) => fs::rename(temp, dest),
            None => fs::remove_file(dest).or_else(ignore_missing),
        };
        moved.map_err(|e| io_err("cannot replace", dest, &e))?;
    }
    Ok(())
}

fn stage(dest: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    let name = dest.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let temp = dest.with_file_name(format!("{name}.argus-new"));
    fs::write(&temp, bytes).map_err(|e| io_err("cannot write", &temp, &e))?;
    Ok(temp)
}

fn discard(staged: &[(PathBuf, Option<PathBuf>)]) {
    for temp in staged.iter().filter_map(|(_, temp)| temp.as_ref()) {
        let _ = fs::remove_file(temp);
    }
}

fn ignore_missing(e: std::io::Error) -> std::io::Result<()> {
    if e.kind() == std::io::ErrorKind::NotFound {
        Ok(())
    } else {
        Err(e)
    }
}

/// The login keys present in a config object.
fn login_keys_of(config: &Value) -> Map<String, Value> {
    let mut keys = Map::new();
    for key in LOGIN_KEYS {
        if let Some(value) = config.get(key) {
            keys.insert(key.to_owned(), value.clone());
        }
    }
    keys
}

/// `config` with the login keys of `saved` put in: a key the saved login lacks is removed, every
/// other key stays where it was. New keys go to the end.
fn with_login(config: &Value, saved: &Value) -> Result<Value, String> {
    let mut next = config
        .as_object()
        .cloned()
        .ok_or_else(|| "config.json is not an object".to_owned())?;
    for key in LOGIN_KEYS {
        match saved.get(key) {
            Some(value) => {
                next.insert(key.to_owned(), value.clone());
            }
            None => {
                next.shift_remove(key);
            }
        }
    }
    Ok(Value::Object(next))
}

impl LoginVault for FsVault {
    fn save(&self, account: &Account) -> Result<(), String> {
        let uuid = account
            .desktop_account
            .as_deref()
            .ok_or_else(|| format!("{} has no Desktop account yet", account.label))?;
        let config = read_json(&self.config_path())?;
        if config.get(ACCOUNT_KEY).and_then(Value::as_str) != Some(uuid) {
            return Err(format!(
                "Claude Desktop is not logged in as {}; its login was not saved",
                account.label
            ));
        }
        let folder = self.folder(account)?;
        self.prepare(&folder)?;
        Self::keep_previous(&folder)?;
        Self::copy_cookies(&self.cookie_dir(), &folder)?;
        let record = json!({
            "accountUuid": uuid,
            "savedAt": now_iso(),
            "config": login_keys_of(&config),
        });
        let bytes = serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?;
        write_atomic(&folder.join(LOGIN_FILE), &bytes)
    }

    fn restore(&self, account: &Account) -> Result<(), String> {
        if lock_running(&self.app) {
            return Err("Claude is running; its login cannot be replaced".to_owned());
        }
        let record = self.record(account)?;
        let uuid = account.desktop_account.as_deref();
        if record.get("accountUuid").and_then(Value::as_str) != uuid {
            return Err(format!(
                "the saved login does not belong to {}",
                account.label
            ));
        }
        let saved = record
            .get("config")
            .ok_or_else(|| format!("the saved login of {} is damaged", account.label))?;
        let next = with_login(&read_json(&self.config_path())?, saved)?;
        let bytes = serde_json::to_vec_pretty(&next).map_err(|e| e.to_string())?;
        let folder = self.folder(account)?;
        let cookies = self.cookie_dir();
        fs::create_dir_all(&cookies).map_err(|e| io_err("cannot create", &cookies, &e))?;
        let main = folder.join(COOKIES);
        let jar = fs::read(&main).map_err(|e| io_err("cannot read", &main, &e))?;
        replace_all(vec![
            (cookies.join(COOKIES), Some(jar)),
            (cookies.join(JOURNAL), fs::read(folder.join(JOURNAL)).ok()),
            (self.config_path(), Some(bytes)),
        ])
    }

    fn saved_at(&self, account: &Account) -> Option<String> {
        let record = self.record(account).ok()?;
        record
            .get("savedAt")
            .and_then(Value::as_str)
            .map(str::to_owned)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use serde_json::{Value, json};
    use tempfile::TempDir;

    use super::{FsVault, last_known_account};
    use crate::ports::LoginVault;
    use crate::types::{Account, Rules};

    fn account(key: &str, uuid: &str) -> Account {
        Account {
            key: key.to_owned(),
            label: key.to_owned(),
            color: "#000000".to_owned(),
            desktop_account: Some(uuid.to_owned()),
            desktop_org: None,
            cli_dir: String::new(),
            can_receive: true,
            rules: Rules::default(),
        }
    }

    fn write_app(app: &Path, uuid: &str, token: &str, cookies: &str) {
        let config = json!({
            "locale": "en",
            "oauth:tokenCache": token,
            "theme": "dark",
            "lastKnownAccountUuid": uuid,
        });
        fs::write(app.join("config.json"), config.to_string()).unwrap();
        fs::create_dir_all(app.join("Network")).unwrap();
        fs::write(app.join("Network").join("Cookies"), cookies).unwrap();
    }

    fn fixture() -> (TempDir, FsVault) {
        let dir = TempDir::new().unwrap();
        let app = dir.path().join("app");
        fs::create_dir_all(&app).unwrap();
        let root = dir.path().join("logins");
        let vault = FsVault {
            app,
            root,
            restrict: false,
        };
        (dir, vault)
    }

    fn config_of(vault: &FsVault) -> Value {
        serde_json::from_str(&fs::read_to_string(vault.app.join("config.json")).unwrap()).unwrap()
    }

    #[test]
    fn save_refuses_when_another_account_is_logged_in() {
        let (_dir, vault) = fixture();
        write_app(&vault.app, "uuid-b", "tok-b", "cookies-b");
        let err = vault.save(&account("a", "uuid-a")).unwrap_err();
        assert!(err.contains("not logged in as a"));
        assert!(!vault.root.exists());
    }

    #[test]
    fn save_then_restore_swaps_the_login_and_keeps_other_keys() {
        let (_dir, vault) = fixture();
        write_app(&vault.app, "uuid-a", "tok-a", "cookies-a");
        vault.save(&account("a", "uuid-a")).unwrap();
        write_app(&vault.app, "uuid-b", "tok-b", "cookies-b");
        fs::write(vault.app.join("Network").join("Cookies-journal"), "stale").unwrap();
        vault.restore(&account("a", "uuid-a")).unwrap();
        let config = config_of(&vault);
        assert_eq!(config["oauth:tokenCache"], "tok-a");
        assert_eq!(config["lastKnownAccountUuid"], "uuid-a");
        assert_eq!(config["theme"], "dark");
        let order: Vec<&String> = config.as_object().unwrap().keys().collect();
        assert_eq!(order[0], "locale");
        assert_eq!(order[2], "theme");
        let cookies = fs::read_to_string(vault.app.join("Network").join("Cookies")).unwrap();
        assert_eq!(cookies, "cookies-a");
        assert!(!vault.app.join("Network").join("Cookies-journal").exists());
    }

    #[test]
    fn restore_adds_a_missing_key_and_removes_one_the_login_lacks() {
        let (_dir, vault) = fixture();
        write_app(&vault.app, "uuid-a", "tok-a", "cookies-a");
        vault.save(&account("a", "uuid-a")).unwrap();
        let mut other = config_of(&vault);
        other["oauth:tokenCacheV2"] = json!("v2-b");
        other.as_object_mut().unwrap().remove("oauth:tokenCache");
        fs::write(vault.app.join("config.json"), other.to_string()).unwrap();
        vault.restore(&account("a", "uuid-a")).unwrap();
        let config = config_of(&vault);
        assert_eq!(config["oauth:tokenCache"], "tok-a");
        assert!(config.get("oauth:tokenCacheV2").is_none());
    }

    #[test]
    fn restore_refuses_without_a_saved_login_or_for_another_account() {
        let (_dir, vault) = fixture();
        write_app(&vault.app, "uuid-a", "tok-a", "cookies-a");
        assert!(
            vault
                .restore(&account("a", "uuid-a"))
                .unwrap_err()
                .contains("no login")
        );
        vault.save(&account("a", "uuid-a")).unwrap();
        fs::rename(vault.root.join("a"), vault.root.join("b")).unwrap();
        assert!(
            vault
                .restore(&account("b", "uuid-b"))
                .unwrap_err()
                .contains("does not belong")
        );
    }

    #[test]
    fn a_second_save_keeps_the_first_as_the_previous_generation() {
        let (_dir, vault) = fixture();
        let a = account("a", "uuid-a");
        write_app(&vault.app, "uuid-a", "tok-1", "cookies-1");
        vault.save(&a).unwrap();
        assert!(!vault.root.join("a").join("login.prev.json").exists());
        write_app(&vault.app, "uuid-a", "tok-2", "cookies-2");
        vault.save(&a).unwrap();
        let folder = vault.root.join("a");
        assert_eq!(
            fs::read_to_string(folder.join("Cookies.prev")).unwrap(),
            "cookies-1"
        );
        assert_eq!(
            fs::read_to_string(folder.join("Cookies")).unwrap(),
            "cookies-2"
        );
        let prev: Value =
            serde_json::from_str(&fs::read_to_string(folder.join("login.prev.json")).unwrap())
                .unwrap();
        assert_eq!(prev["config"]["oauth:tokenCache"], "tok-1");
    }

    #[test]
    fn restore_leaves_no_temp_files_behind() {
        let (_dir, vault) = fixture();
        write_app(&vault.app, "uuid-a", "tok-a", "cookies-a");
        vault.save(&account("a", "uuid-a")).unwrap();
        vault.restore(&account("a", "uuid-a")).unwrap();
        let leftovers = |dir: &Path| {
            fs::read_dir(dir)
                .unwrap()
                .flatten()
                .any(|e| e.file_name().to_string_lossy().ends_with(".argus-new"))
        };
        assert!(!leftovers(&vault.app) && !leftovers(&vault.app.join("Network")));
    }

    #[test]
    fn saved_at_follows_the_save() {
        let (_dir, vault) = fixture();
        write_app(&vault.app, "uuid-a", "tok-a", "cookies-a");
        let a = account("a", "uuid-a");
        assert!(vault.saved_at(&a).is_none());
        vault.save(&a).unwrap();
        assert!(vault.saved_at(&a).unwrap().ends_with('Z'));
    }

    #[test]
    fn errors_never_carry_the_values() {
        let (_dir, vault) = fixture();
        write_app(&vault.app, "uuid-b", "SECRET-TOKEN", "cookies");
        let err = vault.save(&account("a", "uuid-a")).unwrap_err();
        assert!(!err.contains("SECRET-TOKEN"));
    }

    #[test]
    fn a_key_with_a_path_is_refused() {
        let (_dir, vault) = fixture();
        write_app(&vault.app, "uuid-a", "tok-a", "cookies-a");
        assert!(vault.save(&account("..\\x", "uuid-a")).is_err());
    }

    #[test]
    fn the_active_account_is_read_from_the_config() {
        let (_dir, vault) = fixture();
        assert!(last_known_account(&vault.app).is_none());
        write_app(&vault.app, "uuid-a", "tok-a", "cookies-a");
        assert_eq!(last_known_account(&vault.app).as_deref(), Some("uuid-a"));
    }
}
