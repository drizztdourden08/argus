//! `coord item set <I> [--title] [--desc] [--status] [--type] [--parent] [--assignee] [--stage]
//! [--field k=v]... [--force "<reason>"] [--if-rev n]`, `coord item status <I> <lane|stage>
//! [--force "<reason>"]` and `coord item move <I> [--to T] [--status s] [placement]`.

use argus_board::json::display;
use argus_board::limits::require_text;
use argus_board::registry::registry;
use serde_json::{Map, Value, json};

use super::add::check_status;
use super::edit::{Say, edit_item, fixed};
use super::images::{Pending, picked};
use super::{STATUSES, typed};
use crate::cli::Outcome;
use crate::verbs::work::crud::{find_doc, move_doc, move_target};
use crate::verbs::work::place::has_placement;
use crate::verbs::work::run::{Res, Run, bad};

const NOTHING: &str = "nothing to change: pass --title, --desc, --status, --type, --parent, --assignee, --stage or --field.";

fn texts(r: &Run<'_>, fields: &mut Map<String, Value>) -> Res<()> {
    if let Some(title) = r.text_opt("title") {
        fields.insert("title".into(), json!(require_text("title", &json!(title))?));
    }
    if let Some(desc) = r.text_opt("desc") {
        let value = if argus_core::js::trim(desc).is_empty() {
            ""
        } else {
            desc
        };
        fields.insert("description".into(), json!(value));
    }
    Ok(())
}

/// The lane or the stage a word names: the key to set and its value.
fn place_of(given: &Value) -> Res<(&'static str, Value)> {
    let word = given.as_str().unwrap_or_default();
    let is_stage = !STATUSES.contains(&word)
        && registry()
            .types
            .iter()
            .any(|t| t.stages.iter().any(|s| s.id == word));
    if is_stage {
        return Ok(("stage", json!(word)));
    }
    check_status(given).map(|lane| ("status", json!(lane)))
}

fn fields_of(
    r: &Run<'_>,
    place: Option<(&str, Value)>,
    pictures: &[Pending],
) -> Res<Map<String, Value>> {
    let mut fields = Map::new();
    texts(r, &mut fields)?;
    fields.extend(typed::flags(r)?);
    match (place, r.options.json("status")) {
        (Some((key, value)), _) => fields.insert(key.into(), value),
        (None, Some(given)) => fields.insert("status".into(), json!(check_status(&given)?)),
        (None, None) => None,
    };
    if fields.is_empty() && pictures.is_empty() {
        return Err(bad(NOTHING));
    }
    Ok(fields)
}

fn change(r: &mut Run<'_>, place: Option<(&str, Value)>, first_caption: usize) -> Outcome {
    let pictures = picked(r, first_caption)?;
    let fields = fields_of(r, place, &pictures)?;
    edit_item(r, &pictures, fixed(fields, Say::Updated))
}

/// `coord item set`.
pub(super) fn set(r: &mut Run<'_>) -> Outcome {
    change(r, None, 1)
}

/// `coord item status <I> <lane|stage>`: the same as `set` with the lane or stage checked first.
pub(super) fn status(r: &mut Run<'_>) -> Outcome {
    let place = place_of(&r.arg_json(1))?;
    change(r, Some(place), 2)
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
