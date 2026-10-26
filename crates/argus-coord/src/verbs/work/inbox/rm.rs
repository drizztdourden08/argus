//! `coord inbox rm <m-id> [--for T]` and `coord inbox rm --all [--for T]`.

use argus_board::json::map_mut;
use serde_json::Value;

use super::{target, waiting};
use crate::cli::Outcome;
use crate::verbs::work::run::{Run, bad};

/// `coord inbox rm`: drops one message, or all of them.
pub(super) fn rm(r: &mut Run<'_>) -> Outcome {
    let id = r.arg(0).map(str::to_owned);
    let all = r.options.flag("all");
    if id.is_none() && !all {
        return Err(bad("name the message (coord inbox rm <m-id>) or pass --all.").into());
    }
    r.write(|board, r| {
        let who = r.begin(board)?;
        let thread = target(board, &who, r, "rm")?;
        let before = waiting(board, &thread);
        let kept: Vec<Value> = if all {
            Vec::new()
        } else {
            before
                .iter()
                .filter(|m| m["id"].as_str() != id.as_deref())
                .cloned()
                .collect()
        };
        let removed = before.len() - kept.len();
        map_mut(board, "inbox").insert(thread.clone(), Value::Array(kept));
        if removed == 0 {
            let named = id.as_deref().unwrap_or("undefined");
            return Err(bad(format!("{thread} has no message \"{named}\".")));
        }
        r.say(&[&format!("{thread}: {removed} message(s) removed.")]);
        Ok(())
    })
}
