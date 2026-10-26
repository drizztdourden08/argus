//! Bundle status and what a bundle starts.
//!
//! A bundle is `draft` (not built yet: no Play, cannot be handed), `ready` (built and
//! testable) or `closed` (`closedAt`). A record without `status` (written before it
//! existed) reads ready when it was ever handed or starts something, else draft: no data
//! rewrite is needed.

use super::record::{Bundle, LauncherBoard};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BundleStatus {
    Draft,
    Ready,
    Closed,
}

impl BundleStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Ready => "ready",
            Self::Closed => "closed",
        }
    }
}

/// The status of a bundle besides closed: draft or ready.
pub fn bundle_status(board: &LauncherBoard, tag: &str, rec: Option<&Bundle>) -> BundleStatus {
    let Some(rec) = rec else {
        return BundleStatus::Draft;
    };
    match rec.status.as_deref() {
        Some("draft") => return BundleStatus::Draft,
        Some("ready") => return BundleStatus::Ready,
        _ => {}
    }
    let starts_something = rec.launcher.is_some() || rec.cmd.is_some();
    if board.handed_build.as_deref() == Some(tag)
        || rec.last_handed_at.is_some()
        || starts_something
    {
        BundleStatus::Ready
    } else {
        BundleStatus::Draft
    }
}

/// The status a listing shows: closed wins.
pub fn listed_status(board: &LauncherBoard, tag: &str, rec: &Bundle) -> BundleStatus {
    if rec.closed_at.is_some() {
        BundleStatus::Closed
    } else {
        bundle_status(board, tag, Some(rec))
    }
}

/// Where a bundle's Play comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkState {
    Linked,
    Retired,
    Missing,
    Legacy,
    None,
}

impl LinkState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linked => "linked",
            Self::Retired => "retired",
            Self::Missing => "missing",
            Self::Legacy => "legacy",
            Self::None => "none",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub state: LinkState,
    pub launcher: Option<String>,
    pub cmd: Option<String>,
    pub cwd: Option<String>,
}

impl Resolved {
    fn of(state: LinkState, launcher: Option<&str>, cmd: Option<&str>, cwd: Option<&str>) -> Self {
        Self {
            state,
            launcher: launcher.map(str::to_owned),
            cmd: cmd.map(str::to_owned),
            cwd: cwd.map(str::to_owned),
        }
    }
}

/// What a bundle starts: its linked launcher (live, retired or missing), else the command a
/// bundle written before launchers still carries (legacy), else nothing.
pub fn resolve_bundle(board: &LauncherBoard, rec: Option<&Bundle>) -> Resolved {
    let Some(rec) = rec else {
        return Resolved::of(LinkState::None, None, None, None);
    };
    if let Some(name) = rec.launcher.as_deref() {
        return match board.launchers.get(name) {
            None => Resolved::of(LinkState::Missing, Some(name), None, None),
            Some(l) if l.retired_at.is_some() => {
                Resolved::of(LinkState::Retired, Some(name), None, None)
            }
            Some(l) => Resolved::of(
                LinkState::Linked,
                Some(name),
                l.cmd.as_deref(),
                l.cwd.as_deref(),
            ),
        };
    }
    match rec.cmd.as_deref().filter(|c| !c.is_empty()) {
        Some(cmd) => Resolved::of(LinkState::Legacy, None, Some(cmd), rec.cwd.as_deref()),
        None => Resolved::of(LinkState::None, None, None, None),
    }
}
