//! What a launcher starts: the link state of a build a test run names in its `build` field.

use super::record::LauncherBoard;

/// Where a build's Play comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkState {
    Linked,
    Retired,
    Missing,
    None,
}

impl LinkState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linked => "linked",
            Self::Retired => "retired",
            Self::Missing => "missing",
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

/// What a build starts: the launcher a test run names (live, retired or missing), else nothing.
pub fn resolve_launcher(board: &LauncherBoard, name: Option<&str>) -> Resolved {
    let Some(name) = name else {
        return Resolved::of(LinkState::None, None, None, None);
    };
    match board.launchers.get(name) {
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
    }
}
