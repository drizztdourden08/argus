//! The Desktop session behind a CLI session id, read only. A hook sees the CLI session id; the
//! Desktop app names the same session `local_<uuid>` and keeps its record under
//! `claude-code-sessions/<account>/<org>/local_<uuid>.json`. Only the two id fields are parsed,
//! so nothing else in a record is ever read into memory.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::types::Account;

/// The two fields of a session record that hold CLI session ids.
#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Ids {
    cli_session_id: Option<String>,
    prior_cli_session_ids: Vec<Value>,
}

/// Every CLI session id a record holds, the current one first. A text that is not a record holds none.
#[must_use]
pub fn claimed_ids(record: &str) -> Vec<String> {
    let ids: Ids = serde_json::from_str(record).unwrap_or_default();
    ids.cli_session_id
        .into_iter()
        .chain(
            ids.prior_cli_session_ids
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned)),
        )
        .filter(|id| !id.is_empty())
        .collect()
}

fn subdirs(parent: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(parent) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

fn is_record(path: &Path) -> bool {
    let named = path
        .file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with("local_"));
    named
        && path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
}

pub(crate) fn record_files(app_data: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for account in subdirs(&app_data.join("claude-code-sessions")) {
        for org in subdirs(&account) {
            let Ok(entries) = fs::read_dir(&org) else {
                continue;
            };
            files.extend(
                entries
                    .filter_map(Result::ok)
                    .map(|e| e.path())
                    .filter(|p| is_record(p)),
            );
        }
    }
    files.sort();
    files
}

/// For each wanted CLI session id some record holds, the Desktop session id (`local_...`) of that
/// record. An id no record holds is left out.
#[must_use]
pub fn local_ids(app_data: &Path, wanted: &[String]) -> HashMap<String, String> {
    let mut found = HashMap::new();
    if wanted.is_empty() {
        return found;
    }
    for path in record_files(app_data) {
        let Some(local) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
            continue;
        };
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for id in claimed_ids(&text) {
            if wanted.contains(&id) {
                found.insert(id, local.clone());
            }
        }
    }
    found
}

/// Where a Desktop session runs: its current CLI session id and its folder.
#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct Seat {
    /// The CLI session id the Desktop app resumes now.
    pub cli_session_id: Option<String>,
    /// The folder the session works in.
    pub cwd: Option<String>,
    /// The owner archived the session in the Desktop app.
    pub is_archived: bool,
}

/// The record of one Desktop session (`local_...`), read only; `None` when no record has that id.
#[must_use]
pub fn seat_of(app_data: &Path, local_id: &str) -> Option<Seat> {
    let path = record_files(app_data)
        .into_iter()
        .find(|p| p.file_stem().is_some_and(|s| s == local_id))?;
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

/// Is the Desktop session (`local_...`) archived? When any record with that id says archived: the
/// Desktop refuses a message to it as soon as the record of the signed-in account does.
#[must_use]
pub fn is_archived(app_data: &Path, local_id: &str) -> bool {
    let flags: Vec<bool> = record_files(app_data)
        .into_iter()
        .filter(|p| p.file_stem().is_some_and(|s| s == local_id))
        .filter_map(|p| serde_json::from_str::<Seat>(&fs::read_to_string(p).ok()?).ok())
        .map(|seat| seat.is_archived)
        .collect();
    flags.iter().any(|archived| *archived)
}

/// The scope folders `(desktopAccount, desktopOrg)` of every record of a Desktop session
/// (`local_...`), in path order. A session with no record has none.
#[must_use]
pub fn scopes_of(app_data: &Path, local_id: &str) -> Vec<(String, String)> {
    record_files(app_data)
        .into_iter()
        .filter(|p| p.file_stem().is_some_and(|s| s == local_id))
        .filter_map(|p| {
            let org = p.parent()?;
            let account = org.parent()?;
            Some((folder_name(account)?, folder_name(org)?))
        })
        .collect()
}

fn folder_name(dir: &Path) -> Option<String> {
    Some(dir.file_name()?.to_string_lossy().into_owned())
}

/// The first account, in config order, whose `desktop_account` and `desktop_org` both equal one
/// of the scopes.
#[must_use]
pub fn account_for_scopes<'a>(
    accounts: &'a [Account],
    scopes: &[(String, String)],
) -> Option<&'a Account> {
    accounts.iter().find(|a| {
        let (Some(acct), Some(org)) = (&a.desktop_account, &a.desktop_org) else {
            return false;
        };
        scopes.iter().any(|(sa, so)| sa == acct && so == org)
    })
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::write_record;

    const CURRENT: &str = "11111111-1111-4111-8111-111111111111";
    const PRIOR: &str = "22222222-2222-4222-8222-222222222222";

    fn record() -> String {
        format!(
            r#"{{"sessionId":"local_aaaa","cliSessionId":"{CURRENT}","priorCliSessionIds":["{PRIOR}",7],"note":"x"}}"#
        )
    }

    #[test]
    fn a_record_holds_its_current_and_its_prior_cli_ids() {
        assert_eq!(claimed_ids(&record()), [CURRENT, PRIOR]);
    }

    #[test]
    fn a_seat_gives_the_cli_id_and_the_folder_of_a_local_id() {
        let dir = tempfile::tempdir().expect("dir");
        let org = dir.path().join("claude-code-sessions/acct/org");
        fs::create_dir_all(&org).expect("org");
        let text = format!(r#"{{"cliSessionId":"{CURRENT}","cwd":"X:/work"}}"#);
        fs::write(org.join("local_bbbb.json"), text).expect("record");
        let seat = seat_of(dir.path(), "local_bbbb").expect("seat");
        assert_eq!(seat.cli_session_id.as_deref(), Some(CURRENT));
        assert_eq!(seat.cwd.as_deref(), Some("X:/work"));
        assert_eq!(seat_of(dir.path(), "local_none"), None);
    }

    #[test]
    fn a_seat_reads_is_archived_true_false_and_missing() {
        let dir = tempfile::tempdir().expect("dir");
        write_record(
            dir.path(),
            ("a", "o"),
            "local_t.json",
            r#"{"isArchived":true}"#,
        );
        write_record(
            dir.path(),
            ("a", "o"),
            "local_f.json",
            r#"{"isArchived":false}"#,
        );
        write_record(dir.path(), ("a", "o"), "local_m.json", "{}");
        assert!(seat_of(dir.path(), "local_t").expect("seat").is_archived);
        assert!(!seat_of(dir.path(), "local_f").expect("seat").is_archived);
        assert!(!seat_of(dir.path(), "local_m").expect("seat").is_archived);
    }

    #[test]
    fn a_local_id_is_archived_when_any_record_says_so() {
        let dir = tempfile::tempdir().expect("dir");
        let archived = r#"{"isArchived":true}"#;
        write_record(dir.path(), ("a1", "o1"), "local_x.json", archived);
        write_record(dir.path(), ("a2", "o2"), "local_x.json", archived);
        write_record(dir.path(), ("a1", "o1"), "local_y.json", archived);
        write_record(
            dir.path(),
            ("a2", "o2"),
            "local_y.json",
            r#"{"isArchived":false}"#,
        );
        let live = r#"{"isArchived":false}"#;
        write_record(dir.path(), ("a1", "o1"), "local_z.json", live);
        write_record(dir.path(), ("a2", "o2"), "local_z.json", live);
        assert!(is_archived(dir.path(), "local_x"));
        assert!(is_archived(dir.path(), "local_y"));
        assert!(!is_archived(dir.path(), "local_z"));
        assert!(!is_archived(dir.path(), "local_none"));
    }

    #[test]
    fn a_text_that_is_not_a_record_holds_nothing() {
        assert_eq!(claimed_ids("not json").len(), 0);
        assert_eq!(claimed_ids("{}").len(), 0);
    }

    #[test]
    fn the_folder_answers_the_local_id_of_a_current_or_prior_id_and_nothing_for_another() {
        let dir = tempfile::tempdir().expect("dir");
        let org = dir
            .path()
            .join("claude-code-sessions")
            .join("acct")
            .join("org");
        fs::create_dir_all(&org).expect("org");
        fs::write(org.join("local_aaaa.json"), record()).expect("record");
        fs::write(org.join("other.json"), record()).expect("other");
        let wanted = [CURRENT.to_owned(), PRIOR.to_owned(), "none".to_owned()];
        let found = local_ids(dir.path(), &wanted);
        assert_eq!(found.get(CURRENT).map(String::as_str), Some("local_aaaa"));
        assert_eq!(found.get(PRIOR).map(String::as_str), Some("local_aaaa"));
        assert_eq!(found.get("none"), None);
    }

    fn scope(account: &str, org: &str) -> (String, String) {
        (account.to_owned(), org.to_owned())
    }

    fn account_of(key: &str, account: &str, org: &str) -> Account {
        crate::fixtures::account_in(key, key, Some(account.to_owned()), Some(org.to_owned()))
    }

    #[test]
    fn the_scopes_of_a_local_id_are_the_folders_of_each_of_its_records() {
        let dir = tempfile::tempdir().expect("dir");
        let base = dir.path().join("claude-code-sessions");
        for (acct, org) in [("a1", "o1"), ("a2", "o2")] {
            let folder = base.join(acct).join(org);
            fs::create_dir_all(&folder).expect("folder");
            fs::write(folder.join("local_cccc.json"), "{}").expect("record");
        }
        let scopes = scopes_of(dir.path(), "local_cccc");
        assert_eq!(scopes, [scope("a1", "o1"), scope("a2", "o2")]);
        assert_eq!(scopes_of(dir.path(), "local_none").len(), 0);
    }

    #[test]
    fn an_account_matches_only_when_both_ids_match_a_scope() {
        let accounts = [account_of("x", "a1", "o9"), account_of("y", "a1", "o1")];
        let found = account_for_scopes(&accounts, &[scope("a1", "o1")]);
        assert_eq!(found.map(|a| a.key.as_str()), Some("y"));
        assert!(account_for_scopes(&accounts, &[scope("a2", "o1")]).is_none());
    }

    #[test]
    fn with_two_scopes_and_one_match_the_matching_account_wins() {
        let accounts = [account_of("x", "a1", "o1"), account_of("y", "a2", "o2")];
        let scopes = [scope("a3", "o3"), scope("a2", "o2")];
        let found = account_for_scopes(&accounts, &scopes);
        assert_eq!(found.map(|a| a.key.as_str()), Some("y"));
    }
}
