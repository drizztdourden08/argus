//! The owner's profile: a `profile` post sets or clears the avatar, display name and initials the
//! Argus page wears. It is applied here, never handed down to a project. The portfolio board holds
//! `owner`; the entry is kept `applied`, or `refused` with the reason.

use argus_core::config::hex;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::Board;
use crate::json::{map_mut, set, unset};
use crate::ops::{BRAND_APPS, check_image, is_image_icon};
use crate::store::{LogLine, push_log_line, touch};

const NAME_MAX: usize = 40;

type Check = fn(&Value) -> Result<String, String>;

fn text_of<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("{field} must be a text or null."))
}

fn check_avatar(value: &Value) -> Result<String, String> {
    let text = text_of(value, "avatar")?;
    if text == "initials" {
        return Ok(text.to_owned());
    }
    if is_image_icon(text) {
        return check_image(text).map(|()| text.to_owned());
    }
    match text.strip_prefix("brand:") {
        Some(app) if BRAND_APPS.contains(&app) => Ok(text.to_owned()),
        Some(_) => Err(format!(
            "avatar brand must be one of {}.",
            BRAND_APPS.join(", ")
        )),
        None => Err("avatar must be initials, brand:<app>, or a png or svg data URL.".to_owned()),
    }
}

fn check_name(value: &Value) -> Result<String, String> {
    let name = text_of(value, "name")?.trim();
    let size = name.chars().count();
    if (1..=NAME_MAX).contains(&size) {
        Ok(name.to_owned())
    } else {
        Err(format!("name must be 1 to {NAME_MAX} characters."))
    }
}

fn check_initials(value: &Value) -> Result<String, String> {
    let text = text_of(value, "initials")?;
    let fits = text.chars().count() == 2
        && text.chars().all(|c| c.is_alphanumeric() && !c.is_lowercase());
    if fits {
        Ok(text.to_owned())
    } else {
        Err("initials must be two upper case letters or digits.".to_owned())
    }
}

const FIELDS: [(&str, Check); 3] = [
    ("avatar", check_avatar),
    ("name", check_name),
    ("initials", check_initials),
];

/// Checks every field the post names, then sets them on the owner. Nothing changes on a refusal.
fn apply(p: &mut Board, entry: &Value) -> Result<(), String> {
    let mut owner = p
        .get("owner")
        .filter(|o| o.is_object())
        .cloned()
        .unwrap_or_else(|| json!({}));
    let mut changed = Vec::new();
    for (field, check) in FIELDS {
        let Some(value) = entry.get(field) else {
            continue;
        };
        if value.is_null() {
            unset(&mut owner, field);
        } else {
            set(&mut owner, field, check(value)?);
        }
        changed.push(field);
    }
    if changed.is_empty() {
        return Err("a profile post needs avatar, name or initials.".to_owned());
    }
    if owner.as_object().is_some_and(serde_json::Map::is_empty) {
        unset(p, "owner");
    } else {
        set(p, "owner", owner);
    }
    let text = format!("the owner changed the profile: {}", changed.join(", "));
    let line = LogLine {
        thread: "portfolio",
        item: None,
        text: &text,
    };
    push_log_line(p, "portfolio", &line, None);
    Ok(())
}

/// Applies a profile post and keeps it as `applied`, or `refused` with its `reason`. Returns the
/// status.
pub(super) fn import(p: &mut Board, id: &str, mut entry: Value) -> String {
    let status = match apply(p, &entry) {
        Ok(()) => "applied",
        Err(reason) => {
            set(&mut entry, "reason", reason);
            "refused"
        }
    };
    set(&mut entry, "status", status);
    map_mut(p, "entries").insert(id.to_owned(), entry);
    touch(p, "entries", id);
    status.to_owned()
}

/// The owner as the page receives it: an image avatar becomes the url that serves it, carrying a
/// short hash of the image. `None` while no profile field is set.
pub(crate) fn export_owner(board: &Board) -> Option<Value> {
    let mut owner = board.get("owner").filter(|o| o.is_object())?.clone();
    if owner.as_object().is_some_and(serde_json::Map::is_empty) {
        return None;
    }
    let image = owner["avatar"].as_str().filter(|a| is_image_icon(a));
    if let Some(image) = image {
        let digest = hex(&Sha256::digest(image.as_bytes()));
        let url = format!("/api/owner/avatar?v={}", &digest[..8]);
        set(&mut owner, "avatar", url);
    }
    Some(owner)
}
