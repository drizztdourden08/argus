//! The part of a project board the wake logic reads, parsed tolerantly from the board JSON.

use std::sync::Arc;

use argus_board::ids::shown;
use indexmap::IndexMap;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use serde_json::{Value, json};

use crate::dep_wakes::{DepView, RequestedBy, WaitsOn};
use crate::lenient::{self, Flag, Num, Text};
use crate::time::parse_ms;

/// Statuses of a thread that has ended.
pub const ENDED: [&str; 2] = ["done", "archived"];

/// Statuses of a sub-agent that is still running.
pub const LIVE_AGENT: [&str; 3] = ["working", "waiting", "stop-requested"];

/// A turn this recent counts as running.
const BUSY_MS: i64 = 3 * 60 * 1000;

/// The fields every board record shares.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Record {
    /// The record id.
    pub id: Text,
    /// The work item it belongs to.
    pub item: Text,
    /// When it was made.
    pub created_at: Text,
    /// When it last changed.
    pub updated_at: Text,
    /// When it finished.
    pub finished_at: Text,
    /// When it was archived.
    pub archived_at: Text,
    /// When it was deleted.
    pub deleted_at: Text,
}

impl Record {
    /// Is it neither deleted nor archived?
    #[must_use]
    pub fn is_live(&self) -> bool {
        self.deleted_at.some().is_none() && self.archived_at.some().is_none()
    }

    /// Is it a live record of this item?
    #[must_use]
    pub fn is_live_on(&self, item_id: &str) -> bool {
        self.item.is(item_id) && self.is_live()
    }
}

/// A thread of the project.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ThreadView {
    /// The thread id.
    pub id: Text,
    /// `coordinator` or `thread`.
    pub role: Text,
    /// `active`, `paused`, `done`, `archived`...
    pub status: Text,
    /// A placeholder thread is not a session.
    pub placeholder: Flag,
    /// The session title.
    pub session: Text,
    /// The CLI session id the last hook saw.
    pub session_id: Text,
    /// The last sign of life (ISO).
    pub last_seen: Text,
    /// When the thread registered on the board (ISO).
    pub registered_at: Text,
    /// `working` or `idle`.
    pub activity: Text,
    /// `argus` for the hosted Overlord's thread; absent (desktop) for every other.
    pub host: Text,
}

impl ThreadView {
    /// Is the session a child that Argus itself runs (woken by user messages, no listener, no board tab)?
    #[must_use]
    pub fn is_hosted(&self) -> bool {
        self.host.is("argus")
    }

    /// Has the thread ended?
    #[must_use]
    pub fn is_ended(&self) -> bool {
        self.status.is_any(&ENDED)
    }

    /// Not ended and not a placeholder.
    #[must_use]
    pub fn is_live(&self) -> bool {
        !self.is_ended() && !self.placeholder.on()
    }

    /// Is a turn running at `now` (epoch ms): it works and showed a sign of life in the last 3 minutes?
    #[must_use]
    pub fn in_turn(&self, now: i64) -> bool {
        let seen = parse_ms(self.last_seen.as_str()).unwrap_or(0);
        self.activity.is("working") && now - seen <= BUSY_MS
    }
}

/// A work item.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ItemView {
    /// Shared fields.
    #[serde(flatten)]
    pub rec: Record,
    /// The title.
    pub title: Text,
    /// The type of the item (`task`, `bug`...); absent on an old item.
    #[serde(rename = "type")]
    pub kind: Text,
    /// The owning thread.
    pub thread: Text,
    /// Who holds the item: a thread, `owner`, or `agent:A-n`.
    pub assignee: Text,
    /// `todo`, `doing`, `review`, `done`, `dropped`.
    pub status: Text,
    /// The requests to other boards this item waits on, one per dependency.
    #[serde(deserialize_with = "lenient::one_or_many")]
    pub waits_on: Vec<WaitsOn>,
    /// The requests of other boards this item answers, one per dependency.
    #[serde(deserialize_with = "lenient::one_or_many")]
    pub requested_by: Vec<RequestedBy>,
    /// What the item is blocked on, in words; absent when nothing blocks it.
    pub blocker: Text,
}

impl ItemView {
    /// The id the owner reads: the letters of its type and its number (`B-12`), not the stored `I-12`.
    #[must_use]
    pub fn shown_id(&self) -> String {
        shown(&json!({"id": self.rec.id.as_str(), "type": self.kind.some()}))
    }
}

/// A test case.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TestView {
    /// Shared fields.
    #[serde(flatten)]
    pub rec: Record,
    /// The thread that owns it.
    pub thread: Text,
    /// The build the test ran on.
    pub build: Text,
    /// `pending`, `pass`, `fail`...
    pub status: Text,
    /// `manual` or `auto`.
    pub mode: Text,
}

/// A to-do (action).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ActionView {
    /// Shared fields.
    #[serde(flatten)]
    pub rec: Record,
    /// `pending`, `done`...
    pub status: Text,
}

/// A sub-agent.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AgentView {
    /// Shared fields.
    #[serde(flatten)]
    pub rec: Record,
    /// Its name.
    pub name: Text,
    /// The thread that owns it.
    pub thread: Text,
    /// The model it was started with.
    pub model: Text,
    /// The model it really runs on.
    pub model_used: Text,
    /// `working`, `waiting`, `done`...
    pub status: Text,
    /// What it works on.
    pub task: Text,
    /// When it started (ISO).
    pub started_at: Text,
    /// Set by the reconciler when Claude has no task for it.
    pub orphan_since: Text,
    /// Set by the reconciler when its turn ended.
    pub idle_since: Text,
    /// Minutes between check-ins, as the lead chose.
    pub checkin_min: Num,
}

/// An owner or thread entry.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EntryView {
    /// Shared fields.
    #[serde(flatten)]
    pub rec: Record,
    /// `result`, `followup`, `decision`, `move`, `bulk`, `resume`, `question`...
    pub kind: Text,
    /// `open`, `pending`, `applied`...
    pub status: Text,
    /// The text.
    pub text: Text,
    /// A result's outcome.
    pub outcome: Text,
    /// The entry or thread this answers.
    pub reply_to: Text,
    /// The thread it is about.
    pub thread: Text,
    /// Who wrote it.
    pub author: Text,
    /// A portfolio send's project.
    pub project: Text,
    /// The Worktrees tab request a followup carries.
    #[serde(deserialize_with = "lenient::object")]
    pub worktree: Option<WorktreeRequest>,
    /// The operations of a bulk entry.
    #[serde(deserialize_with = "lenient::list")]
    pub ops: Vec<Op>,
    /// The results of those operations.
    #[serde(deserialize_with = "lenient::list")]
    pub results: Vec<OpResult>,
    /// The projects a portfolio send reached.
    #[serde(deserialize_with = "lenient::list")]
    pub delivered_to: Vec<Text>,
    /// What a move entry acts on.
    #[serde(deserialize_with = "lenient::object")]
    pub target: Option<OpTarget>,
    /// Where a move entry moves to.
    pub to: Option<Value>,
}

impl EntryView {
    /// The entry itself seen as one operation.
    #[must_use]
    pub fn as_op(&self) -> Op {
        Op {
            kind: self.kind.clone(),
            target: self.target.clone(),
            to: self.to.clone(),
        }
    }
}

/// The request a Delete or Check button of the Worktrees tab posts.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct WorktreeRequest {
    /// The worktree id.
    pub id: Text,
    /// Its path.
    pub path: Text,
    /// `delete` or `check`.
    pub kind: Text,
    /// What the owner wrote.
    pub message: Text,
}

/// An operation of a move or a bulk entry.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Op {
    /// `move`, `edit`...
    pub kind: Text,
    /// What it acts on.
    #[serde(deserialize_with = "lenient::object")]
    pub target: Option<OpTarget>,
    /// Where it moves to.
    pub to: Option<Value>,
}

impl Op {
    /// Does it move an item to a thread (the `to` has a `thread` key)?
    #[must_use]
    pub fn moves_to_thread(&self) -> bool {
        self.kind.is("move")
            && self
                .target
                .as_ref()
                .is_some_and(|t| t.collection.is("items"))
            && self
                .to
                .as_ref()
                .is_some_and(|to| to.get("thread").is_some())
    }
}

/// What an operation acts on.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OpTarget {
    /// The collection (`items`).
    #[serde(rename = "c")]
    pub collection: Text,
    /// The record id.
    pub id: Text,
}

/// The result of one operation of a bulk entry.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OpResult {
    /// `applied`, `refused`...
    pub status: Text,
}

/// A message waiting in a thread's inbox.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct InboxMessage {
    /// The message id.
    pub id: Text,
}

/// A project board, as far as the wakes care.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(remote = "Self", default, rename_all = "camelCase")]
pub struct BoardView {
    /// The threads by id.
    #[serde(deserialize_with = "lenient::map")]
    pub threads: IndexMap<String, ThreadView>,
    /// Waiting messages by thread.
    #[serde(deserialize_with = "lenient::map")]
    pub inbox: IndexMap<String, Vec<InboxMessage>>,
    /// The work items.
    #[serde(deserialize_with = "lenient::map")]
    pub items: IndexMap<String, ItemView>,
    /// The tests.
    #[serde(deserialize_with = "lenient::map")]
    pub tests: IndexMap<String, TestView>,
    /// The to-dos.
    #[serde(deserialize_with = "lenient::map")]
    pub actions: IndexMap<String, ActionView>,
    /// The sub-agents.
    #[serde(deserialize_with = "lenient::map")]
    pub agents: IndexMap<String, AgentView>,
    /// The entries.
    #[serde(deserialize_with = "lenient::map")]
    pub entries: IndexMap<String, EntryView>,
    /// The dependency ledger (only the portfolio board holds one).
    #[serde(deserialize_with = "lenient::map")]
    pub deps: IndexMap<String, DepView>,
    /// The board JSON these views were read from, for the blocked rule that `argus-board` shares.
    #[serde(skip)]
    raw: Arc<Value>,
}

impl<'de> Deserialize<'de> for BoardView {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Value::deserialize(deserializer)?;
        let mut view = BoardView::deserialize(&raw).map_err(D::Error::custom)?;
        view.raw = Arc::new(raw);
        Ok(view)
    }
}

impl BoardView {
    /// The board JSON the view was parsed from.
    #[must_use]
    pub fn json(&self) -> &Value {
        &self.raw
    }

    /// A board parsed from its JSON; anything unreadable reads as empty.
    #[must_use]
    pub fn from_json(value: Value) -> Self {
        serde_json::from_value(value).unwrap_or_default()
    }

    /// The ids of the messages waiting for a thread.
    #[must_use]
    pub fn inbox_ids(&self, thread: &str) -> Vec<String> {
        self.inbox
            .get(thread)
            .map(|list| {
                list.iter()
                    .filter_map(|m| m.id.some().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The coordinator thread of the board: the first one that has not ended.
    #[must_use]
    pub fn coordinator(&self) -> Option<String> {
        self.threads
            .iter()
            .find(|(_, t)| t.role.is("coordinator") && !t.is_ended())
            .map(|(key, t)| t.id.some().map_or_else(|| key.clone(), str::to_owned))
    }
}
