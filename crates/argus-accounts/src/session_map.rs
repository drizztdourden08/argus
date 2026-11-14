//! The Desktop session behind a CLI session id, read only. A hook sees the CLI session id; the
//! Desktop app names the same session `local_<uuid>` and keeps its record under
//! `claude-code-sessions/<account>/<org>/local_<uuid>.json`. Only the two id fields are parsed,
//! so nothing else in a record is ever read into memory.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

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
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .is_some_and(|n| n.starts_with("local_") && n.ends_with(".json"))
}

fn record_files(app_data: &Path) -> Vec<PathBuf> {
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

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

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
}
