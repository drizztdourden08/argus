//! `manifest.json` of a backup: path, size and sha256 of every file.

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// One file of the backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The path under the backup, with `/` separators.
    pub path: String,
    /// The size in bytes.
    pub size: u64,
    /// The sha256 as lowercase hex.
    pub sha256: String,
}

/// The manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// When the backup was made.
    pub created: String,
    /// The home it was made from.
    pub source: String,
    /// Every file.
    pub files: Vec<Entry>,
}

/// The name of the manifest file in a backup.
pub const NAME: &str = "manifest.json";

struct HashingWriter<W: Write> {
    inner: W,
    hash: Sha256,
    size: u64,
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.hash.update(&buf[..n]);
        self.size += u64::try_from(n).unwrap_or(0);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Size and sha256 of a file.
pub fn hash_file(path: &Path) -> io::Result<(u64, String)> {
    let mut sink = HashingWriter {
        inner: io::sink(),
        hash: Sha256::new(),
        size: 0,
    };
    io::copy(&mut File::open(path)?, &mut sink)?;
    Ok((sink.size, hex(&sink.hash.finalize())))
}

/// Copies `from` to `to` (flushed) and returns the size and sha256 of what was written.
pub fn copy_hashed(from: &Path, to: &Path) -> io::Result<(u64, String)> {
    let mut sink = HashingWriter {
        inner: File::create(to)?,
        hash: Sha256::new(),
        size: 0,
    };
    let mut source = File::open(from)?;
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = source.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sink.write_all(&buf[..n])?;
    }
    sink.inner.sync_all()?;
    Ok((sink.size, hex(&sink.hash.finalize())))
}
