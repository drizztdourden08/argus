//! What the services read from disk, as a small trait with the real file system behind it.

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::SystemTime;

/// The facts about a file the services use.
#[derive(Debug, Clone, Copy)]
pub struct FileStat {
    /// Bytes in the file.
    pub size: u64,
    /// Last write, epoch milliseconds.
    pub mtime_ms: i64,
    /// Creation, epoch milliseconds; the last write when the system keeps no creation time.
    pub birth_ms: i64,
    /// Whether it is a folder.
    pub is_dir: bool,
}

/// One entry of a folder.
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// The file or folder name.
    pub name: String,
    /// Whether it is a folder.
    pub is_dir: bool,
}

/// Read-only access to transcripts, metas and task outputs.
pub trait FileSystem: Send + Sync {
    /// The facts about a file or folder, `None` when it is not there.
    fn stat(&self, path: &Path) -> Option<FileStat>;
    /// Up to `len` bytes from `offset`.
    ///
    /// # Errors
    /// The file cannot be opened or read.
    fn read_at(&self, path: &Path, offset: u64, len: usize) -> io::Result<Vec<u8>>;
    /// The whole file as text, `None` when it cannot be read.
    fn read_text(&self, path: &Path) -> Option<String>;
    /// The entries of a folder, by name.
    ///
    /// # Errors
    /// The folder cannot be read.
    fn list(&self, dir: &Path) -> io::Result<Vec<DirEntry>>;
}

/// The real file system.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdFs;

fn millis(time: io::Result<SystemTime>) -> Option<i64> {
    let since = time.ok()?.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    i64::try_from(since.as_millis()).ok()
}

impl FileSystem for StdFs {
    fn stat(&self, path: &Path) -> Option<FileStat> {
        let meta = fs::metadata(path).ok()?;
        let mtime_ms = millis(meta.modified())?;
        Some(FileStat {
            size: meta.len(),
            mtime_ms,
            birth_ms: millis(meta.created()).unwrap_or(mtime_ms),
            is_dir: meta.is_dir(),
        })
    }

    fn read_at(&self, path: &Path, offset: u64, len: usize) -> io::Result<Vec<u8>> {
        let mut file = File::open(path)?;
        file.seek(SeekFrom::Start(offset))?;
        let mut out = Vec::with_capacity(len);
        file.take(len as u64).read_to_end(&mut out)?;
        Ok(out)
    }

    fn read_text(&self, path: &Path) -> Option<String> {
        fs::read_to_string(path).ok()
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<DirEntry>> {
        let mut out = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let is_dir = entry.file_type().is_ok_and(|t| t.is_dir());
            out.push(DirEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir,
            });
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }
}
