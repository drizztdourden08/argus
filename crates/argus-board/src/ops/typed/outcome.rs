//! What a typed change tells the caller besides success: the warnings of the rules that only
//! warn, and the override of a rule that refuses, which is kept as a decision on the item.

use serde_json::{Value, json};

use super::enter::Entered;
use crate::Board;
use crate::error::BoardError;
use crate::json::doc;
use crate::ops::{OpResult, Who};
use crate::store::create_doc;
use crate::text::clip_text;

const DECISION_LINE: usize = 200;
const CONTEXT_LINE: usize = 600;

/// A rule that refused, overridden with a reason.
#[derive(Debug)]
struct Forced {
    stage: String,
    overridden: Vec<String>,
    reason: String,
}

/// The warnings of a change, and the override it made.
#[derive(Debug, Default)]
pub(crate) struct Outcome {
    warnings: Vec<String>,
    forced: Option<Forced>,
}

impl Outcome {
    /// What entering `stage` let through, with the reason that forced it when there is one.
    pub(crate) fn of(entered: Entered, stage: &str, reason: Option<String>) -> Outcome {
        let forced = match reason {
            Some(reason) if !entered.overridden.is_empty() => Some(Forced {
                stage: stage.to_owned(),
                overridden: entered.overridden,
                reason,
            }),
            _ => None,
        };
        Outcome {
            warnings: entered.warnings,
            forced,
        }
    }

    /// The result with the warnings added to its sentence and listed in `warnings`.
    pub(crate) fn finish(&self, mut result: OpResult) -> OpResult {
        if self.warnings.is_empty() {
            return result;
        }
        result.outcome = format!("{} Warning: {}.", result.outcome, self.warnings.join("; "));
        result.with_extra("warnings", json!(self.warnings))
    }

    /// Keep the override as a decision on the item, so it stays visible.
    pub(crate) fn record(&self, board: &mut Board, id: &str, who: &Who) -> Result<(), BoardError> {
        let Some(forced) = &self.forced else {
            return Ok(());
        };
        let thread = doc(board, "items", id).map_or(Value::Null, |i| i["thread"].clone());
        let text = format!("Forced {id} to {}: {}", forced.stage, forced.reason);
        let context = format!("Overrode: {}", forced.overridden.join("; "));
        let entry = json!({
            "kind": "decision", "item": id, "thread": thread, "author": who.actor_of(),
            "text": clip_text(&text, DECISION_LINE), "context": clip_text(&context, CONTEXT_LINE),
            "status": "done", "replyTo": null, "deliveredAt": null,
        });
        let fields = entry.as_object().cloned().unwrap_or_default();
        create_doc(board, "entries", "D", fields, &who.actor_of())?;
        Ok(())
    }
}
