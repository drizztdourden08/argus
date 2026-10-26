//! `coord item set <I> [--title] [--desc] [--subject] [--status] [--if-rev n]`,
//! `coord item status <I> <s>` and `coord item move <I> [--to T] [--status s] [placement]`.

use argus_board::json::display;
use argus_board::limits::require_text;
use argus_board::ops::{Check, authorize};
use argus_board::text::slug;
use serde_json::{Map, Value, json};

use super::add::check_status;
use crate::cli::Outcome;
use crate::verbs::work::crud::{
    apply, assert_live, find_doc, move_op, op, rev_of, thread_arg, who_of,
};
use crate::verbs::work::place::{has_placement, rank_to};
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
        fields.insert("subject".into(), json!(slug(&require_text("subject", &json!(subject))?)));
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

fn change(r: &mut Run<'_>, status: Option<&str>) -> Outcome {
    let fields = fields_of(r, status)?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let id = r.arg(0);
        let item = find_doc(board, "items", id)?;
        assert_live(&item, "items")?;
        authorize(&who_of(&who), &Check::new("set", "items", Some(&item)))?;
        let id = id.unwrap_or_default().to_owned();
        let named: Vec<&str> = fields.keys().map(String::as_str).collect();
        let operation = op(("edit", "items", &id), Map::from_iter([("set".to_owned(), Value::Object(fields.clone()))]), r.options)?;
        apply(board, &who, &operation)?;
        r.say(&[&format!("{id}: {} updated (rev {}).", named.join(", "), rev_of(board, "items", &id))]);
        Ok(())
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

/// `coord item move`.
pub(super) fn move_to(r: &mut Run<'_>) -> Outcome {
    let mut to = Map::new();
    if let Some(thread) = r.text_opt("to") {
        to.insert("thread".into(), json!(thread));
    }
    if let Some(given) = r.options.json("status") {
        to.insert("status".into(), json!(check_status(&given)?));
    }
    if to.is_empty() && !has_placement(r.options) {
        return Err(bad("say where: --to <thread>, --status <s>, --before|--after <item>, --top or --bottom.").into());
    }
    r.write(|board, r| {
        let who = r.begin(board)?;
        let id = r.arg(0);
        let item = find_doc(board, "items", id)?;
        assert_live(&item, "items")?;
        let id = id.unwrap_or_default().to_owned();
        let thread = to.get("thread").and_then(Value::as_str).map(str::to_owned);
        if let Some(name) = &thread {
            thread_arg(board, name)?;
        }
        let check = Check::new("move", "items", Some(&item)).to(thread.as_deref());
        authorize(&who_of(&who), &check)?;
        if let Some(place) = rank_to(board, ("items", &id), r.options)? {
            to.extend(place);
        }
        apply(board, &who, &move_op("items", &id, to.clone(), r.options)?)?;
        let now = find_doc(board, "items", Some(&id))?;
        r.say(&[&format!("{id}: {}, {} (rev {}).", display(&now["thread"]), display(&now["status"]), display(&now["rev"]))]);
        Ok(())
    })
}
