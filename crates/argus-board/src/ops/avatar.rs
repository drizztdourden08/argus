//! The `avatar` op: the picture a project's lead wears on the cards. A change at the lead's
//! level: the board holds `avatar`, either `brand:<app>` (a Tessera brand mark), `icon:<name>` (a curated icon) or an image as a
//! data URL no larger than [`AVATAR_MAX`], and one audit line records it. `null` clears it.

use serde_json::Value;

use super::{
    announce::audit,
    authorize::{Check, authorize},
    icon::ICON_NAMES,
    icon_image::text_or_clear,
    result::OpResult,
    who::Who,
};
use crate::{
    Board,
    error::{BoardError, refuse},
    json::{nullish, set},
};

/// The most characters an uploaded avatar may take as a data URL. An owner post is at most 256 KB
/// in all, so this leaves room for the rest of the entry.
const AVATAR_MAX: usize = 240 * 1024;

/// The Tessera apps whose brand mark may be picked.
pub(crate) const BRAND_APPS: [&str; 6] =
    ["tessera", "rotp", "archipelia", "brock", "bxel", "argus"];

const IMAGE_PREFIXES: [&str; 4] = [
    "data:image/png;base64,",
    "data:image/jpeg;base64,",
    "data:image/webp;base64,",
    "data:image/gif;base64,",
];

fn is_base64(body: &str) -> bool {
    !body.is_empty()
        && body
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'='))
}

fn check_image(text: &str) -> Result<(), BoardError> {
    let Some(prefix) = IMAGE_PREFIXES.iter().find(|p| text.starts_with(**p)) else {
        return Err(refuse(
            "avatar must be brand:<app> or a png, jpeg, webp or gif data URL.",
        ));
    };
    if text.len() > AVATAR_MAX {
        return Err(refuse(format!(
            "avatar image is {} chars, the limit is {AVATAR_MAX}. Use a smaller image.",
            text.len()
        )));
    }
    if is_base64(&text[prefix.len()..]) {
        Ok(())
    } else {
        Err(refuse("avatar image is not valid base64."))
    }
}

/// The checked avatar: `Ok(None)` clears it.
///
/// # Errors
/// Refused when the value is neither a brand, an image data URL within the cap, nor `null`.
fn check(value: &Value) -> Result<Option<String>, BoardError> {
    let Some(text) = text_or_clear(value, "avatar")? else {
        return Ok(None);
    };
    if let Some(app) = text.strip_prefix("brand:") {
        return if BRAND_APPS.contains(&app) {
            Ok(Some(text.to_owned()))
        } else {
            Err(refuse(format!(
                "avatar brand must be one of {}.",
                BRAND_APPS.join(", ")
            )))
        };
    }
    if let Some(name) = text.strip_prefix("icon:") {
        return if ICON_NAMES.contains(&name) {
            Ok(Some(text.to_owned()))
        } else {
            Err(refuse("avatar icon is not a known icon name."))
        };
    }
    check_image(text).map(|()| Some(text.to_owned()))
}

/// Set or clear the lead's avatar.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    authorize(who, &Check::new("hand", "threads", None))?;
    let avatar = check(&op["avatar"])?;
    if avatar == nullish(board, "avatar") {
        return Ok(OpResult::applied("the avatar is unchanged"));
    }
    let what = if avatar.is_some() { "set" } else { "cleared" };
    set(board, "avatar", avatar);
    audit(
        board,
        &who.actor_of(),
        &format!("{} {what} the avatar", who.label()),
    );
    Ok(OpResult::applied(format!("avatar {what}")))
}
