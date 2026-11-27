//! A made-up Claude Desktop home for the headless account switch: a Local Storage `LevelDB`
//! with the sidebar of two fake accounts, empty chat scopes, a `config.json` signed in as the
//! first account, saved logins and the server copy of each organization. Everything is built
//! here from literals. Nothing is copied from, or read out of, the owner's real Claude.

use std::fs;
use std::path::{Path, PathBuf};

use argus_accounts::sidebar::store::{KEY_NAMES, KEY_PREFIX, encode_value};
use rusty_leveldb::{DB, Options};
use serde_json::{Map, Value, json};

pub const ACCOUNT_A: &str = "a1a1a1a1-0000-4000-8000-00000000000a";
pub const ORG_A: &str = "0a0a0a0a-0000-4000-8000-00000000000a";
pub const ACCOUNT_B: &str = "b2b2b2b2-0000-4000-8000-00000000000b";
pub const ORG_B: &str = "0b0b0b0b-0000-4000-8000-00000000000b";
/// The Argus keys of the two accounts.
pub const FIRST: &str = "first";
pub const SECOND: &str = "second";

/// The folders the first account has and the second lacks.
const FOLDERS: [&str; 3] = ["Work", "Home", "Ideas"];
const LATIN1: u8 = 1;

/// Where the made-up home lives.
pub struct SimHome {
    /// The app data folder.
    pub app: PathBuf,
    /// The server copies, one `<org>.json` each.
    pub remote: PathBuf,
    /// The file that asks the fake app to quit.
    pub quit_file: PathBuf,
    /// The marker that makes the next start of the fake fail.
    pub fail_marker: PathBuf,
    root: PathBuf,
}

fn section(id: &str, kind: &str, name: &str, order: usize) -> Value {
    json!({"id": id, "kind": kind, "v": 2, "name": name, "order": order, "members": []})
}

/// The sections and the groups of one scope with the given folders.
fn scope_data(folders: &[&str]) -> (Value, Value) {
    let mut sections = vec![section("pinned", "pinned", "Pinned", 0)];
    let mut groups = Vec::new();
    for (i, name) in folders.iter().enumerate() {
        let id = format!("cg-{}", name.to_lowercase());
        sections.push(section(&id, "manual", name, i + 1));
        groups.push(json!({"id": id, "name": name, "mode": "code"}));
    }
    sections.push(section("sessions", "sessions", "Sessions", 99));
    let groups = json!({"groups": groups, "assignments": {}, "order": {}});
    (json!({ "sections": sections }), groups)
}

fn scope_name(account: &str, org: &str) -> String {
    format!("{account}/{org}")
}

/// The three sidebar values: the first account has the folders, the second has none.
fn sidebar_values() -> [Value; 3] {
    let scopes = [
        (scope_name(ACCOUNT_A, ORG_A), scope_data(&FOLDERS)),
        (scope_name(ACCOUNT_B, ORG_B), scope_data(&[])),
    ];
    let (mut code, mut groups) = (Map::new(), Map::new());
    let (mut custom, mut sidebar) = (Map::new(), Map::new());
    for (scope, (sections, group)) in scopes {
        sidebar.insert(
            scope.clone(),
            json!({"migratedFrom": 1, "sections": sections["sections"]}),
        );
        custom.insert(scope.clone(), group.clone());
        code.insert(scope.clone(), sections);
        groups.insert(scope, group);
    }
    [
        json!({"value": code, "tabId": "fake", "timestamp": 1}),
        json!({"value": groups, "tabId": "fake", "timestamp": 1}),
        json!({"state": {"customGroupsByScope": custom, "codeSidebarByScope": sidebar}, "version": 3}),
    ]
}

fn write_store(app: &Path, values: &[Value; 3]) {
    let dir = app.join("Local Storage").join("leveldb");
    fs::create_dir_all(&dir).expect("leveldb dir");
    let options = Options {
        create_if_missing: true,
        ..Options::default()
    };
    let mut db = DB::open(&dir, options).expect("seed store");
    for (name, value) in KEY_NAMES.iter().zip(values) {
        let mut key = KEY_PREFIX.to_vec();
        key.extend_from_slice(name.as_bytes());
        let raw = encode_value(LATIN1, value).expect("encode");
        db.put(&key, &raw).expect("seed key");
    }
    db.flush().expect("flush");
    db.close().expect("close");
}

fn write_json(path: &Path, value: &Value) {
    fs::create_dir_all(path.parent().expect("parent")).expect("dir");
    fs::write(path, serde_json::to_string_pretty(value).expect("json")).expect("write");
}

/// A login of the app: `config.json` signed in as `account`, with fake tokens.
fn login_config(account: &str, token: &str) -> Value {
    json!({
        "locale": "en",
        "theme": "dark",
        "oauth:tokenCache": token,
        "lastKnownAccountUuid": account,
    })
}

fn save_login(logins: &Path, key: &str, account: &str) {
    let folder = logins.join(key);
    let token = format!("fake-token-{key}");
    let record = json!({
        "accountUuid": account,
        "savedAt": "2026-01-01T00:00:00.000Z",
        "config": {"oauth:tokenCache": token, "lastKnownAccountUuid": account},
    });
    write_json(&folder.join("login.json"), &record);
    fs::write(folder.join("Cookies"), format!("fake-cookies-{key}")).expect("cookies");
}

impl SimHome {
    /// Builds the home under `root`, with the saved logins in `logins` (the server's
    /// `accounts\logins` folder). The app is signed in as the first account.
    pub fn build(root: &Path, logins: &Path) -> SimHome {
        let app = root.join("app");
        write_store(&app, &sidebar_values());
        write_json(
            &app.join("config.json"),
            &login_config(ACCOUNT_A, "fake-token-first"),
        );
        let network = app.join("Network");
        fs::create_dir_all(&network).expect("network dir");
        fs::write(network.join("Cookies"), "fake-cookies-first").expect("cookies");
        for (account, org) in [(ACCOUNT_A, ORG_A), (ACCOUNT_B, ORG_B)] {
            let scope = app.join("claude-code-sessions").join(account).join(org);
            fs::create_dir_all(scope).expect("scope dir");
        }
        let remote = root.join("remote");
        let [_, _, store] = sidebar_values();
        for org in [ORG_A, ORG_B] {
            write_json(&remote.join(format!("{org}.json")), &store);
        }
        save_login(logins, FIRST, ACCOUNT_A);
        save_login(logins, SECOND, ACCOUNT_B);
        SimHome {
            app,
            remote,
            quit_file: root.join("quit"),
            fail_marker: root.join("fail-next-start"),
            root: root.to_path_buf(),
        }
    }

    /// The `accounts` setting of the server config for the two fake accounts.
    pub fn accounts_setting(&self, force_quit: bool) -> Value {
        let account = |key: &str, desktop: (&str, &str)| {
            json!({
                "key": key, "label": key, "color": "#112233",
                "cliDir": slashed(&self.root.join(format!("cli-{key}"))),
                "desktopAccount": desktop.0, "desktopOrg": desktop.1,
            })
        };
        json!({ "accounts": {
            "appData": slashed(&self.app),
            "cliRoot": slashed(&self.root.join("none")),
            "forceQuit": force_quit,
            "accounts": [account(FIRST, (ACCOUNT_A, ORG_A)), account(SECOND, (ACCOUNT_B, ORG_B))],
        } })
    }

    /// The account `config.json` says is signed in.
    pub fn signed_in(&self) -> String {
        let text = fs::read_to_string(self.app.join("config.json")).expect("config");
        let config: Value = serde_json::from_str(&text).expect("json");
        config["lastKnownAccountUuid"]
            .as_str()
            .expect("account")
            .to_owned()
    }
}

fn slashed(path: &Path) -> String {
    path.display().to_string().replace(char::from(92), "/")
}
