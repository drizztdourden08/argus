//! Finds the accounts on this PC: the Desktop scopes (`claude-code-sessions/<account>/<org>`)
//! and the CLI folders (`<cliRoot>/<key>`), matched by the organization the CLI login is in.

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::process::hidden_command;

use crate::types::{Account, CliStatus, Rules, Scope};

const COLORS: [&str; 6] = [
    "#6f9cf0", "#e0955f", "#6fcf97", "#c792ea", "#f2c94c", "#eb7f8f",
];

/// Asks a CLI folder who is logged in. A trait so tests fake the process.
pub trait AuthProbe: Send + Sync {
    /// The login of `config_dir`, or `None` when the CLI cannot say.
    fn status(&self, exe: &Path, config_dir: &Path) -> Option<CliStatus>;
}

/// Runs `claude auth status --json` with `CLAUDE_CONFIG_DIR` set.
pub struct ClaudeAuthProbe;

impl AuthProbe for ClaudeAuthProbe {
    fn status(&self, exe: &Path, config_dir: &Path) -> Option<CliStatus> {
        let out = hidden_command(exe)
            .args(["auth", "status", "--json"])
            .env("CLAUDE_CONFIG_DIR", config_dir)
            .output()
            .ok()?;
        parse_status(&String::from_utf8_lossy(&out.stdout))
    }
}

/// The status JSON the CLI printed, if it is one.
#[must_use]
pub fn parse_status(text: &str) -> Option<CliStatus> {
    serde_json::from_str(text.trim()).ok()
}

/// The Desktop app's data folder: the configured one, else the first
/// `%LOCALAPPDATA%\Claude-Data\claude_*`, else `%APPDATA%\Claude`.
#[must_use]
pub fn find_app_data(configured: Option<&str>) -> Option<PathBuf> {
    if let Some(dir) = configured.filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let relocated = local.and_then(|l| first_dir_with_prefix(&l.join("Claude-Data"), "claude_"));
    relocated.or_else(|| {
        let dir = PathBuf::from(std::env::var_os("APPDATA")?).join("Claude");
        dir.is_dir().then_some(dir)
    })
}

fn first_dir_with_prefix(parent: &Path, prefix: &str) -> Option<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(parent)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with(prefix))
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    found.sort();
    found.into_iter().next()
}

/// The newest CLI the Desktop app carries: `<app>/claude-code/<ver>/<hash>/claude.exe`.
#[must_use]
pub fn find_cli_exe(app: &Path) -> Option<PathBuf> {
    let versions = subdirs(&app.join("claude-code"));
    let newest = versions
        .into_iter()
        .max_by_key(|p| version_key(&file_name(p)))?;
    subdirs(&newest)
        .into_iter()
        .map(|hash| hash.join("claude.exe"))
        .find(|exe| exe.is_file())
}

fn subdirs(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    out.sort();
    out
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// `2.1.138` as `[2, 1, 138]`, so versions compare by number.
fn version_key(name: &str) -> Vec<u64> {
    name.split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

/// The scopes under `<app>/claude-code-sessions`.
#[must_use]
pub fn scan_scopes(app: &Path) -> Vec<Scope> {
    let mut out = Vec::new();
    for account in subdirs(&app.join("claude-code-sessions")) {
        for org in subdirs(&account) {
            out.push(Scope {
                account: file_name(&account),
                org: file_name(&org),
            });
        }
    }
    out
}

/// The CLI folders under `cli_root`, as `(key, folder)`.
#[must_use]
pub fn scan_cli_dirs(cli_root: &Path) -> Vec<(String, PathBuf)> {
    subdirs(cli_root)
        .into_iter()
        .map(|dir| (file_name(&dir), dir))
        .collect()
}

/// `hotmail` as `Hotmail`.
#[must_use]
pub fn capitalised(key: &str) -> String {
    let mut chars = key.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn account_of(index: usize, key: &str, dir: &Path, home: Option<&Scope>) -> Account {
    Account {
        key: key.to_owned(),
        label: capitalised(key),
        color: COLORS[index % COLORS.len()].to_owned(),
        desktop_account: home.map(|s| s.account.clone()),
        desktop_org: home.map(|s| s.org.clone()),
        cli_dir: dir.display().to_string().replace('\\', "/"),
        can_receive: true,
        rules: Rules::default(),
    }
}

/// The accounts of this PC. A CLI folder is matched to the scope whose organization is the
/// organization of its login; a folder the CLI cannot answer for stays a CLI-only account.
#[must_use]
pub fn discover(app: Option<&Path>, cli_root: &Path, probe: &dyn AuthProbe) -> Vec<Account> {
    let exe = app.and_then(find_cli_exe);
    let scopes = app.map(scan_scopes).unwrap_or_default();
    scan_cli_dirs(cli_root)
        .iter()
        .enumerate()
        .map(|(i, (key, dir))| {
            let org = exe
                .as_deref()
                .and_then(|exe| probe.status(exe, dir))
                .and_then(|status| status.org_id);
            let home = org.and_then(|o| scopes.iter().find(|s| s.org == o));
            account_of(i, key, dir, home)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    struct FakeProbe(Vec<(&'static str, &'static str)>);

    impl AuthProbe for FakeProbe {
        fn status(&self, _exe: &Path, dir: &Path) -> Option<CliStatus> {
            let key = file_name(dir);
            let (_, org) = self.0.iter().find(|(k, _)| *k == key)?;
            parse_status(&format!(r#"{{"loggedIn":true,"orgId":"{org}"}}"#))
        }
    }

    fn app_with(scopes: &[(&str, &str)], cli: bool) -> tempfile::TempDir {
        let app = tempfile::tempdir().unwrap();
        for (account, org) in scopes {
            let dir = app
                .path()
                .join("claude-code-sessions")
                .join(account)
                .join(org);
            fs::create_dir_all(dir).unwrap();
        }
        if cli {
            for ver in ["2.1.9", "2.1.100"] {
                let dir = app.path().join("claude-code").join(ver).join("h");
                fs::create_dir_all(&dir).unwrap();
                fs::write(dir.join("claude.exe"), "").unwrap();
            }
        }
        app
    }

    #[test]
    fn capitalises_a_key() {
        assert_eq!(capitalised("hotmail"), "Hotmail");
        assert_eq!(capitalised(""), "");
    }

    #[test]
    fn versions_compare_by_number() {
        assert!(version_key("2.1.100") > version_key("2.1.9"));
    }

    #[test]
    fn picks_the_highest_version() {
        let app = app_with(&[], true);
        let exe = find_cli_exe(app.path()).unwrap();
        assert!(exe.to_string_lossy().contains("2.1.100"));
    }

    #[test]
    fn parses_the_status_json() {
        let s = parse_status(r#"{"loggedIn":true,"orgId":"o1","orgName":"X"}"#).unwrap();
        assert!(s.logged_in);
        assert_eq!(s.org_id.as_deref(), Some("o1"));
        assert!(parse_status("not json").is_none());
    }

    #[test]
    fn matches_a_cli_folder_to_the_scope_of_its_org() {
        let app = app_with(&[("a1", "o1"), ("a1", "o2"), ("a2", "o3")], true);
        let root = tempfile::tempdir().unwrap();
        for key in ["hotmail", "second"] {
            fs::create_dir_all(root.path().join(key)).unwrap();
        }
        let probe = FakeProbe(vec![("hotmail", "o2"), ("second", "o3")]);
        let found = discover(Some(app.path()), root.path(), &probe);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].label, "Hotmail");
        assert_eq!(found[0].desktop_account.as_deref(), Some("a1"));
        assert_eq!(found[0].desktop_org.as_deref(), Some("o2"));
        assert_eq!(found[1].desktop_account.as_deref(), Some("a2"));
    }

    #[test]
    fn an_unanswered_folder_is_cli_only() {
        let app = app_with(&[("a1", "o1")], true);
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("lost")).unwrap();
        let found = discover(Some(app.path()), root.path(), &FakeProbe(vec![]));
        assert_eq!(found.len(), 1);
        assert!(found[0].desktop_account.is_none());
    }
}
