//! `coord portfolio tick`: retry the deliveries still waiting for a board, refresh the project
//! cards, and report what is new. The dashboard posts the owner's entries itself, so nothing is
//! imported or exported here.

use std::collections::BTreeSet;

use argus_board::json::{display, map_mut, obj};
use argus_board::portfolio::{dispatch_entry, project_keys, summarize};
use argus_board::store::touch;
use argus_core::config::hex;
use serde_json::{Map, Value, json};
use sha1::{Digest, Sha1};

use super::with_portfolio;
use crate::cli::exit::Outcome;
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::remote::support::scope::coord_home;

/// The hash a card is compared by: its JSON without `updatedAt`, so a card counts as changed
/// only when something on it changed.
fn hash_of(card: &Value) -> String {
    let mut shown = card.clone();
    if let Some(map) = shown.as_object_mut() {
        map.shift_remove("updatedAt");
    }
    hex(&Sha1::digest(shown.to_string().as_bytes()))
}

fn attention_of(cards: &[Value]) -> Vec<String> {
    let mut current = Vec::new();
    for card in cards.iter().filter(|c| c["state"] != "not started") {
        let name = display(&card["name"]);
        for item in card["attention"].as_array().into_iter().flatten() {
            current.push(format!("{name}: {}", display(item)));
        }
    }
    current
}

fn refresh(p: &mut Value, cards: &[Value]) {
    let by_key: Map<String, Value> = cards
        .iter()
        .map(|c| (display(&c["key"]), c.clone()))
        .collect();
    p["cards"] = Value::Object(by_key);
    for card in cards {
        let key = display(&card["key"]);
        let hash = hash_of(card);
        if p["pushedHash"][key.as_str()] != json!(hash) {
            map_mut(p, "pushedHash").insert(key.clone(), json!(hash));
            touch(p, "cards", &key);
        }
    }
}

/// `coord portfolio tick`.
pub(super) fn tick(call: &mut Call<'_>) -> Outcome {
    if call.options.get("dir").is_some() {
        call.inv
            .warn("coord: --dir is ignored; the dashboard syncs the portfolio itself.");
    }
    let home = coord_home(call);
    with_portfolio(call, false, |p, scope| {
        let waiting: Vec<String> = obj(&p["entries"])
            .iter()
            .filter(|(_, e)| e["pendingFor"].as_array().is_some_and(|l| !l.is_empty()))
            .map(|(id, _)| id.clone())
            .collect();
        for id in waiting {
            dispatch_entry(&home, p, &id)?;
        }
        let registry = json!({ "projects": p["projects"] });
        let cards: Vec<Value> = project_keys(&home, &registry)
            .iter()
            .map(|key| summarize(&home, key, &registry, None))
            .collect();
        refresh(p, &cards);
        let current = attention_of(&cards);
        let seen: BTreeSet<String> = p["lastAttention"]
            .as_array()
            .into_iter()
            .flatten()
            .map(display)
            .collect();
        p["lastAttention"] = json!(current);
        let fresh: Vec<&String> = current.iter().filter(|a| !seen.contains(*a)).collect();
        out::as_json(
            scope.out,
            &json!({ "attention": fresh, "open": current.len() }),
        );
        Ok(())
    })
}
