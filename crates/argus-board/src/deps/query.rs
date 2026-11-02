//! Reading the ledger: the list, one dependency, the dependency of an item.

use std::path::Path;
use std::str::FromStr;

use serde_json::Value;

use super::error::{DepError, not_a_dependency_of};
use super::ledger::{deps_in, get, read};
use super::model::{Dep, DepState, End, node};
use crate::json::doc;
use crate::store::read_board;

/// Which side of the dependencies of a project a list shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// The project waits on someone.
    Waiting,
    /// Someone waits on the project.
    ForMe,
    /// Both.
    #[default]
    All,
}

impl FromStr for View {
    type Err = DepError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "waiting" => Ok(View::Waiting),
            "for-me" => Ok(View::ForMe),
            "all" => Ok(View::All),
            other => Err(DepError::Bad(format!(
                "view must be waiting, for-me or all, not \"{other}\""
            ))),
        }
    }
}

/// The query of `GET /api/deps`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListFilter {
    /// The project whose dependencies are listed; every project when `None`.
    pub project: Option<String>,
    /// The side of the project.
    pub view: View,
    /// Only this state.
    pub state: Option<DepState>,
}

impl ListFilter {
    /// The filter of the query parameters `project`, `view` and `state`.
    ///
    /// # Errors
    /// A view or a state that does not exist (400).
    pub fn parse(
        project: Option<&str>,
        view: Option<&str>,
        state: Option<&str>,
    ) -> Result<ListFilter, DepError> {
        Ok(ListFilter {
            project: project.filter(|p| !p.is_empty()).map(str::to_owned),
            view: view
                .filter(|v| !v.is_empty())
                .map(str::parse::<View>)
                .transpose()?
                .unwrap_or_default(),
            state: state
                .filter(|s| !s.is_empty())
                .map(str::parse::<DepState>)
                .transpose()
                .map_err(|e| DepError::Bad(e.to_string()))?,
        })
    }

    fn keeps(&self, dep: &Dep) -> bool {
        let side = match (&self.project, self.view) {
            (None, _) => true,
            (Some(p), View::Waiting) => dep.from.project == *p,
            (Some(p), View::ForMe) => dep.to.project == *p,
            (Some(p), View::All) => dep.from.project == *p || dep.to.project == *p,
        };
        side && self.state.is_none_or(|s| dep.state == s)
    }
}

/// The dependencies a filter keeps, oldest first.
///
/// # Errors
/// The ledger cannot be read.
pub fn list(home: &Path, filter: &ListFilter) -> Result<Vec<Dep>, DepError> {
    let all = deps_in(&read(home)?);
    Ok(all.into_iter().filter(|d| filter.keeps(d)).collect())
}

/// One line per dependency, or `no dependencies`.
#[must_use]
pub fn lines(deps: &[Dep]) -> String {
    if deps.is_empty() {
        return "no dependencies".to_owned();
    }
    deps.iter().map(Dep::line).collect::<Vec<_>>().join("\n")
}

fn touches(end: &End, project: &str, item: &str) -> bool {
    end.project == project && end.item.as_deref() == Some(item)
}

/// The dependency of an item, on either end: the live one, else the latest.
///
/// # Errors
/// The item has none (404), or the ledger cannot be read.
pub fn by_item(home: &Path, project: &str, item: &str) -> Result<Dep, DepError> {
    let deps = deps_in(&read(home)?);
    let mine = |d: &&Dep| touches(&d.from, project, item) || touches(&d.to, project, item);
    let live = deps.iter().filter(mine).find(|d| d.state.is_live());
    live.or_else(|| deps.iter().rfind(mine))
        .cloned()
        .ok_or_else(|| DepError::Unknown(format!("no dependency on {}", node(project, item))))
}

/// A dependency with the two items it links, as their boards hold them now.
#[derive(Debug, Clone, PartialEq)]
pub struct Shown {
    /// The record, history included.
    pub dep: Dep,
    /// The requester item, `null` when it is gone.
    pub requester: Value,
    /// The assignee item, `null` when it is gone.
    pub assignee: Value,
}

fn item_doc(home: &Path, end: &End) -> Result<Value, DepError> {
    let board = read_board(home, &end.project)?;
    let item = end.item.as_deref().unwrap_or_default();
    Ok(board
        .and_then(|b| doc(&b, "items", item).cloned())
        .unwrap_or(Value::Null))
}

/// One dependency with both items. `project` is the caller's: a dependency that has no end on
/// it is refused.
///
/// # Errors
/// The dependency is unknown or not the project's (404), or a board cannot be read.
pub fn show(home: &Path, id: &str, project: Option<&str>) -> Result<Shown, DepError> {
    let dep = get(home, id)?;
    if let Some(p) = project.filter(|p| dep.from.project != *p && dep.to.project != *p) {
        return Err(not_a_dependency_of(id, p));
    }
    Ok(Shown {
        requester: item_doc(home, &dep.from)?,
        assignee: item_doc(home, &dep.to)?,
        dep,
    })
}
