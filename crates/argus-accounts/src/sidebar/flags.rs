//! The sync flags Claude Desktop keeps beside the sidebar keys. After a start Desktop pulls the
//! server's copy of `ccd/dframe-store` unless the store says its local copy is pending for the
//! account that is signed in: it drops every pending flag when `ccd-sync-owner` names another
//! account. A write of the sidebar therefore also names the target account and marks the key
//! pending, or Desktop overwrites the write about a second after it starts.

use rusty_leveldb::DB;

use super::store::{LATIN1, UTF16, full_key};
use crate::types::Scope;

const OWNER: &str = "ccd-sync-owner";
const ORG: &str = "ccd-sync-org";
const PENDING: &str = "ccd-sync-pending:ccd/dframe-store";
/// The key whose removal makes the pending copy the one that wins.
pub const PENDING_BASE: &str = "ccd-sync-pending-base:ccd/dframe-store";

/// A plain string as a stored value: Latin-1 when it fits, else UTF-16LE.
#[must_use]
pub fn encode_text(text: &str) -> Vec<u8> {
    if text.is_ascii() {
        let mut out = vec![LATIN1];
        out.extend_from_slice(text.as_bytes());
        return out;
    }
    let mut out = vec![UTF16];
    for unit in text.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

/// Names `target` as the sync owner and org, marks the `dframe-store` copy pending and drops
/// its pending base, as part of the caller's batch of writes.
///
/// # Errors
/// On any write or delete failure.
pub fn put_flags(db: &mut DB, target: &Scope) -> Result<(), String> {
    let values = [
        (OWNER, target.account.as_str()),
        (ORG, target.org.as_str()),
        (PENDING, "1"),
    ];
    for (name, text) in values {
        db.put(&full_key(name), &encode_text(text))
            .map_err(|e| format!("cannot write `{name}`: {e}"))?;
    }
    db.delete(&full_key(PENDING_BASE))
        .map_err(|e| format!("cannot delete `{PENDING_BASE}`: {e}"))
}

/// Reads a flag back as the text it holds, `None` when the key is not in the store.
#[cfg(test)]
pub fn flag_text(db: &mut DB, name: &str) -> Option<String> {
    let raw = db.get(&full_key(name))?;
    let (&enc, body) = raw.split_first()?;
    Some(if enc == UTF16 {
        let units: Vec<u16> = body
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        body.iter().map(|&b| char::from(b)).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_text_is_latin1_and_other_text_is_utf16() {
        assert_eq!(encode_text("ab"), [1, b'a', b'b']);
        assert_eq!(encode_text("é€"), [0, 0xe9, 0, 0xac, 0x20]);
    }
}
