//! Incremental reading of a transcript: the byte offset is kept, each poll reads only the new
//! bytes (bounded per poll), a partial last line waits for its newline.

use super::tracked::Tracked;
use crate::config::WatcherOptions;
use crate::fs::FileSystem;

/// What a poll of one transcript found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Tail {
    Missing,
    Same,
    Read,
}

/// Splits the carried bytes and a new chunk into whole lines; the rest waits for its newline.
fn feed(tracked: &mut Tracked, chunk: &[u8], opts: &WatcherOptions, now_ms: i64) {
    let mut buf = std::mem::take(&mut tracked.cursor.carry);
    buf.extend_from_slice(chunk);
    let mut from = 0;
    while let Some(rel) = buf[from..].iter().position(|b| *b == b'\n') {
        let end = from + rel;
        if tracked.cursor.skipping {
            tracked.cursor.skipping = false;
        } else if end > from {
            let line = String::from_utf8_lossy(&buf[from..end]);
            tracked.apply_line(&line, now_ms, opts.text_max);
        }
        from = end + 1;
    }
    let rest = &buf[from..];
    if !tracked.cursor.skipping && rest.len() > opts.max_line_bytes {
        tracked.cursor.skipping = true;
    }
    if !tracked.cursor.skipping {
        tracked.cursor.carry = rest.to_vec();
    }
}

fn read_new(
    tracked: &mut Tracked,
    fs: &dyn FileSystem,
    size: u64,
    opts: &WatcherOptions,
    now_ms: i64,
) -> Tail {
    let mut budget = opts.max_bytes_per_tick;
    while tracked.cursor.offset < size && budget > 0 {
        let left = usize::try_from(size - tracked.cursor.offset).unwrap_or(usize::MAX);
        let want = opts.chunk_bytes.min(left);
        let chunk = match fs.read_at(&tracked.file, tracked.cursor.offset, want) {
            Ok(chunk) => chunk,
            Err(err) => {
                tracing::warn!(file = %tracked.file.display(), %err, "sub-agent transcript unreadable");
                return Tail::Same;
            }
        };
        if chunk.is_empty() {
            break;
        }
        tracked.cursor.offset += chunk.len() as u64;
        budget = budget.saturating_sub(chunk.len());
        feed(tracked, &chunk, opts, now_ms);
    }
    Tail::Read
}

/// Reads what the transcript gained since the last poll.
pub(super) fn tail(
    tracked: &mut Tracked,
    fs: &dyn FileSystem,
    opts: &WatcherOptions,
    now_ms: i64,
) -> Tail {
    let Some(stat) = fs.stat(&tracked.file) else {
        return Tail::Missing;
    };
    if stat.size < tracked.cursor.offset {
        tracked.restart();
    }
    if stat.size == tracked.cursor.offset {
        return Tail::Same;
    }
    read_new(tracked, fs, stat.size, opts, now_ms)
}
