//! What an ask checks before it takes any lock: the target is a board, the requester item is
//! open, the text fits its limits, the assignee thread is live.

use std::path::Path;

use argus_core::ids::PORTFOLIO;
use serde::Deserialize;
use serde_json::{Value, json};

use super::error::{DepError, bad, no_lead, not_a_board, not_open, same_project, thread_not_live};
use super::model::node;
use super::roster::Roster;
use super::sight::Seen;
use crate::Board;
use crate::error::BoardError;
use crate::json::doc;
use crate::limits::{optional_text, require_text, text_list};
use crate::portfolio::project_keys;
use crate::store::{board_path, read_board};

/// The body of `POST /api/deps`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AskRequest {
    /// The requester's project.
    pub project: String,
    /// The requester item.
    pub item: String,
    /// Who asks: `project/T-n` or `project lead`.
    pub actor: String,
    /// The project asked.
    pub target: String,
    /// The thread asked; the target lead by default.
    #[serde(default)]
    pub to: Option<String>,
    /// The title of the request, and of the new item.
    pub title: String,
    /// What the new item says.
    #[serde(default)]
    pub description: Option<String>,
    /// Acceptance criteria of the new item.
    #[serde(default)]
    pub ac: Vec<String>,
    /// An existing item of the target to request, instead of a new one.
    #[serde(default)]
    pub onto: Option<String>,
}

/// Everything an ask learned before it started.
pub(crate) struct Plan {
    pub title: String,
    pub description: String,
    pub ac: Vec<String>,
    pub thread: String,
    pub requester_thread: Option<String>,
    pub requester_title: String,
    pub assignee_title: String,
}

/// The projects with a board, other than `own`.
fn boards_here(home: &Path, own: &str) -> Result<Vec<String>, DepError> {
    let registry = read_board(home, PORTFOLIO)?.unwrap_or(Value::Null);
    Ok(project_keys(home, &registry)
        .into_iter()
        .filter(|k| k != own && board_path(home, k).exists())
        .collect())
}

fn board_of(home: &Path, project: &str) -> Result<Board, DepError> {
    read_board(home, project)?
        .ok_or_else(|| DepError::Board(BoardError::Missing(project.to_owned())))
}

fn item_of(board: &Board, project: &str, item: &str) -> Result<Seen, DepError> {
    doc(board, "items", item)
        .map(|i| Seen::of(item, i))
        .ok_or_else(|| DepError::Unknown(format!("no item {item} on {project}")))
}

fn pick_thread(roster: &dyn Roster, target: &str, to: Option<&str>) -> Result<String, DepError> {
    let Some(thread) = to else {
        return roster.lead(target).ok_or_else(|| no_lead(target));
    };
    let live = roster.live_threads(target);
    if live.iter().any(|l| l == thread) {
        Ok(thread.to_owned())
    } else {
        Err(thread_not_live(thread, target, &live))
    }
}

/// The existing assignee item a request lands on: it must still be open work.
fn onto_item(home: &Path, req: &AskRequest, onto: &str) -> Result<Seen, DepError> {
    let seen = item_of(&board_of(home, &req.target)?, &req.target, onto)?;
    if seen.open() {
        Ok(seen)
    } else {
        Err(DepError::Unknown(format!(
            "no open item {onto} on {}",
            req.target
        )))
    }
}

/// Check an ask without locking anything.
pub(crate) fn plan(home: &Path, req: &AskRequest, roster: &dyn Roster) -> Result<Plan, DepError> {
    if req.project == req.target {
        return Err(same_project());
    }
    let known = boards_here(home, &req.project)?;
    if !known.contains(&req.target) {
        return Err(not_a_board(&req.target, &known));
    }
    let own = item_of(&board_of(home, &req.project)?, &req.project, &req.item)?;
    if !own.open() {
        return Err(not_open(&node(&req.project, &req.item), own.shown_status()));
    }
    let title = require_text("title", &json!(req.title)).map_err(bad)?;
    let description = optional_text("description", &json!(req.description)).map_err(bad)?;
    let ac = text_list("ac", &json!(req.ac), 20).map_err(bad)?;
    let onto = match req.onto.as_deref() {
        Some(onto) => Some(onto_item(home, req, onto)?),
        None => None,
    };
    let thread = match onto.as_ref().and_then(|o| o.thread.clone()) {
        Some(thread) => thread,
        None => pick_thread(roster, &req.target, req.to.as_deref())?,
    };
    Ok(Plan {
        assignee_title: onto.map_or_else(|| title.clone(), |o| o.title),
        title,
        description: description.unwrap_or_default(),
        ac,
        thread,
        requester_thread: own.thread,
        requester_title: own.title,
    })
}
