//! `coord item show <I>` and `coord item list [--mine|--for T] [--status s] [--subject s]
//! [--archived] [--deleted] [--json]`.

use argus_board::json::{display, truthy};
use argus_board::text::slug;
use serde_json::{Value, json};

use crate::cli::Outcome;
use crate::cli::out::clip;
use crate::verbs::work::crud::{docs_of, find_doc};
use crate::verbs::work::listing::{Table, cell, emit, scoped, visible};
use crate::verbs::work::run::Run;

const ITEMS: Table = Table {
    head: &["item", "status", "thread", "subject", "ac", "title"],
    empty: "no items.",
};

/// `coord item show`: the item and the entries about it.
pub(super) fn show(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let mut item = find_doc(board, "items", r.arg(0))?;
        let id = display(&item["id"]);
        let related: Vec<Value> = docs_of(board, "entries")
            .into_iter()
            .filter(|e| display(&e["item"]) == id && !truthy(e, "deletedAt"))
            .collect();
        if let Some(fields) = item.as_object_mut() {
            fields.insert("entries".into(), json!(related));
        }
        let text = serde_json::to_string_pretty(&item).unwrap_or_default();
        r.say(&[&text]);
        Ok(())
    })
}

fn row(item: &Value, _now: i64) -> Vec<String> {
    let criteria = item["ac"].as_array().map_or(&[][..], Vec::as_slice);
    let done = criteria.iter().filter(|c| truthy(c, "done")).count();
    vec![
        cell(&item["id"]),
        cell(&item["status"]),
        cell(&item["thread"]),
        cell(&item["subject"]),
        format!("{done}/{}", criteria.len()),
        clip(&cell(&item["title"]), 50),
    ]
}

/// `coord item list`.
pub(super) fn list(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let mut items: Vec<Value> = docs_of(board, "items")
            .into_iter()
            .filter(|i| visible(i, r.options))
            .collect();
        items = scoped(board, r, items)?;
        if let Some(status) = r.text_opt("status") {
            items.retain(|i| i["status"].as_str() == Some(status));
        }
        if let Some(subject) = r.text_opt("subject") {
            let wanted = slug(subject);
            items.retain(|i| i["subject"].as_str() == Some(wanted.as_str()));
        }
        emit(r, items, &ITEMS, row);
        Ok(())
    })
}
