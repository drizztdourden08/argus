//! `coord portfolio send <key|all> "<text>" [--decision]`: a note or a decision handed down to
//! one project's lead, or to every project.

use argus_board::json::{display, map_mut};
use argus_board::limits::require_text;
use argus_board::portfolio::{dispatch_entry, project_keys};
use argus_board::store::next_id;
use argus_core::clock::now_iso;
use serde_json::{Value, json};

use super::with_portfolio;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::scope::coord_home;
use crate::who::board_ref;

fn names(list: &Value) -> Vec<String> {
    list.as_array()
        .map(|a| a.iter().map(display).collect())
        .unwrap_or_default()
}

/// `coord portfolio send`.
pub(super) fn send(call: &mut Call<'_>) -> Outcome {
    let kind = if call.options.flag("decision") {
        "decision"
    } else {
        "followup"
    };
    let text = require_text(kind, &json!(call.positional.get(1)))?;
    let target = call
        .positional
        .first()
        .cloned()
        .unwrap_or_else(|| "undefined".to_owned());
    let home = coord_home(call);
    let target = if target == "all" {
        target
    } else {
        board_ref(&home, &target)?
    };
    with_portfolio(call, false, |p, scope| {
        if target != "all" && !project_keys(&home, p).contains(&target) {
            return Err(CoordError::refuse(format!(
                "unknown project \"{target}\"; coord portfolio add it first."
            )));
        }
        let id = next_id(p, "S");
        let entry = json!({
            "id": id, "kind": kind, "author": "portfolio", "project": target,
            "text": text, "createdAt": now_iso(),
        });
        map_mut(p, "entries").insert(id.clone(), entry);
        dispatch_entry(&home, p, &id)?;
        let entry = &p["entries"][id.as_str()];
        let delivered = names(&entry["deliveredTo"]);
        let waiting = names(&entry["pendingFor"]);
        let to = if delivered.is_empty() {
            "none yet".to_owned()
        } else {
            delivered.join(", ")
        };
        let left = if waiting.is_empty() {
            String::new()
        } else {
            format!("; waiting for a board: {}", waiting.join(", "))
        };
        scope.say(&[&format!(
            "{id} {kind} for {target}: delivered to {to}{left}"
        )]);
        Ok(())
    })
}
