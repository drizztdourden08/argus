//! The transcript files of the hosted Overlord: `<home>/overlord/transcript-<id>.jsonl`, one
//! appended write per line, no fsync (a transcript lost to a power cut is not worth the wait).

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use argus_overlord::ports::{TranscriptFault, TranscriptStore};
use argus_overlord::transcript::{Page, Stamp, file_name, lines_after, session_of};

/// The transcript folder, as the `TranscriptStore` port.
pub struct FileTranscripts {
    dir: PathBuf,
}

fn fault(what: &str) -> TranscriptFault {
    TranscriptFault(what.to_owned())
}

impl FileTranscripts {
    /// The store under the dashboard home.
    #[must_use]
    pub fn new(home: &Path) -> Self {
        Self {
            dir: home.join("overlord"),
        }
    }

    fn path(&self, session: &str) -> Result<PathBuf, TranscriptFault> {
        let name = file_name(session).ok_or_else(|| fault("the session id is not usable"))?;
        Ok(self.dir.join(name))
    }
}

impl TranscriptStore for FileTranscripts {
    fn append(&self, session: &str, line: &str) -> Result<(), TranscriptFault> {
        let path = self.path(session)?;
        fs::create_dir_all(&self.dir).map_err(|_| fault("the transcript folder cannot be made"))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| fault("the transcript cannot be opened"))?;
        file.write_all(format!("{line}\n").as_bytes())
            .map_err(|_| fault("the transcript cannot be written"))
    }

    fn read_after(&self, session: &str, after: usize) -> Result<Page, TranscriptFault> {
        match fs::read(self.path(session)?) {
            Ok(bytes) => Ok(lines_after(&String::from_utf8_lossy(&bytes), after)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(lines_after("", after)),
            Err(_) => Err(fault("the transcript cannot be read")),
        }
    }

    fn list(&self) -> Result<Vec<Stamp>, TranscriptFault> {
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err(fault("the transcript folder cannot be read")),
        };
        Ok(entries.filter_map(Result::ok).filter_map(stamp).collect())
    }

    fn delete(&self, session: &str) -> Result<(), TranscriptFault> {
        match fs::remove_file(self.path(session)?) {
            Err(e) if e.kind() != ErrorKind::NotFound => {
                Err(fault("the transcript cannot be removed"))
            }
            _ => Ok(()),
        }
    }
}

fn stamp(entry: fs::DirEntry) -> Option<Stamp> {
    let session = session_of(entry.file_name().to_str()?)?;
    let modified = entry.metadata().ok()?.modified().ok()?;
    let ms = modified.duration_since(UNIX_EPOCH).ok()?.as_millis();
    Some(Stamp {
        session,
        modified_ms: u64::try_from(ms).ok()?,
    })
}

#[cfg(test)]
mod tests {
    use argus_overlord::ports::ApiKey;
    use argus_overlord::transcript::Transcript;

    use super::*;

    const KEY: &str = "sk-ant-api03-SECRETSECRETSECRET-abcd";

    // test-tag: critical
    #[test]
    fn a_fresh_store_reads_what_an_earlier_one_appended() {
        let home = tempfile::tempdir().unwrap();
        let first = Transcript::new(FileTranscripts::new(home.path()), &ApiKey::new(KEY));
        first.record("s1", r#"{"n":1}"#).unwrap();
        first.record("s1", &format!(r#"{{"n":2,"text":"{KEY}"}}"#)).unwrap();
        drop(first);
        let second = Transcript::new(FileTranscripts::new(home.path()), &ApiKey::new(KEY));
        let all = second.read_after("s1", 0).unwrap();
        assert_eq!(all.next, 2);
        let rest = second.read_after("s1", 1).unwrap();
        assert_eq!(rest.lines.len(), 1);
        assert_eq!(rest.next, 2);
        let on_disk = fs::read_to_string(home.path().join("overlord/transcript-s1.jsonl")).unwrap();
        assert!(!on_disk.contains("SECRETSECRET"));
    }

    #[test]
    fn listing_and_deleting_see_only_transcript_files() {
        let home = tempfile::tempdir().unwrap();
        let store = FileTranscripts::new(home.path());
        assert_eq!(store.list().unwrap().len(), 0);
        store.append("s1", "{}").unwrap();
        fs::write(home.path().join("overlord/state.json"), "{}").unwrap();
        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].session, "s1");
        store.delete("s1").unwrap();
        store.delete("s1").unwrap();
        assert_eq!(store.list().unwrap().len(), 0);
        assert!(store.append("../x", "{}").is_err());
    }
}
