//! Lines into a thread's inbox (its hooks hand them over).

use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::sync::atomic::{AtomicU64, Ordering};

use argus_core::clock::{now_iso, now_ms};
use serde_json::json;

use crate::Board;
use crate::json::{array_mut, slot};

static SALT: AtomicU64 = AtomicU64::new(0);

/// One message for an inbox.
pub struct Message<'a> {
    pub id: String,
    pub from: &'a str,
    pub kind: &'a str,
    pub text: &'a str,
    pub reference: Option<&'a str>,
}

/// `m-<epoch ms>-<suffix>`: the id form of inbox messages.
#[must_use]
pub fn message_id(suffix: &str) -> String {
    format!("m-{}-{suffix}", now_ms())
}

/// Append a message to a thread's inbox (created when missing).
pub fn push(board: &mut Board, to: &str, m: &Message) {
    array_mut(slot(board, "inbox"), to).push(json!({
        "id": m.id, "ts": now_iso(), "from": m.from, "kind": m.kind, "text": m.text,
        "ref": m.reference, "deliveredAt": null,
    }));
}

fn base36(mut n: u64) -> String {
    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::new();
    while n > 0 {
        out.push(DIGITS[usize::try_from(n % 36).unwrap_or(0)]);
        n /= 36;
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

fn random_suffix() -> String {
    let salt = SALT.fetch_add(1, Ordering::Relaxed);
    let digits = base36(RandomState::new().hash_one(salt));
    digits.chars().take(4).collect()
}

/// `queue`: one line into a thread's inbox. False for a thread that is gone.
pub fn queue(
    board: &mut Board,
    to: &str,
    from: &str,
    kind: &str,
    text: &str,
    reference: Option<&str>,
) -> bool {
    if to.is_empty() || board["threads"][to].is_null() {
        return false;
    }
    let message = Message {
        id: message_id(&random_suffix()),
        from,
        kind,
        text,
        reference,
    };
    push(board, to, &message);
    true
}
