//! Which board events ping the owner: a build handed and a new open question by a thread.
//!
//! The first time a project is seen only sets the markers, so nothing old is told. The markers
//! move even when a switch is off, so turning it on later does not replay the past. An event older
//! than six hours when first seen is marked without a ping.

use argus_native::plain;

use crate::board::{BoardFacts, EntryFact};
use crate::clock::parse_ms;
use crate::config::AutoConfig;
use crate::kinds::By;
use crate::level::Level;
use crate::request::PingRequest;
use crate::seen::Marker;

const FRESH_MS: i64 = 6 * 3_600_000;
const BUNDLE_TITLE_MAX: usize = 80;
const QUESTION_READ_MAX: usize = 400;
const QUESTION_SHOWN_MAX: usize = 100;

/// A ping to send: the title, the text and who it is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    title: String,
    text: String,
    project: String,
    thread: Option<String>,
}

impl Draft {
    /// The `ping` from Argus itself that sends this draft.
    #[must_use]
    pub fn into_request(self) -> PingRequest {
        PingRequest {
            title: Some(self.title),
            text: self.text,
            level: Level::Ping,
            project: Some(self.project),
            thread: self.thread,
            by: By::Auto,
            url: None,
        }
    }
}

/// What `plan` needs to know besides the board and the marker.
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    /// The project key.
    pub key: &'a str,
    /// The display name of the project.
    pub name: &'a str,
    /// True the first time the project is seen.
    pub first: bool,
    /// Which pings are switched on.
    pub auto: AutoConfig,
    /// Now, in milliseconds.
    pub now: i64,
}

impl Context<'_> {
    fn fresh(&self, at: Option<&str>) -> bool {
        at.and_then(parse_ms)
            .is_none_or(|time| self.now - time <= FRESH_MS)
    }

    fn draft(&self, title: String, text: String, thread: Option<String>) -> Draft {
        Draft {
            title,
            text,
            project: self.key.to_owned(),
            thread,
        }
    }
}

/// The pings the board calls for; the marker is moved past everything seen.
pub fn plan(facts: &BoardFacts, marker: &mut Marker, context: &Context) -> Vec<Draft> {
    let mut drafts = Vec::new();
    drafts.extend(handed_ping(facts, marker, context));
    drafts.extend(question_pings(facts, marker, context));
    drafts
}

fn handed_ping(facts: &BoardFacts, marker: &mut Marker, context: &Context) -> Option<Draft> {
    let tag = facts.handed_build.as_deref().filter(|tag| !tag.is_empty());
    let current = tag.map(|tag| format!("{tag}|{}", facts.handed_at.as_deref().unwrap_or("")));
    if current == marker.handed {
        return None;
    }
    marker.handed = current;
    let tag = tag?;
    let told = !context.first && context.auto.handed && context.fresh(facts.handed_at.as_deref());
    told.then(|| handed_draft(facts, tag, context))
}

fn handed_draft(facts: &BoardFacts, tag: &str, context: &Context) -> Draft {
    let bundle = facts.bundle.clone().unwrap_or_default();
    let title = bundle.title.as_deref().filter(|title| !title.is_empty());
    let shown = plain(title.unwrap_or(tag), BUNDLE_TITLE_MAX);
    let tests = facts
        .tests
        .iter()
        .filter(|test| !test.removed && test.build.as_deref() == Some(tag))
        .count();
    let count = match tests {
        0 => String::new(),
        1 => " (1 test)".to_owned(),
        n => format!(" ({n} tests)"),
    };
    let name = context.name;
    context.draft(
        format!("{name}: ready to test"),
        format!("{name}: {shown} is ready to test{count}"),
        bundle.thread,
    )
}

fn question_pings(facts: &BoardFacts, marker: &mut Marker, context: &Context) -> Vec<Draft> {
    let mut drafts = Vec::new();
    for question in open_questions(facts) {
        if marker.asked.contains(&question.id) {
            continue;
        }
        let told =
            !context.first && context.auto.ask && context.fresh(question.created_at.as_deref());
        if told {
            drafts.push(question_draft(question, context));
        }
        marker.remember(&question.id);
    }
    drafts
}

fn open_questions(facts: &BoardFacts) -> Vec<&EntryFact> {
    let mut open: Vec<&EntryFact> = facts
        .entries
        .iter()
        .filter(|entry| entry.kind == "question" && entry.status == "open")
        .filter(|entry| entry.author != "human" && !entry.removed)
        .collect();
    open.sort_by(|a, b| {
        let by_time = a.created_at.as_deref().unwrap_or("");
        by_time
            .cmp(b.created_at.as_deref().unwrap_or(""))
            .then_with(|| a.id.cmp(&b.id))
    });
    open
}

fn question_draft(question: &EntryFact, context: &Context) -> Draft {
    let said = plain(&question.text, QUESTION_READ_MAX);
    let shown = if said.chars().count() > QUESTION_SHOWN_MAX {
        let head: String = said.chars().take(QUESTION_SHOWN_MAX).collect();
        format!("{head}\u{2026}")
    } else {
        said
    };
    let name = context.name;
    let needs = if question.needs_decision {
        "needs a decision"
    } else {
        "has a question"
    };
    context.draft(
        format!("{name} {needs}"),
        format!("{name} asks: {shown}"),
        question.thread.clone(),
    )
}
