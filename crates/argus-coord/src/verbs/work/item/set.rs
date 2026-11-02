//! `coord item set <I> [--title] [--desc] [--subject] [--status] [--if-rev n]`,
//! `coord item status <I> <s>`, `coord item block <I> --on "<why>"`, `coord item unblock <I>` and `coord item move <I> [--to T] [--status s] [placement]`.

use argus_board::json::display;
use argus_board::limits::require_text;
use argus_board::text::slug;
use serde_json::{Map, Value, json};

use super::add::check_status;
use crate::cli::Outcome;
use crate::verbs::work::crud::{apply, checked, find_doc, move_doc, move_target, op, rev_of};
use crate::verbs::work::place::has_placement;
use crate::verbs::work::run::{Res, Run, bad};

fn fields_of(r: &Run<'_>, status: Option<&str>) -> Res<Map<String, Value>> {
    let mut fields = Map::new();
    if let Some(title) = r.text_opt("title") {
        fields.insert("title".into(), json!(require_text("title", &json!(title))?));
    }
    if let Some(desc) = r.text_opt("desc") {
        let value = if argus_core::js::trim(desc).is_empty() {
            String::new()
        } else {
            require_text("description", &json!(desc))?
        };
        fields.insert("description".into(), json!(value));
    }
    if let Some(subject) = r.text_opt("subject") {
        fields.insert(
            "subject".into(),
            json!(slug(&require_text("subject", &json!(subject))?)),
        );
    }
    match (status, r.options.json("status")) {
        (Some(given), _) => fields.insert("status".into(), json!(given)),
        (None, Some(given)) => fields.insert("status".into(), json!(check_status(&given)?)),
        (None, None) => None,
    };
    if fields.is_empty() {
        return Err(bad(
            "nothing to change: pass --title, --desc, --subject or --status.",
        ));
    }
    Ok(fields)
}

/// Sets `fields` on the item the first word names; `said` turns its id and new rev into the line.
fn edit(
    r: &mut Run<'_>,
    fields: &Map<String, Value>,
    said: impl Fn(&str, &str) -> String,
) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let id = r.arg(0);
        checked(board, &who, ("items", "set"), id)?;
        let id = id.unwrap_or_default().to_owned();
        let operation = op(
            ("edit", "items", &id),
            Map::from_iter([("set".to_owned(), Value::Object(fields.clone()))]),
            r.options,
        )?;
        apply(board, &who, &operation)?;
        let rev = rev_of(board, "items", &id);
        r.say(&[&said(&id, &rev)]);
        Ok(())
    })
}

fn change(r: &mut Run<'_>, status: Option<&str>) -> Outcome {
    let fields = fields_of(r, status)?;
    let named: Vec<&str> = fields.keys().map(String::as_str).collect();
    let named = named.join(", ");
    edit(r, &fields, |id, rev| {
        format!("{id}: {named} updated (rev {rev}).")
    })
}

/// `coord item set`.
pub(super) fn set(r: &mut Run<'_>) -> Outcome {
    change(r, None)
}

/// `coord item status <I> <s>`: the same as `set` with the status checked first.
pub(super) fn status(r: &mut Run<'_>) -> Outcome {
    let given = check_status(&r.arg_json(1))?;
    change(r, Some(&given))
}

/// Sets or clears the blocker of the item the first word names.
fn set_blocker(r: &mut Run<'_>, value: Value) -> Outcome {
    let line = value.as_str().map_or_else(
        || "blocker cleared.".to_owned(),
        |on| format!("blocked on {on}."),
    );
    let fields = Map::from_iter([("blocker".to_owned(), value)]);
    edit(r, &fields, |id, _| format!("{id}: {line}"))
}

/// `coord item block <I> --on "<D-n | Q-n | external: who, what>"`.
pub(super) fn block(r: &mut Run<'_>) -> Outcome {
    let on = r
        .text_opt("on")
        .filter(|t| !argus_core::js::trim(t).is_empty())
        .map(str::to_owned);
    let Some(on) = on else {
        return Err(bad(
            "usage: coord item block <I-n> --on \"<D-n | Q-n | external: who, what>\".",
        )
        .into());
    };
    set_blocker(r, json!(on))
}

/// `coord item unblock <I>`.
pub(super) fn unblock(r: &mut Run<'_>) -> Outcome {
    set_blocker(r, Value::Null)
}

/// `coord item move`.
pub(super) fn move_to(r: &mut Run<'_>) -> Outcome {
    let mut to = move_target(r.options);
    if let Some(given) = r.options.json("status") {
        to.insert("status".into(), json!(check_status(&given)?));
    }
    if to.is_empty() && !has_placement(r.options) {
        return Err(bad(
            "say where: --to <thread>, --status <s>, --before|--after <item>, --top or --bottom.",
        )
        .into());
    }
    r.write(|board, r| {
        let who = r.begin(board)?;
        let (_, id) = move_doc(board, &who, ("items", r.arg(0)), (to, r.options))?;
        let now = find_doc(board, "items", Some(&id))?;
        r.say(&[&format!(
            "{id}: {}, {} (rev {}).",
            display(&now["thread"]),
            display(&now["status"]),
            display(&now["rev"])
        )]);
        Ok(())
    })
}
