//! The rows of `build list` (`buildRows`): one per bundle record and per tag on a live test, plus
//! the handed tag even when nothing carries it yet. Handed first, then the rest newest-looking
//! first. "to run" is every test not run yet; `cmd` and `cwd` are what Play starts (the linked
//! launcher's, or a legacy bundle's own) and `launcherState` says where they come from.

use argus_board::Board;
use argus_board::json::{section, truthy};
use argus_launchers::launcher::{listed_status, resolve_bundle};
use serde_json::{Value, json};

use super::support::live;
use crate::cli::exit::CoordError;
use crate::verbs::agent::rules::typed_board;
use argus_core::collate::compare;

const FAILING: [&str; 3] = ["fail", "partial", "blocked"];

/// One line of the list.
pub struct Row {
    /// The tag.
    pub build: String,
    /// Draft, ready or closed; none for a tag with no record that is not handed.
    pub status: Option<String>,
    /// The record's title.
    pub title: Option<String>,
    /// The record's note.
    pub note: Option<String>,
    /// The launcher the bundle starts.
    pub launcher: Option<String>,
    /// Where Play comes from: linked, retired, missing, legacy or none.
    pub launcher_state: &'static str,
    /// What Play starts.
    pub cmd: Option<String>,
    /// Where it starts.
    pub cwd: Option<String>,
    /// The managing thread.
    pub thread: Option<String>,
    /// When it was closed.
    pub closed_at: Option<String>,
    /// Who closed it.
    pub closed_by: Option<String>,
    /// Whether a record exists.
    pub record: bool,
    /// Live tests carrying the tag.
    pub tests: u32,
    /// Of those, passed.
    pub pass: u32,
    /// Of those, failed, partial or blocked.
    pub fail: u32,
    /// Of those, not run yet.
    pub to_run: u32,
    /// The handed build.
    pub handed: bool,
}

impl Row {
    fn blank(build: &str, handed_tag: Option<&str>) -> Row {
        let handed = Some(build) == handed_tag;
        Row {
            build: build.to_owned(),
            status: handed.then(|| "ready".to_owned()),
            title: None,
            note: None,
            launcher: None,
            launcher_state: "none",
            cmd: None,
            cwd: None,
            thread: None,
            closed_at: None,
            closed_by: None,
            record: false,
            tests: 0,
            pass: 0,
            fail: 0,
            to_run: 0,
            handed,
        }
    }

    /// The row as `--json` prints it.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "build": self.build, "status": self.status, "title": self.title, "note": self.note,
            "launcher": self.launcher, "launcherState": self.launcher_state, "cmd": self.cmd,
            "cwd": self.cwd, "thread": self.thread, "closedAt": self.closed_at,
            "closedBy": self.closed_by, "record": self.record, "tests": self.tests,
            "pass": self.pass, "fail": self.fail, "toRun": self.to_run, "handed": self.handed,
        })
    }
}

fn text(v: &Value) -> Option<String> {
    (!v.is_null()).then(|| argus_board::json::display(v))
}

/// The index of the row of a tag, adding a blank one.
fn row_of(rows: &mut Vec<Row>, tag: &str, handed: Option<&str>) -> usize {
    if let Some(at) = rows.iter().position(|r| r.build == tag) {
        return at;
    }
    rows.push(Row::blank(tag, handed));
    rows.len() - 1
}

fn records(board: &Board, rows: &mut Vec<Row>, handed: Option<&str>) -> Result<(), CoordError> {
    let typed = typed_board(board)?;
    for (tag, rec) in section(board, "builds") {
        let at = row_of(rows, tag, handed);
        let starts = resolve_bundle(&typed, typed.builds.get(tag));
        let status = typed
            .builds
            .get(tag)
            .map(|b| listed_status(&typed, tag, b).as_str().to_owned());
        rows[at] = Row {
            status: if truthy(rec, "closedAt") {
                Some("closed".to_owned())
            } else {
                status
            },
            title: text(&rec["title"]),
            note: text(&rec["note"]),
            launcher: starts.launcher,
            launcher_state: starts.state.as_str(),
            cmd: starts.cmd,
            cwd: starts.cwd,
            thread: text(&rec["thread"]),
            closed_at: text(&rec["closedAt"]),
            closed_by: text(&rec["closedBy"]),
            record: true,
            ..Row::blank(tag, handed)
        };
    }
    Ok(())
}

/// The rows of the board, sorted.
///
/// # Errors
/// A bundle or launcher record that cannot be read.
pub fn build_rows(board: &Board) -> Result<Vec<Row>, CoordError> {
    let handed = board["handedBuild"].as_str().filter(|h| !h.is_empty());
    let mut rows = Vec::new();
    records(board, &mut rows, handed)?;
    for test in section(board, "tests").values().filter(|t| live(t)) {
        let Some(tag) = test["build"].as_str().filter(|t| !t.is_empty()) else {
            continue;
        };
        let at = row_of(&mut rows, tag, handed);
        let row = &mut rows[at];
        row.tests += 1;
        match test["status"].as_str() {
            Some("pass") => row.pass += 1,
            Some(s) if FAILING.contains(&s) => row.fail += 1,
            Some("pending") => row.to_run += 1,
            _ => {}
        }
    }
    if let Some(tag) = handed {
        row_of(&mut rows, tag, handed);
    }
    rows.sort_by(|a, b| {
        b.handed
            .cmp(&a.handed)
            .then_with(|| compare(&b.build, &a.build, true))
    });
    Ok(rows)
}
