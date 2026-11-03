//! Entering a stage: the flow of the type, the fields the stage requires, its gates and, for a
//! stage in the done lane, the done rule of the type.

use serde_json::Value;

use super::needs::{Subject, missing};
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::ops::Who;
use crate::registry::{ItemType, Mode, Stage};

/// A move of an item into a stage.
pub(crate) struct Entry<'a> {
    pub board: &'a Board,
    /// The item as it will be: its id, status, fields and criteria.
    pub item: &'a Value,
    pub ty: &'a ItemType,
    pub from: Option<&'a Stage>,
    pub to: &'a Stage,
    pub who: &'a Who,
    /// The reason that overrides a rule which refuses.
    pub force: Option<&'a str>,
}

/// What entering a stage let through.
#[derive(Debug, Default)]
pub(crate) struct Entered {
    /// What a rule that only warns found missing.
    pub warnings: Vec<String>,
    /// What a rule that refuses found missing, overridden by the force reason.
    pub overridden: Vec<String>,
}

fn label(item: &Value) -> &str {
    item["id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .unwrap_or("the new item")
}

/// The owner moves freely; anyone else follows the `next` list of the stage left.
fn flow(e: &Entry) -> Result<(), BoardError> {
    let Some(from) = e.from else {
        return Ok(());
    };
    match &from.next {
        Some(next) if !e.who.is_human() && !next.contains(&e.to.id) => Err(refuse(format!(
            "{} cannot go from {} to {}; from {} it goes to {}.",
            label(e.item),
            from.id,
            e.to.id,
            from.id,
            next.join(", ")
        ))),
        _ => Ok(()),
    }
}

fn requires(e: &Entry) -> Result<(), BoardError> {
    for field in &e.to.requires {
        if e.item["fields"][field.as_str()].is_null() {
            return Err(refuse(format!(
                "{} cannot move to {}: set {field} first.",
                label(e.item),
                e.to.id
            )));
        }
    }
    Ok(())
}

/// The conditions the stage names: its gates, and the done rule when the stage is in the done
/// lane.
fn needs_of<'a>(e: &Entry<'a>) -> Vec<(&'a str, Mode)> {
    let mut all: Vec<(&str, Mode)> =
        e.to.gates
            .iter()
            .map(|g| (g.need.as_str(), g.mode))
            .collect();
    let rule = e.ty.done.as_ref().filter(|_| e.to.lane == "done");
    if let Some(rule) = rule {
        all.extend(rule.needs.iter().map(|n| (n.as_str(), rule.mode)));
    }
    all
}

fn findings(e: &Entry) -> Vec<(Mode, String)> {
    let subject = Subject {
        board: e.board,
        item: e.item,
        who: e.who,
    };
    needs_of(e)
        .into_iter()
        .flat_map(|(need, mode)| {
            missing(need, &subject)
                .into_iter()
                .map(move |text| (mode, text))
        })
        .collect()
}

fn settle(e: &Entry, found: Vec<(Mode, String)>) -> Result<Entered, BoardError> {
    let (hard, soft): (Vec<(Mode, String)>, Vec<(Mode, String)>) = found
        .into_iter()
        .partition(|(mode, _)| *mode == Mode::Refuse);
    let warnings: Vec<String> = soft.into_iter().map(|(_, text)| text).collect();
    let hard: Vec<String> = hard.into_iter().map(|(_, text)| text).collect();
    if hard.is_empty() || e.force.is_some() {
        return Ok(Entered {
            warnings,
            overridden: hard,
        });
    }
    Err(refuse(format!(
        "{} cannot move to {}: {}. Fix that, or pass force with a reason.",
        label(e.item),
        e.to.id,
        hard.join("; ")
    )))
}

/// Check a move into a stage and say what it let through.
///
/// A move into the stage the item is in changes nothing and checks nothing.
pub(crate) fn enter(e: &Entry) -> Result<Entered, BoardError> {
    if e.from.is_some_and(|f| f.id == e.to.id) {
        return Ok(Entered::default());
    }
    flow(e)?;
    requires(e)?;
    settle(e, findings(e))
}
