//! The kv keys that embed a board key. The formats, found where the server and the crates write
//! them (`<key>` stands for the board key or scope; it is the second segment unless noted):
//!
//! - `check-run:<key>:<runId>`, `check-seq:<key>`, `check-link:<key>:<runId>`,
//!   `check-flag:<key>:<runId>`
//! - `launcher-played:<key>:<name>`, `inbox-read:<key>:<thread>`, `run:<key>` (a counter)
//! - `stall-ack:<key>:<kind>:<subject>:<since>`
//! - `<topic>-owner:<key>:<thread>:<since>` (assigned, verify, run, stall, wake-fault)
//! - `changes:cleared:<key>` (third segment)
//! - `worktrees:hygiene|leftovers|made|requests|sent:<key>` (third segment)

use rusqlite::Transaction;

use crate::error::Result;
use crate::map::IdMap;

const SECOND: [&str; 8] = [
    "check-run",
    "check-seq",
    "check-link",
    "check-flag",
    "launcher-played",
    "inbox-read",
    "stall-ack",
    "run",
];

fn key_index(segments: &[&str]) -> Option<usize> {
    let head = *segments.first()?;
    if SECOND.contains(&head) || head.ends_with("-owner") {
        return Some(1);
    }
    match head {
        "changes" if segments.get(1) == Some(&"cleared") => Some(2),
        "worktrees" => Some(2),
        _ => None,
    }
}

/// The kv key with its board key replaced by the one `map` gives; `None` when it has none or the
/// board is not in the map.
pub fn rewrite_key(key: &str, map: &IdMap) -> Option<String> {
    let mut segments: Vec<&str> = key.split(':').collect();
    let at = key_index(&segments)?;
    let to = map.get(*segments.get(at)?)?;
    segments[at] = to.as_str();
    Some(segments.join(":"))
}

/// Renames every kv key of the formats above.
pub fn rewrite(tx: &Transaction, map: &IdMap) -> Result<()> {
    let mut keys: Vec<String> = Vec::new();
    let mut stmt = tx.prepare("SELECT key FROM kv WHERE key LIKE '%:%'")?;
    for row in stmt.query_map([], |r| r.get(0))? {
        keys.push(row?);
    }
    drop(stmt);
    for old in keys {
        if let Some(new) = rewrite_key(&old, map) {
            tx.execute("UPDATE kv SET key = ?1 WHERE key = ?2", [new, old])?;
        }
    }
    Ok(())
}

// test-tag: critical
#[cfg(test)]
mod tests {
    //! One case for every kv format that embeds a board key.
    use super::rewrite_key;
    use crate::map::IdMap;

    fn map() -> IdMap {
        IdMap::from([("alpha".to_owned(), "b1k2m3n".to_owned())])
    }

    #[test]
    fn every_format_swaps_its_board_key() {
        for (old, new) in [
            ("check-run:alpha:r1", "check-run:b1k2m3n:r1"),
            ("check-seq:alpha", "check-seq:b1k2m3n"),
            ("check-link:alpha:r1", "check-link:b1k2m3n:r1"),
            ("check-flag:alpha:r1", "check-flag:b1k2m3n:r1"),
            ("launcher-played:alpha:dev", "launcher-played:b1k2m3n:dev"),
            ("inbox-read:alpha:lead", "inbox-read:b1k2m3n:lead"),
            ("run:alpha", "run:b1k2m3n"),
            (
                "stall-ack:alpha:idle:I-1:99",
                "stall-ack:b1k2m3n:idle:I-1:99",
            ),
            (
                "assigned-owner:alpha:lead:5",
                "assigned-owner:b1k2m3n:lead:5",
            ),
            (
                "wake-fault-owner:alpha:lead:5",
                "wake-fault-owner:b1k2m3n:lead:5",
            ),
            ("changes:cleared:alpha", "changes:cleared:b1k2m3n"),
            ("worktrees:made:alpha", "worktrees:made:b1k2m3n"),
            ("worktrees:sent:alpha", "worktrees:sent:b1k2m3n"),
        ] {
            assert_eq!(rewrite_key(old, &map()).as_deref(), Some(new), "{old}");
        }
    }

    #[test]
    fn other_keys_stay() {
        for key in [
            "instance",
            "changes:prunedTo",
            "worktrees",
            "check-run:other:r1",
            "run:_portfolio",
            "accounts.handover.cycle",
        ] {
            assert_eq!(rewrite_key(key, &map()), None, "{key}");
        }
    }
}
