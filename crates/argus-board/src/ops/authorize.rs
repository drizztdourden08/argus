//! The ownership rules. A record belongs to one thread ([`owner_of`]); a thread changes only its
//! own, the coordinator changes anything, and the owner can do what the coordinator can except
//! restore from a backup. Authority comes from the caller's role, never from a flag.

use serde_json::Value;

use super::who::{Role, Who};
use crate::error::{BoardError, refuse};
use crate::json::{display, nullish};

type Verdict = Result<(), BoardError>;

const READ_VERBS: [&str; 3] = ["read", "list", "show"];
const CREATE_VERBS: [&str; 2] = ["create", "add"];
const COORDINATOR_ONLY: [&str; 6] = [
    "purge",
    "restore-backup",
    "trash",
    "dispatch",
    "free",
    "hand",
];
const EDITABLE_ENTRY: [&str; 2] = ["open", "pending"];

/// What is being checked: the verb on a record of a collection.
pub(crate) struct Check<'a> {
    verb: &'a str,
    coll: &'a str,
    doc: Option<&'a Value>,
    to: Option<&'a str>,
    fields: &'a [String],
}

impl<'a> Check<'a> {
    /// `verb` on `doc` (a record of `coll`, `None` for a create or a collection-wide verb).
    pub(crate) fn new(verb: &'a str, coll: &'a str, doc: Option<&'a Value>) -> Self {
        Check {
            verb,
            coll,
            doc,
            to: None,
            fields: &[],
        }
    }

    /// The thread that will own the new record (create) or end up with the moved one (move).
    pub(crate) fn to(mut self, to: Option<&'a str>) -> Self {
        self.to = to;
        self
    }

    /// What a set changes.
    pub(crate) fn fields(mut self, fields: &'a [String]) -> Self {
        self.fields = fields;
        self
    }

    fn id(&self) -> String {
        self.doc
            .and_then(|d| nullish(d, "id").or_else(|| nullish(d, "resource")))
            .unwrap_or_else(|| self.coll.to_owned())
    }
}

/// The one thread (or `human`) a record belongs to. Entries the owner or the portfolio wrote
/// belong to `human`; the thread they are routed to may only pick or close them.
pub(crate) fn owner_of(coll: &str, doc: &Value) -> Option<String> {
    match coll {
        "threads" => nullish(doc, "id"),
        "locks" => nullish(doc, "holder"),
        "entries" if matches!(doc["author"].as_str(), Some("human" | "portfolio")) => {
            Some("human".to_owned())
        }
        "entries" => nullish(doc, "thread").or_else(|| nullish(doc, "author")),
        "agents" | "inbox" => nullish(doc, "thread").or_else(|| nullish(doc, "id")),
        _ => nullish(doc, "thread"),
    }
}

fn hint(owner: &str) -> String {
    format!("Ask with coord inbox send {owner} \"...\" or coord message.")
}

fn belongs(id: &str, owner: &str, word: &str) -> String {
    let label = if id == owner {
        format!("the {owner} record")
    } else {
        id.to_owned()
    };
    let allowed = if owner == "coordinator" {
        "the coordinator".to_owned()
    } else {
        format!("{owner} or the coordinator")
    };
    format!(
        "{label} belongs to {owner}; only {allowed} can {word} it. {}",
        hint(owner)
    )
}

fn coordinator_words(verb: &str) -> &str {
    match verb {
        "restore-backup" => "restore from a backup",
        "hand" => "hand a build to the owner",
        other => other,
    }
}

fn verb_word(verb: &str) -> &str {
    match verb {
        "rm" => "delete",
        "archive" | "unarchive" | "restore" | "move" => verb,
        _ => "change",
    }
}

/// Refuses with the reason when `who` may not do what `check` describes.
pub(crate) fn authorize(who: &Who, c: &Check) -> Verdict {
    if READ_VERBS.contains(&c.verb) && c.coll != "inbox" {
        return Ok(());
    }
    let coordinator_thread =
        c.coll == "threads" && c.doc.is_some_and(|d| d["role"] == "coordinator");
    if coordinator_thread && matches!(c.verb, "archive" | "rm") {
        return Err(refuse(
            "the coordinator thread is never archived or deleted.",
        ));
    }
    match who.role {
        Role::Human if c.verb == "restore-backup" => {
            Err(refuse("only the coordinator can restore from a backup."))
        }
        Role::Human | Role::Coordinator => Ok(()),
        Role::Thread => thread_rules(who, c),
    }
}

fn thread_rules(who: &Who, c: &Check) -> Verdict {
    let Some(me) = who.self_id() else {
        return Err(refuse(
            "no thread is calling: this needs a registered session.",
        ));
    };
    if COORDINATOR_ONLY.contains(&c.verb) {
        return Err(refuse(format!(
            "only the coordinator can {}. {}",
            coordinator_words(c.verb),
            hint("coordinator")
        )));
    }
    if c.coll == "locks" {
        return lock_rule(me, c);
    }
    let creating = CREATE_VERBS.contains(&c.verb);
    if c.coll == "inbox" && (creating || c.verb == "send") {
        return Ok(());
    }
    if creating {
        return create_rule(me, c);
    }
    match c.doc {
        Some(doc) => doc_rule(me, c, doc),
        None => Err(refuse(format!("no {} record to {}.", c.coll, c.verb))),
    }
}

fn lock_rule(me: &str, c: &Check) -> Verdict {
    match c.verb {
        "claim" | "drop" => Ok(()),
        "release" if c.doc.and_then(|d| nullish(d, "holder")).as_deref() == Some(me) => Ok(()),
        "release" => Err(refuse(format!(
            "{} is held by {}; only the holder or the coordinator can release it.",
            c.id(),
            c.doc
                .and_then(|d| nullish(d, "holder"))
                .unwrap_or_else(|| "nobody".to_owned())
        ))),
        verb => Err(refuse(format!(
            "only the coordinator can {verb} a lock. {}",
            hint("coordinator")
        ))),
    }
}

fn create_rule(me: &str, c: &Check) -> Verdict {
    let owner = match (c.to, c.doc) {
        (Some(to), _) => Some(to.to_owned()),
        (None, Some(doc)) => owner_of(c.coll, doc),
        (None, None) => Some(me.to_owned()),
    };
    if owner.as_deref() == Some(me) {
        return Ok(());
    }
    let shown = owner.unwrap_or_else(|| "null".to_owned());
    Err(refuse(format!(
        "you cannot create records for {shown}; only the coordinator can. {}",
        hint(&shown)
    )))
}

fn thread_doc_rule(me: &str, c: &Check) -> Verdict {
    let id = c.id();
    if id != me {
        return Err(refuse(format!(
            "{me} cannot {} {id}: only {id} itself or the coordinator can. {}",
            c.verb,
            hint(&id)
        )));
    }
    let brief_or_rank = c.fields.iter().any(|f| f == "brief" || f == "rank");
    if c.verb == "move" || brief_or_rank {
        return Err(refuse(
            "only the coordinator can change a thread brief or the thread order.",
        ));
    }
    Ok(())
}

/// The rules of entries before the general owner check: whether the verb is settled outright.
fn entry_rule(me: &str, c: &Check, doc: &Value, owner: Option<&str>) -> Result<bool, BoardError> {
    match c.verb {
        "answer" => Err(refuse(format!(
            "only the coordinator can answer. {}",
            hint("coordinator")
        ))),
        "pick" | "done" | "ack" => {
            let routed = owner == Some("human") && doc["thread"].as_str() == Some(me);
            if owner == Some(me) || routed {
                return Ok(true);
            }
            let to = nullish(doc, "thread")
                .or_else(|| owner.map(str::to_owned))
                .unwrap_or_else(|| "nobody".to_owned());
            let word = if c.verb == "pick" { "pick" } else { "close" };
            Err(refuse(belongs(&c.id(), &to, word)))
        }
        _ => Ok(false),
    }
}

fn doc_rule(me: &str, c: &Check, doc: &Value) -> Verdict {
    let owner = owner_of(c.coll, doc);
    if c.coll == "threads" {
        return thread_doc_rule(me, c);
    }
    if c.coll == "entries" && entry_rule(me, c, doc, owner.as_deref())? {
        return Ok(());
    }
    if owner.as_deref() != Some(me) {
        let shown = owner.unwrap_or_else(|| "nobody".to_owned());
        let id = if c.coll == "inbox" {
            format!("the {shown} inbox")
        } else {
            c.id()
        };
        let word = if READ_VERBS.contains(&c.verb) {
            "read"
        } else {
            verb_word(c.verb)
        };
        return Err(refuse(belongs(&id, &shown, word)));
    }
    if c.verb == "move" && c.to.is_some_and(|to| Some(to) != owner.as_deref()) {
        return Err(refuse(format!(
            "{} is yours, but only the coordinator can hand it to {}. Ask with coord message or coord inbox send {} \"...\".",
            c.id(),
            c.to.unwrap_or_default(),
            c.to.unwrap_or_default()
        )));
    }
    let editable = EDITABLE_ENTRY.contains(&doc["status"].as_str().unwrap_or_default());
    if c.coll == "entries" && c.verb == "set" && !editable {
        return Err(refuse(format!(
            "{} is {}; only an open or pending one can be edited.",
            c.id(),
            display(&doc["status"])
        )));
    }
    Ok(())
}
