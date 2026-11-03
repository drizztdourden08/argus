//! Launchers (what the owner can start) and bundles, as far as the page export and the cards
//! need them: which launcher is master, joined or standalone, its usage window, and a bundle's
//! status.

use argus_core::clock::{iso_from_ms, parse_ms};
use serde_json::Value;

use crate::Board;
use crate::json::{section, text, truthy};

/// The launcher every project is expected to have.
pub const MASTER: &str = "master";
const DEF_PREFIX: &str = "launcher-";
const EVERY_LEGACY: &str = "7d";
const EVERY_MIN_MS: i64 = 3_600_000;
const EVERY_MAX_MS: i64 = 365 * 86_400_000;

/// The name of the run definition behind a launcher's Play button.
#[must_use]
pub fn launcher_def_name(name: &str) -> String {
    format!("{DEF_PREFIX}{name}")
}

fn is_live(rec: &Value) -> bool {
    rec.is_object() && !truthy(rec, "retiredAt")
}

/// Whether the board has a live master build.
#[must_use]
pub fn has_master(board: &Board) -> bool {
    crate::served::master_name(board).is_some()
}

fn linked_bundles(board: &Board, name: &str) -> Vec<String> {
    section(board, "builds")
        .iter()
        .filter(|(_, b)| b["launcher"].as_str() == Some(name))
        .map(|(tag, _)| tag.clone())
        .collect()
}

/// The bundle a launcher is joined to: the one it was made for, else an open bundle that links it.
#[must_use]
pub fn joined_bundle(board: &Board, rec: &Value) -> Option<String> {
    let name = text(rec, "name")?;
    if crate::served::is_master(board, name) {
        return None;
    }
    if truthy(rec, "bundle") {
        return Some(crate::json::display(&rec["bundle"]));
    }
    linked_bundles(board, name)
        .into_iter()
        .find(|tag| !truthy(&board["builds"][tag.as_str()], "closedAt"))
}

/// Not the master build, not joined to a bundle and not serving an item: it has a usage window.
/// A build that serves an item goes with the item and has no window.
#[must_use]
pub fn is_standalone(board: &Board, rec: &Value) -> bool {
    rec.is_object()
        && !is_master_rec(board, rec)
        && crate::served::for_of(rec).is_none()
        && joined_bundle(board, rec).is_none()
}

fn is_master_rec(board: &Board, rec: &Value) -> bool {
    text(rec, "name").is_some_and(|name| crate::served::is_master(board, name))
}

/// A bundle's status besides closed: `draft` or `ready`. A record without a status reads ready
/// when it was ever handed or starts something, else draft.
#[must_use]
pub fn bundle_status(board: &Board, tag: &str, rec: &Value) -> &'static str {
    match rec["status"].as_str() {
        Some("draft") => return "draft",
        Some("ready") => return "ready",
        _ => {}
    }
    if !rec.is_object() {
        return "draft";
    }
    let handed = board["handedBuild"].as_str() == Some(tag);
    if handed || truthy(rec, "lastHandedAt") || truthy(rec, "launcher") || truthy(rec, "cmd") {
        "ready"
    } else {
        "draft"
    }
}

/// The usage of a live standalone launcher.
pub struct Usage {
    /// The window in milliseconds.
    pub every_ms: i64,
    /// When it was last used, as an ISO time.
    pub last_used_at: String,
    /// Unused for longer than its window.
    pub stale: bool,
}

fn parse_every(value: &str) -> Option<i64> {
    let lower = js_lower(value);
    let (digits, unit) = ["mo", "h", "d", "w"]
        .iter()
        .find_map(|u| lower.strip_suffix(u).map(|d| (d, *u)))?;
    let valid = (1..=5).contains(&digits.len())
        && !digits.starts_with('0')
        && digits.chars().all(|c| c.is_ascii_digit());
    if !valid {
        return None;
    }
    let unit_ms: i64 = match unit {
        "h" => 3_600_000,
        "d" => 86_400_000,
        "w" => 7 * 86_400_000,
        _ => 30 * 86_400_000,
    };
    let ms = digits.parse::<i64>().ok()? * unit_ms;
    (EVERY_MIN_MS..=EVERY_MAX_MS).contains(&ms).then_some(ms)
}

fn js_lower(value: &str) -> String {
    argus_core::js::trim(value).to_lowercase()
}

fn ms_of(rec: &Value, key: &str) -> i64 {
    rec[key].as_str().and_then(parse_ms).unwrap_or(0)
}

/// The usage window of a launcher, or `None` for master, a joined one or a retired one.
#[must_use]
pub fn usage_of(board: &Board, rec: &Value, now_ms: i64) -> Option<Usage> {
    if !is_live(rec) || !is_standalone(board, rec) {
        return None;
    }
    let every_text = match &rec["useEvery"] {
        Value::Null => EVERY_LEGACY.to_owned(),
        v => crate::json::display(v),
    };
    let every_ms = parse_every(&every_text)
        .or_else(|| parse_every(EVERY_LEGACY))
        .unwrap_or(7 * 86_400_000);
    let last = ["createdAt", "updatedAt", "keptAt"]
        .iter()
        .map(|k| ms_of(rec, k))
        .max()
        .unwrap_or(0);
    let unused = (now_ms - last).max(0);
    let held = ms_of(rec, "keptUntil") > now_ms;
    Some(Usage {
        every_ms,
        last_used_at: iso_from_ms(last),
        stale: !held && unused > every_ms,
    })
}
