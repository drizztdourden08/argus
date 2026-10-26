//! `archive | unarchive | rm | restore <id>... [--reason "..."]`: nothing is ever hard-deleted
//! here. Each id is checked, changed and announced in turn, all in one locked write.

use argus_board::limits::optional_text;
use argus_board::ops::{Check, authorize};
use serde_json::Map;

use super::crud::{apply, find_doc, label, op, who_of};
use super::group::Soft;
use super::run::{Run, bad};
use crate::cli::Outcome;

/// The soft verb, the op kind it sends and how its result reads.
const SOFT: [(&str, &str, &str); 4] = [
    ("archive", "archive", "archived"),
    ("unarchive", "unarchive", "unarchived"),
    ("rm", "delete", "moved to the trash"),
    ("restore", "restore", "restored"),
];

/// Whether a sub-verb is one of the four.
#[must_use]
pub fn is_soft(verb: &str) -> bool {
    SOFT.iter().any(|(name, ..)| *name == verb)
}

/// Runs a soft verb on the ids after it.
///
/// # Errors
/// Refused when no id is named, for an unknown id, or as `authorize` and `apply_op` refuse.
pub fn run(r: &mut Run<'_>, kind: &Soft, verb: &str) -> Outcome {
    let Some((_, op_kind, past)) = SOFT.iter().find(|(name, ..)| *name == verb) else {
        return Ok(());
    };
    let word = label(kind.coll);
    if r.args.is_empty() {
        return Err(bad(format!(
            "name the {word} to {verb}: coord {word} {verb} <id>."
        ))
        .into());
    }
    let reason = optional_text("reason", &r.opt("reason"))?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        for id in r.args.clone() {
            let found = find_doc(board, kind.coll, Some(&id))?;
            authorize(&who_of(&who), &Check::new(verb, kind.coll, Some(&found)))?;
            let mut rest = Map::new();
            if let Some(why) = &reason {
                rest.insert("reason".into(), why.as_str().into());
            }
            let operation = op((op_kind, kind.coll, &id), rest, r.options)?;
            apply(board, &who, &operation)?;
            (kind.after)(board, &found);
            let because = reason
                .as_ref()
                .map_or_else(String::new, |w| format!(" ({w})"));
            r.say(&[&format!("{id}: {past}{because}")]);
        }
        Ok(())
    })
}
