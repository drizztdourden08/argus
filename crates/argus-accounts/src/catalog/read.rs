//! The real catalog: the Desktop app's `claude-code-sessions` folder and the shared Claude Code
//! transcripts, read and written as files. Writes are atomic: a temp file in the same folder,
//! then a rename over the target.

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

use serde_json::{Value, json};

use super::merge::{json_bytes, sha256_hex};
use crate::ports::{Catalog, Transcripts};
use crate::types::Scope;

const INDEX_FILE: &str = "archived-sessions.idx";
const TOMBSTONE_PREFIX: &str = "deleted_";

/// The chat records of every scope under `<app>\claude-code-sessions`.
#[derive(Debug, Clone)]
pub struct FsCatalog {
    root: PathBuf,
}

impl FsCatalog {
    /// A catalog over the `claude-code-sessions` folder itself.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        FsCatalog { root: root.into() }
    }

    /// A catalog over the sessions folder of the Desktop app's data folder.
    #[must_use]
    pub fn for_app(app_data: &Path) -> Self {
        Self::new(app_data.join("claude-code-sessions"))
    }

    fn dir(&self, scope: &Scope) -> Result<PathBuf, String> {
        if !is_name(&scope.account) || !is_name(&scope.org) {
            return Err(format!("not a scope name: {}", scope.name()));
        }
        Ok(self.root.join(&scope.account).join(&scope.org))
    }

    fn file(&self, scope: &Scope, name: &str) -> Result<PathBuf, String> {
        if !is_name(name) {
            return Err(format!("not a file name: {name}"));
        }
        Ok(self.dir(scope)?.join(name))
    }

    fn session_file(&self, scope: &Scope, id: &str) -> Result<PathBuf, String> {
        if !is_session_id(id) {
            return Err(format!("not a chat id: {id}"));
        }
        self.file(scope, &format!("{id}.json"))
    }
}

/// A single path component: no separators, no dots at the start.
fn is_name(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && !name.contains(['/', '\\', ':', '\0'])
}

/// `local_` and a uuid.
#[must_use]
pub fn is_session_id(id: &str) -> bool {
    id.strip_prefix("local_").is_some_and(is_uuid)
}

/// The tombstone file name of a chat: `deleted_<uuid>`, without the `local_` prefix.
#[must_use]
pub fn tombstone_name(id: &str) -> String {
    format!(
        "{TOMBSTONE_PREFIX}{}",
        id.strip_prefix("local_").unwrap_or(id)
    )
}

/// 36 characters of hex digits and dashes.
#[must_use]
pub fn is_uuid(text: &str) -> bool {
    text.len() == 36 && text.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

fn io_err(what: &Path, e: &std::io::Error) -> String {
    format!("{}: {e}", what.display())
}

fn read_dirs(path: &Path) -> Result<Vec<fs::DirEntry>, String> {
    match fs::read_dir(path) {
        Ok(entries) => Ok(entries.filter_map(Result::ok).collect()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(io_err(path, &e)),
    }
}

fn sub_dirs(path: &Path) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = read_dirs(path)?
        .iter()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    Ok(names)
}

pub(crate) fn file_names(path: &Path) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = read_dirs(path)?
        .iter()
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    Ok(names)
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Writes `bytes` to a temp file next to `path`, then renames it over `path`.
///
/// # Errors
/// When the folder or file cannot be written.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = path
        .parent()
        .ok_or_else(|| format!("no folder: {}", path.display()))?;
    fs::create_dir_all(dir).map_err(|e| io_err(dir, &e))?;
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp = dir.join(format!(".sync-{}-{n}.tmp", std::process::id()));
    let written = fs::File::create(&temp).and_then(|mut f| {
        f.write_all(bytes)?;
        f.sync_all()
    });
    if let Err(e) = written {
        let _ = fs::remove_file(&temp);
        return Err(io_err(&temp, &e));
    }
    fs::rename(&temp, path).map_err(|e| {
        let _ = fs::remove_file(&temp);
        io_err(path, &e)
    })
}

impl Catalog for FsCatalog {
    fn scopes(&self) -> Result<Vec<Scope>, String> {
        let mut scopes = Vec::new();
        for account in sub_dirs(&self.root)? {
            for org in sub_dirs(&self.root.join(&account))? {
                scopes.push(Scope {
                    account: account.clone(),
                    org,
                });
            }
        }
        Ok(scopes)
    }

    fn session_ids(&self, scope: &Scope) -> Result<Vec<String>, String> {
        Ok(file_names(&self.dir(scope)?)?
            .iter()
            .filter_map(|n| n.strip_suffix(".json"))
            .filter(|id| is_session_id(id))
            .map(str::to_owned)
            .collect())
    }

    fn read_session(&self, scope: &Scope, id: &str) -> Result<Value, String> {
        let path = self.session_file(scope, id)?;
        let bytes = fs::read(&path).map_err(|e| io_err(&path, &e))?;
        serde_json::from_slice(&bytes).map_err(|e| io_err(&path, &e.into()))
    }

    fn write_session(&self, scope: &Scope, id: &str, record: &Value) -> Result<(), String> {
        write_atomic(&self.session_file(scope, id)?, &json_bytes(record))
    }

    fn archived(&self, scope: &Scope) -> Result<Vec<String>, String> {
        let path = self.file(scope, INDEX_FILE)?;
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(io_err(&path, &e)),
        };
        let value: Value = serde_json::from_slice(&bytes).map_err(|e| io_err(&path, &e.into()))?;
        let ids = value
            .get("archived")
            .and_then(Value::as_array)
            .filter(|_| value.get("v") == Some(&json!(1)))
            .and_then(|a| a.iter().map(|s| s.as_str().map(str::to_owned)).collect());
        ids.ok_or_else(|| format!("{}: unrecognized archive index format", path.display()))
    }

    fn write_archived(&self, scope: &Scope, ids: &[String]) -> Result<(), String> {
        let path = self.file(scope, INDEX_FILE)?;
        // Every other field of the app's index stays as it is.
        let mut index = fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .filter(Value::is_object)
            .unwrap_or_else(|| json!({}));
        index["v"] = json!(1);
        index["archived"] = json!(ids);
        write_atomic(&path, &json_bytes(&index))
    }

    fn session_mtime_ms(&self, scope: &Scope, id: &str) -> Result<i64, String> {
        let path = self.session_file(scope, id)?;
        let modified = fs::metadata(&path)
            .and_then(|m| m.modified())
            .map_err(|e| io_err(&path, &e))?;
        let since = modified.duration_since(UNIX_EPOCH).unwrap_or_default();
        Ok(i64::try_from(since.as_millis()).unwrap_or(i64::MAX))
    }

    fn tombstones(&self, scope: &Scope) -> Result<Vec<String>, String> {
        Ok(file_names(&self.dir(scope)?)?
            .iter()
            .filter_map(|n| n.strip_prefix(TOMBSTONE_PREFIX))
            .map(|rest| format!("local_{}", rest.strip_suffix(".json").unwrap_or(rest)))
            .collect())
    }

    fn delete_session(&self, scope: &Scope, id: &str, at_ms: i64) -> Result<(), String> {
        let record = self.session_file(scope, id)?;
        let tombstone = self.file(scope, &tombstone_name(id))?;
        write_atomic(&tombstone, at_ms.to_string().as_bytes())?;
        fs::remove_file(&record).map_err(|e| io_err(&record, &e))
    }

    fn file_sha256(&self, scope: &Scope, file: &str) -> Result<Option<String>, String> {
        let path = self.file(scope, file)?;
        match fs::read(&path) {
            Ok(bytes) => Ok(Some(sha256_hex(&bytes))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io_err(&path, &e)),
        }
    }
}

/// The transcripts under `~\.claude\projects`, indexed by session id when it is built.
#[derive(Debug, Clone, Default)]
pub struct FsTranscripts {
    index: HashMap<String, Vec<PathBuf>>,
}

impl FsTranscripts {
    /// Scans `<projects>\<project>\<id>.jsonl`. A missing folder gives an empty index.
    ///
    /// # Errors
    /// When the folder exists and cannot be read.
    pub fn scan(projects: &Path) -> Result<Self, String> {
        let mut index: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for project in sub_dirs(projects)? {
            let dir = projects.join(project);
            for name in file_names(&dir)? {
                if let Some(id) = name.strip_suffix(".jsonl") {
                    index
                        .entry(id.to_owned())
                        .or_default()
                        .push(dir.join(&name));
                }
            }
        }
        Ok(FsTranscripts { index })
    }
}

impl Transcripts for FsTranscripts {
    fn find(&self, cli_session_id: &str) -> Vec<PathBuf> {
        self.index.get(cli_session_id).cloned().unwrap_or_default()
    }

    fn is_valid(&self, path: &Path) -> bool {
        valid_transcript(path)
    }
}

/// Accepts the messages of the chat itself, and a connected inherited prefix from a fork. A
/// foreign message after the chat's own, or a copied file that never links up, is ambiguous.
#[must_use]
pub fn valid_transcript(path: &Path) -> bool {
    let Ok(file) = fs::File::open(path) else {
        return false;
    };
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let mut found_own = false;
    let mut inherited: Vec<String> = Vec::new();
    let mut linked = false;
    for line in BufReader::new(file).lines() {
        let Ok(line) = line else { return false };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(entry) = serde_json::from_str::<Value>(&line) else {
            return false;
        };
        let Some(object) = entry.as_object() else {
            return false;
        };
        let kind = object.get("type").and_then(Value::as_str);
        if !matches!(kind, Some("user" | "assistant"))
            || !object.get("message").is_some_and(Value::is_object)
        {
            continue;
        }
        let session = object.get("sessionId").and_then(Value::as_str);
        if session == Some(stem) {
            found_own = true;
            let parent = object.get("parentUuid").and_then(Value::as_str);
            linked |= parent.is_some_and(|p| inherited.iter().any(|u| u == p));
            continue;
        }
        let uuid = object.get("uuid").and_then(Value::as_str);
        match (session, uuid) {
            (Some(s), Some(u)) if !found_own && is_uuid(s) && is_uuid(u) => {
                inherited.push(u.to_owned());
            }
            _ => return false,
        }
    }
    found_own && (inherited.is_empty() || linked)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::json;
    use tempfile::tempdir;

    use super::{FsCatalog, FsTranscripts, valid_transcript};
    use crate::ports::{Catalog, Transcripts};
    use crate::types::Scope;

    const SID: &str = "local_11111111-1111-1111-1111-111111111111";

    fn scope() -> Scope {
        Scope {
            account: "acct".to_owned(),
            org: "org".to_owned(),
        }
    }

    /// A temp root whose `acct/org` folder holds one file.
    fn root_with(name: &str, body: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().unwrap();
        let folder = dir.path().join("acct").join("org");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join(name), body).unwrap();
        (dir, folder)
    }

    #[test]
    fn writes_and_reads_a_record_and_the_index() {
        let dir = tempdir().unwrap();
        let catalog = FsCatalog::new(dir.path());
        let record = json!({ "sessionId": SID, "title": "t" });
        catalog.write_session(&scope(), SID, &record).unwrap();
        assert_eq!(catalog.read_session(&scope(), SID).unwrap(), record);
        assert_eq!(catalog.session_ids(&scope()).unwrap(), vec![SID.to_owned()]);
        assert_eq!(catalog.scopes().unwrap(), vec![scope()]);
        assert!(catalog.archived(&scope()).unwrap().is_empty());
        catalog.write_archived(&scope(), &[SID.to_owned()]).unwrap();
        assert_eq!(catalog.archived(&scope()).unwrap(), vec![SID.to_owned()]);
        assert!(
            catalog
                .file_sha256(&scope(), "archived-sessions.idx")
                .unwrap()
                .is_some()
        );
        assert!(
            catalog
                .file_sha256(&scope(), "local_x.json")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn writing_the_index_keeps_its_other_fields() {
        let (dir, folder) = root_with(
            "archived-sessions.idx",
            br#"{"v":1,"archived":[],"other":{"x":1}}"#,
        );
        let catalog = FsCatalog::new(dir.path());
        catalog.write_archived(&scope(), &[SID.to_owned()]).unwrap();
        let now: serde_json::Value =
            serde_json::from_slice(&fs::read(folder.join("archived-sessions.idx")).unwrap())
                .unwrap();
        assert_eq!(now["other"]["x"], 1);
        assert_eq!(now["archived"], json!([SID]));
    }

    #[test]
    fn tombstones_are_listed_as_chat_ids() {
        let (dir, _folder) = root_with("deleted_22222222-2222-2222-2222-222222222222", b"");
        let catalog = FsCatalog::new(dir.path());
        assert_eq!(
            catalog.tombstones(&scope()).unwrap(),
            vec!["local_22222222-2222-2222-2222-222222222222".to_owned()]
        );
    }

    #[test]
    fn refuses_names_that_leave_the_folder() {
        let dir = tempdir().unwrap();
        let catalog = FsCatalog::new(dir.path());
        assert!(catalog.read_session(&scope(), "../local_x").is_err());
        assert!(catalog.file_sha256(&scope(), "..\\evil").is_err());
    }

    fn write_lines(
        dir: &std::path::Path,
        name: &str,
        lines: &[serde_json::Value],
    ) -> std::path::PathBuf {
        fs::create_dir_all(dir).unwrap();
        let path = dir.join(name);
        let text: Vec<String> = lines.iter().map(ToString::to_string).collect();
        fs::write(&path, text.join("\n")).unwrap();
        path
    }

    fn msg(session: &str, uuid: &str, parent: Option<&str>) -> serde_json::Value {
        json!({ "type": "user", "message": { "content": "hi" },
            "sessionId": session, "uuid": uuid, "parentUuid": parent })
    }

    #[test]
    fn transcript_validity() {
        let dir = tempdir().unwrap();
        let own = "33333333-3333-3333-3333-333333333333";
        let other = "44444444-4444-4444-4444-444444444444";
        let u1 = "55555555-5555-5555-5555-555555555555";
        let u2 = "66666666-6666-6666-6666-666666666666";
        let good = write_lines(dir.path(), &format!("{own}.jsonl"), &[msg(own, u1, None)]);
        assert!(valid_transcript(&good));
        let fork = write_lines(
            &dir.path().join("p"),
            &format!("{own}.jsonl"),
            &[msg(other, u1, None), msg(own, u2, Some(u1))],
        );
        assert!(
            valid_transcript(&fork),
            "a connected inherited prefix is fine"
        );
        let unlinked = write_lines(
            &dir.path().join("q"),
            &format!("{own}.jsonl"),
            &[msg(other, u1, None), msg(own, u2, None)],
        );
        assert!(!valid_transcript(&unlinked));
        let foreign_suffix = write_lines(
            dir.path(),
            &format!("{other}.jsonl"),
            &[msg(other, u1, None), msg(own, u2, None)],
        );
        assert!(!valid_transcript(&foreign_suffix));
        let empty = write_lines(dir.path(), "e.jsonl", &[]);
        assert!(!valid_transcript(&empty));
        fs::write(dir.path().join("bad.jsonl"), b"not json\n").unwrap();
        assert!(!valid_transcript(&dir.path().join("bad.jsonl")));
    }

    #[test]
    fn indexes_transcripts_by_id() {
        let dir = tempdir().unwrap();
        let project = dir.path().join("p1");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("abc.jsonl"), b"").unwrap();
        let index = FsTranscripts::scan(dir.path()).unwrap();
        assert_eq!(index.find("abc").len(), 1);
        assert!(index.find("zzz").is_empty());
    }
}
