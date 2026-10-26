//! When a standalone launcher was last used and whether it is stale.

use super::every::{EVERY_LEGACY, parse_every};
use super::join::{is_live, is_standalone};
use super::record::{Launcher, LauncherBoard};
use crate::time::{format_iso, parse_iso};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Usage {
    pub use_every: String,
    pub every_ms: i64,
    /// No `useEvery` on the record yet: treated as 7d until the thread sets one.
    pub legacy: bool,
    pub last_used_ms: i64,
    pub last_used_at: String,
    pub unused_ms: i64,
    pub stale: bool,
}

fn ms_of(iso: Option<&str>) -> i64 {
    iso.and_then(parse_iso).unwrap_or(0)
}

/// The newest of the Play runs (`played_ms`, from the dashboard's runs; 0 when unknown),
/// the record's own `updatedAt` and `createdAt`, and the last `coord launcher keep`.
pub fn last_used_ms(rec: &Launcher, played_ms: i64) -> i64 {
    [
        ms_of(Some(&rec.created_at)),
        ms_of(Some(&rec.updated_at)),
        ms_of(rec.kept_at.as_deref()),
        played_ms,
    ]
    .into_iter()
    .max()
    .unwrap_or(0)
}

/// The usage of a live standalone launcher, or `None` for master, a joined one or a
/// retired one. A `keep --days` holds it fresh until `keptUntil`.
pub fn usage_of(
    board: &LauncherBoard,
    rec: &Launcher,
    now_ms: i64,
    played_ms: i64,
) -> Option<Usage> {
    if !is_live(Some(rec)) || !is_standalone(board, rec) {
        return None;
    }
    let every = parse_every(rec.use_every.as_deref().unwrap_or(EVERY_LEGACY))
        .or_else(|_| parse_every(EVERY_LEGACY))
        .ok()?;
    let last = last_used_ms(rec, played_ms);
    let unused = (now_ms - last).max(0);
    let held = ms_of(rec.kept_until.as_deref()) > now_ms;
    Some(Usage {
        use_every: every.text,
        every_ms: every.ms,
        legacy: rec.use_every.is_none(),
        last_used_ms: last,
        last_used_at: format_iso(last),
        unused_ms: unused,
        stale: !held && unused > every.ms,
    })
}
