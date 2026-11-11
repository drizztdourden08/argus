//! The backup a sync run takes before it writes, and the undo that puts it back. A backup folder
//! holds a copy of every file the run will overwrite, the whole `Local Storage\leveldb` folder
//! of the Desktop app (without its `LOCK`) and a `manifest.json` with the `SHA-256` of each copy.
//! Nothing is ever deleted: an undo moves the files the run created into the backup folder, and
//! copies every live file it is about to overwrite into `after-apply` first. After the writes the
//! manifest also holds the `SHA-256` of what the run wrote, so an undo can tell a file the sync
//! left alone from one that changed afterwards (work done since) and keep the latter.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use time::OffsetDateTime;
use time::macros::format_description;

use crate::catalog::merge::sha256_hex;
use crate::catalog::plan::Plan;
use crate::catalog::read::{file_names, write_atomic};
use crate::sidebar::store::ensure_unlocked;
use crate::types::Scope;

const MANIFEST: &str = "manifest.json";
const FILES_DIR: &str = "files";
const LEVELDB_DIR: &str = "leveldb";
const AFTER_DIR: &str = "after-apply";
const LIVE_DIR: &str = "live";
const LIVE_LEVELDB_DIR: &str = "live-leveldb";
const LOCK: &str = "LOCK";

/// One backed-up file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    /// The path the file has in the Desktop app's data.
    pub original: String,
    /// The copy, relative to the backup folder.
    pub copy: String,
    /// `SHA-256` of the file, lower hex.
    pub sha256: String,
}

/// The backed-up `LevelDB` folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelDbEntry {
    /// The live folder.
    pub dir: String,
    /// The copy, relative to the backup folder.
    pub copy: String,
    /// The files copied, with their `SHA-256`.
    pub files: Vec<LevelFile>,
}

/// One file of the `LevelDB` folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelFile {
    /// The file name.
    pub name: String,
    /// `SHA-256` of the file, lower hex.
    pub sha256: String,
}

/// What `manifest.json` holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    /// The backup's UTC time, as in the folder name.
    pub created_at: String,
    /// The chat files and archive indexes that existed and will be overwritten.
    pub files: Vec<FileEntry>,
    /// The files the run creates; an undo moves them aside.
    pub created: Vec<String>,
    /// The sidebar storage, when the app has one.
    pub leveldb: Option<LevelDbEntry>,
    /// The counts of the plan the run follows.
    pub plan: Value,
    /// What the files were right after the run wrote them; absent until the run is written.
    #[serde(default)]
    pub applied: Option<Applied>,
}

/// The `SHA-256` of what a run wrote.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Applied {
    /// Chat files and indexes by their path in the Desktop app's data; a file that was not
    /// there has no entry.
    pub files: BTreeMap<String, String>,
    /// The `LevelDB` files after the sidebar write, without `LOCK`.
    pub leveldb: Vec<LevelFile>,
}

/// What an undo did.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoReport {
    /// Files put back, the `LevelDB` ones included.
    pub restored: usize,
    /// Files the run had created, moved into the backup folder.
    pub moved_aside: usize,
    /// Live files copied into `after-apply` before they were overwritten.
    pub saved: usize,
    /// What was left alone, one line each: changed since the sync.
    pub kept: Vec<String>,
}

/// The UTC time as a folder name, `20261111T153045Z`.
#[must_use]
pub fn utc_stamp(now_ms: i64) -> String {
    let nanos = i128::from(now_ms) * 1_000_000;
    let time =
        OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    time.format(format_description!(
        "[year][month][day]T[hour][minute][second]Z"
    ))
    .unwrap_or_else(|_| "unknown".to_owned())
}

fn io_err(path: &Path, e: &std::io::Error) -> String {
    format!("{}: {e}", path.display())
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| io_err(path, &e))
}

fn session_file(app: &Path, scope: &Scope, file: &str) -> PathBuf {
    app.join("claude-code-sessions")
        .join(&scope.account)
        .join(&scope.org)
        .join(file)
}

/// A fresh folder under `root` named for the time; a second run in the same second gets `-2`.
fn fresh_dir(root: &Path, stamp: &str) -> Result<PathBuf, String> {
    for n in 1..100 {
        let name = if n == 1 {
            stamp.to_owned()
        } else {
            format!("{stamp}-{n}")
        };
        let dir = root.join(name);
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| io_err(&dir, &e))?;
            return Ok(dir);
        }
    }
    Err(format!("no free backup folder under {}", root.display()))
}

/// Copies what the plan will touch, and the `LevelDB` folder, into a new folder under `root`.
///
/// # Errors
/// When a file cannot be read or copied, or a file differs from what the plan read.
pub fn create(root: &Path, app: &Path, plan: &Plan, stamp: &str) -> Result<PathBuf, String> {
    let dir = fresh_dir(root, stamp)?;
    let mut files = Vec::new();
    let mut created = Vec::new();
    for (n, change) in plan.changes.iter().enumerate() {
        let path = session_file(app, &change.scope, &change.file);
        let Some(expected) = &change.before_sha256 else {
            created.push(path.display().to_string());
            continue;
        };
        let bytes = read(&path)?;
        let sha256 = sha256_hex(&bytes);
        if &sha256 != expected {
            return Err(format!(
                "{} changed after the plan was made",
                path.display()
            ));
        }
        let copy = format!("{FILES_DIR}/{n:05}-{}", change.file);
        write_atomic(&dir.join(&copy), &bytes)?;
        files.push(FileEntry {
            original: path.display().to_string(),
            copy,
            sha256,
        });
    }
    let manifest = Manifest {
        created_at: stamp.to_owned(),
        files,
        created,
        leveldb: copy_leveldb(app, &dir)?,
        plan: json!(plan.summary),
        applied: None,
    };
    let text = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    write_atomic(&dir.join(MANIFEST), &text)?;
    Ok(dir)
}

/// Writes into the manifest the `SHA-256` of every file the run wrote and of the `LevelDB` files, so
/// an undo can tell later work from the sync's own writes. Call it after the writes, also when
/// they stopped half way.
///
/// # Errors
/// When the manifest or a written file cannot be read, or the manifest cannot be saved.
pub fn record_applied(dir: &Path, app: &Path, plan: &Plan) -> Result<(), String> {
    let mut manifest = read_manifest(dir)?;
    let mut files = BTreeMap::new();
    for change in &plan.changes {
        let path = session_file(app, &change.scope, &change.file);
        if let Ok(bytes) = fs::read(&path) {
            files.insert(path.display().to_string(), sha256_hex(&bytes));
        }
    }
    let leveldb = match &manifest.leveldb {
        Some(entry) => hash_files(Path::new(&entry.dir))?,
        None => Vec::new(),
    };
    manifest.applied = Some(Applied { files, leveldb });
    let text = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    write_atomic(&dir.join(MANIFEST), &text)
}

/// The files of a folder with their `SHA-256`, without `LOCK`; empty when it is not there.
fn hash_files(dir: &Path) -> Result<Vec<LevelFile>, String> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for name in file_names(dir)? {
        if name != LOCK {
            let sha256 = sha256_hex(&read(&dir.join(&name))?);
            out.push(LevelFile { name, sha256 });
        }
    }
    Ok(out)
}

fn copy_leveldb(app: &Path, backup: &Path) -> Result<Option<LevelDbEntry>, String> {
    let live = app.join("Local Storage").join("leveldb");
    if !live.is_dir() {
        return Ok(None);
    }
    let target = backup.join(LEVELDB_DIR);
    fs::create_dir_all(&target).map_err(|e| io_err(&target, &e))?;
    let mut files = Vec::new();
    for name in file_names(&live)? {
        if name == LOCK {
            continue;
        }
        let bytes = read(&live.join(&name))?;
        write_atomic(&target.join(&name), &bytes)?;
        files.push(LevelFile {
            sha256: sha256_hex(&bytes),
            name,
        });
    }
    Ok(Some(LevelDbEntry {
        dir: live.display().to_string(),
        copy: LEVELDB_DIR.to_owned(),
        files,
    }))
}

/// The newest backup folder under `root` that has a manifest.
#[must_use]
pub fn last(root: &Path) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.join(MANIFEST).is_file())
        .collect();
    dirs.sort_by_key(|p| p.file_name().map(std::ffi::OsStr::to_os_string));
    dirs.pop()
}

/// Reads a backup's manifest.
///
/// # Errors
/// When the manifest is missing or not readable.
pub fn read_manifest(dir: &Path) -> Result<Manifest, String> {
    let path = dir.join(MANIFEST);
    serde_json::from_slice(&read(&path)?).map_err(|e| format!("{}: {e}", path.display()))
}

/// Puts a backup back byte for byte and checks the `SHA-256` of every file after. Before a live
/// file is overwritten it is copied into `after-apply`. Files the run created, and `LevelDB`
/// files the backup does not have, are moved there too; nothing is deleted.
///
/// A file whose `SHA-256` differs from what the run wrote (work happened after the sync) is left
/// alone and listed in the report, unless `force`. When any `LevelDB` file differs, the whole
/// `LevelDB` part is left alone without `force`.
///
/// # Errors
/// When Claude holds the `LevelDB` `LOCK`, a copy no longer matches its manifest entry (nothing
/// is changed then), or a file cannot be restored or fails its check.
pub fn undo(dir: &Path, force: bool) -> Result<UndoReport, String> {
    let manifest = read_manifest(dir)?;
    let live_level = manifest.leveldb.as_ref().map(|l| PathBuf::from(&l.dir));
    if let Some(live) = live_level.as_deref().filter(|d| d.is_dir()) {
        ensure_unlocked(live)?;
    }
    check_copies(dir, &manifest)?;
    let applied = manifest.applied.as_ref();
    let after = dir.join(AFTER_DIR);
    let mut report = UndoReport::default();
    for (n, entry) in manifest.files.iter().enumerate() {
        let original = Path::new(&entry.original);
        if !force && changed_since(applied, original) {
            report.keep(original);
            continue;
        }
        let name = format!("{n:05}-{}", name_of(original));
        report.saved += usize::from(save_live(original, &after.join(LIVE_DIR).join(name))?);
        restore(&dir.join(&entry.copy), original, &entry.sha256)?;
        report.restored += 1;
    }
    for (n, path) in manifest.created.iter().enumerate() {
        let path = Path::new(path);
        if !force && changed_since(applied, path) {
            report.keep(path);
            continue;
        }
        let name = format!("{n:05}-{}", name_of(path));
        let aside = after.join(FILES_DIR).join(name);
        report.moved_aside += usize::from(move_aside(path, &aside)?);
    }
    if let (Some(entry), Some(live)) = (&manifest.leveldb, live_level) {
        undo_leveldb(dir, (entry, applied), &live, force, &mut report)?;
    }
    Ok(report)
}

impl UndoReport {
    fn keep(&mut self, path: &Path) {
        self.kept
            .push(format!("{}: changed since the sync, kept", path.display()));
    }
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |f| f.to_string_lossy().into_owned())
}

/// Whether the file is not what the run wrote; false when no record says what it wrote.
fn changed_since(applied: Option<&Applied>, path: &Path) -> bool {
    let Some(want) = applied.and_then(|a| a.files.get(&path.display().to_string())) else {
        return false;
    };
    fs::read(path).is_ok_and(|bytes| &sha256_hex(&bytes) != want)
}

/// Every copy in the backup still has the `SHA-256` the manifest says.
fn check_copies(dir: &Path, manifest: &Manifest) -> Result<(), String> {
    for entry in &manifest.files {
        verify(&dir.join(&entry.copy), &entry.sha256)?;
    }
    if let Some(level) = &manifest.leveldb {
        for file in &level.files {
            verify(&dir.join(&level.copy).join(&file.name), &file.sha256)?;
        }
    }
    Ok(())
}

fn verify(path: &Path, sha256: &str) -> Result<(), String> {
    if sha256_hex(&read(path)?) == sha256 {
        Ok(())
    } else {
        Err(format!(
            "{} does not match its backup checksum",
            path.display()
        ))
    }
}

fn restore(copy: &Path, original: &Path, sha256: &str) -> Result<(), String> {
    write_atomic(original, &read(copy)?)?;
    verify(original, sha256)
}

fn undo_leveldb(
    dir: &Path,
    (entry, applied): (&LevelDbEntry, Option<&Applied>),
    live: &Path,
    force: bool,
    report: &mut UndoReport,
) -> Result<(), String> {
    fs::create_dir_all(live).map_err(|e| io_err(live, &e))?;
    if !force && applied.is_some_and(|a| hash_files(live).is_ok_and(|now| now != a.leveldb)) {
        report.kept.push(format!(
            "{}: LevelDB changed since the sync, kept",
            live.display()
        ));
        return Ok(());
    }
    let after = dir.join(AFTER_DIR);
    let kept: Vec<&str> = entry.files.iter().map(|f| f.name.as_str()).collect();
    for name in file_names(live)? {
        if name != LOCK && !kept.contains(&name.as_str()) {
            let aside = after.join(LEVELDB_DIR).join(&name);
            report.moved_aside += usize::from(move_aside(&live.join(&name), &aside)?);
        }
    }
    for file in &entry.files {
        let target = live.join(&file.name);
        let saved = after.join(LIVE_LEVELDB_DIR).join(&file.name);
        report.saved += usize::from(save_live(&target, &saved)?);
        restore(
            &dir.join(&entry.copy).join(&file.name),
            &target,
            &file.sha256,
        )?;
        report.restored += 1;
    }
    Ok(())
}

/// A path that does not exist yet: `path`, else `path.2`, `path.3` and so on.
fn free(path: &Path) -> PathBuf {
    let mut candidate = path.to_path_buf();
    let mut n = 1;
    while candidate.exists() {
        n += 1;
        candidate = PathBuf::from(format!("{}.{n}", path.display()));
    }
    candidate
}

/// Copies a live file to `saved`; false when it is not there.
fn save_live(from: &Path, saved: &Path) -> Result<bool, String> {
    if !from.is_file() {
        return Ok(false);
    }
    write_atomic(&free(saved), &read(from)?)?;
    Ok(true)
}

/// Moves a file to `aside`; false when it is not there.
fn move_aside(from: &Path, aside: &Path) -> Result<bool, String> {
    if !from.is_file() {
        return Ok(false);
    }
    let aside = free(aside);
    if let Some(parent) = aside.parent() {
        fs::create_dir_all(parent).map_err(|e| io_err(parent, &e))?;
    }
    if fs::rename(from, &aside).is_err() {
        fs::copy(from, &aside).map_err(|e| io_err(from, &e))?;
        fs::remove_file(from).map_err(|e| io_err(from, &e))?;
    }
    Ok(true)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;
    use crate::catalog::plan::{Change, ChangeKind};

    fn scope() -> Scope {
        Scope {
            account: "a1".into(),
            org: "o1".into(),
        }
    }

    fn change(file: &str, kind: ChangeKind, before: Option<&[u8]>) -> Change {
        Change {
            scope: scope(),
            file: file.to_owned(),
            kind,
            session_id: String::new(),
            title: String::new(),
            before_sha256: before.map(sha256_hex),
            data: json!({}),
        }
    }

    /// An app folder with one chat record, an archive index and a `LevelDB` folder, and a plan
    /// that overwrites the record and the index and creates one more record.
    struct World {
        app: TempDir,
        backups: TempDir,
        plan: Plan,
    }

    impl World {
        fn new() -> Self {
            let world = World {
                app: TempDir::new().unwrap(),
                backups: TempDir::new().unwrap(),
                plan: Plan::default(),
            };
            let sessions = world.sessions();
            fs::create_dir_all(&sessions).unwrap();
            fs::write(sessions.join("local_1.json"), b"record one").unwrap();
            fs::write(sessions.join("archived-sessions.idx"), b"index").unwrap();
            let level = world.level();
            fs::create_dir_all(&level).unwrap();
            for (name, body) in [
                ("CURRENT", "manifest-1"),
                ("000003.ldb", "table"),
                ("LOCK", ""),
            ] {
                fs::write(level.join(name), body).unwrap();
            }
            World {
                plan: Plan {
                    changes: vec![
                        change(
                            "local_1.json",
                            ChangeKind::UpdateSession,
                            Some(b"record one"),
                        ),
                        change(
                            "archived-sessions.idx",
                            ChangeKind::ArchiveIndex,
                            Some(b"index"),
                        ),
                        change("local_2.json", ChangeKind::CopySession, None),
                    ],
                    ..Plan::default()
                },
                ..world
            }
        }

        /// A world after a backup, an apply and the record of what the apply wrote.
        fn applied() -> (World, PathBuf) {
            let w = World::new();
            let dir = w.backup();
            w.apply();
            record_applied(&dir, w.app.path(), &w.plan).unwrap();
            (w, dir)
        }

        fn sessions(&self) -> PathBuf {
            self.app.path().join("claude-code-sessions/a1/o1")
        }

        fn level(&self) -> PathBuf {
            self.app.path().join("Local Storage/leveldb")
        }

        fn backup(&self) -> PathBuf {
            create(
                self.backups.path(),
                self.app.path(),
                &self.plan,
                "20261111T100000Z",
            )
            .unwrap()
        }

        /// What an apply does: rewrite, create and add files.
        fn apply(&self) {
            fs::write(self.sessions().join("local_1.json"), b"record one, synced").unwrap();
            fs::write(self.sessions().join("archived-sessions.idx"), b"new index").unwrap();
            fs::write(self.sessions().join("local_2.json"), b"copied").unwrap();
            fs::write(self.level().join("CURRENT"), b"manifest-2").unwrap();
            fs::write(self.level().join("000004.log"), b"appended").unwrap();
        }
    }

    #[test]
    fn stamp_is_a_folder_name() {
        assert_eq!(utc_stamp(1_700_000_000_000), "20231114T221320Z");
    }

    #[test]
    fn a_backup_holds_the_touched_files_and_the_leveldb_without_its_lock() {
        let w = World::new();
        let dir = w.backup();
        let manifest = read_manifest(&dir).unwrap();
        assert_eq!(manifest.files.len(), 2);
        assert_eq!(manifest.created.len(), 1);
        let level = manifest.leveldb.unwrap();
        let names: Vec<&str> = level.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["000003.ldb", "CURRENT"]);
        assert!(!dir.join("leveldb/LOCK").exists());
        assert_eq!(manifest.files[0].sha256, sha256_hex(b"record one"));
    }

    #[test]
    fn undo_puts_every_byte_back_and_deletes_nothing() {
        let (w, dir) = World::applied();
        let report = undo(&dir, false).unwrap();
        assert_eq!(report.restored, 4);
        assert_eq!(report.moved_aside, 2);
        assert_eq!(report.saved, 4);
        assert!(report.kept.is_empty());
        let read_back = |p: PathBuf| fs::read(p).unwrap();
        assert_eq!(read_back(w.sessions().join("local_1.json")), b"record one");
        assert_eq!(
            read_back(w.sessions().join("archived-sessions.idx")),
            b"index"
        );
        assert_eq!(read_back(w.level().join("CURRENT")), b"manifest-1");
        assert!(!w.sessions().join("local_2.json").exists());
        assert!(!w.level().join("000004.log").exists());
        let aside = dir.join("after-apply");
        assert_eq!(read_back(aside.join("files/00000-local_2.json")), b"copied");
        assert_eq!(read_back(aside.join("leveldb/000004.log")), b"appended");
        let live = aside.join("live");
        assert_eq!(
            read_back(live.join("00000-local_1.json")),
            b"record one, synced"
        );
        assert_eq!(read_back(aside.join("live-leveldb/CURRENT")), b"manifest-2");
    }

    #[test]
    fn work_after_the_sync_is_kept_unless_forced() {
        let (w, dir) = World::applied();
        fs::write(w.sessions().join("local_1.json"), b"worked on since").unwrap();
        fs::write(w.sessions().join("local_2.json"), b"edited copy").unwrap();
        let report = undo(&dir, false).unwrap();
        assert_eq!(report.kept.len(), 2);
        assert!(report.kept[0].ends_with("changed since the sync, kept"));
        assert_eq!(report.restored, 3);
        assert_eq!(report.moved_aside, 1);
        let now = fs::read(w.sessions().join("local_1.json")).unwrap();
        assert_eq!(now, b"worked on since");
        assert!(w.sessions().join("local_2.json").exists());
        let forced = undo(&dir, true).unwrap();
        assert!(forced.kept.is_empty());
        let back = fs::read(w.sessions().join("local_1.json")).unwrap();
        assert_eq!(back, b"record one");
        let saved = dir.join("after-apply/live/00000-local_1.json");
        assert_eq!(fs::read(saved).unwrap(), b"worked on since");
    }

    #[test]
    fn a_changed_leveldb_is_refused_without_force() {
        let (w, dir) = World::applied();
        fs::write(w.level().join("000004.log"), b"claude wrote more").unwrap();
        let report = undo(&dir, false).unwrap();
        assert_eq!(report.kept.len(), 1);
        assert!(report.kept[0].contains("LevelDB"));
        let current = fs::read(w.level().join("CURRENT")).unwrap();
        assert_eq!(current, b"manifest-2");
        assert_eq!(report.restored, 2);
        let forced = undo(&dir, true).unwrap();
        assert!(forced.kept.is_empty());
        let current = fs::read(w.level().join("CURRENT")).unwrap();
        assert_eq!(current, b"manifest-1");
    }

    #[test]
    fn a_damaged_backup_changes_nothing() {
        let w = World::new();
        let dir = w.backup();
        w.apply();
        fs::write(dir.join("leveldb/000003.ldb"), b"rotten").unwrap();
        assert!(undo(&dir, false).is_err());
        let now = fs::read(w.sessions().join("local_1.json")).unwrap();
        assert_eq!(now, b"record one, synced");
    }

    #[test]
    fn a_changed_file_stops_the_backup() {
        let w = World::new();
        fs::write(w.sessions().join("local_1.json"), b"edited meanwhile").unwrap();
        let made = create(w.backups.path(), w.app.path(), &w.plan, "20261111T100000Z");
        assert!(made.is_err());
    }

    #[test]
    fn backups_in_one_second_get_their_own_folder_and_the_last_is_the_newest() {
        let w = World::new();
        let first = w.backup();
        let second = w.backup();
        assert_ne!(first, second);
        assert!(second.ends_with("20261111T100000Z-2"));
        assert_eq!(last(w.backups.path()), Some(second));
    }
}
