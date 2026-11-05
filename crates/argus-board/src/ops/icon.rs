//! The `icon` op: the picture a board wears in the shell. A change at the coordinator's level: the
//! board holds `icon`, one of `brand:<app>` (a Tessera brand mark), `icon:<name>` (a name of
//! [`ICON_NAMES`]) or a png or svg data URL no larger than the icon cap, and one audit line records
//! it. `null` clears it.

use serde_json::Value;

use super::{
    announce::audit,
    authorize::{Check, authorize},
    icon_image::{check_image, is_image_icon},
    result::OpResult,
    who::Who,
};
use crate::{
    Board,
    error::{BoardError, refuse},
    json::{nullish, set},
};

/// The Tessera apps whose brand mark may be picked.
const BRAND_APPS: [&str; 6] = ["tessera", "rotp", "archipelia", "brock", "bxel", "argus"];

/// The curated icon names. `apps/web/src/board-icon/board-icon.constants.ts` lists the same names.
pub const ICON_NAMES: [&str; 16] = [
    "gamepad-2",
    "palette",
    "flask",
    "book",
    "globe",
    "database",
    "code",
    "cpu",
    "package",
    "compass",
    "layers",
    "network",
    "music",
    "camera",
    "heart",
    "zap",
];

fn check_named(text: &str) -> Option<Result<(), BoardError>> {
    if let Some(app) = text.strip_prefix("brand:") {
        return Some(if BRAND_APPS.contains(&app) {
            Ok(())
        } else {
            Err(refuse(format!(
                "icon brand must be one of {}.",
                BRAND_APPS.join(", ")
            )))
        });
    }
    let name = text.strip_prefix("icon:")?;
    Some(if ICON_NAMES.contains(&name) {
        Ok(())
    } else {
        Err(refuse(format!(
            "icon name must be one of {}.",
            ICON_NAMES.join(", ")
        )))
    })
}

/// The checked icon: `Ok(None)` clears it.
///
/// # Errors
/// Refused when the value is none of a brand, a curated name, a safe image data URL or `null`.
fn check(value: &Value) -> Result<Option<String>, BoardError> {
    let Some(text) = value.as_str() else {
        return if value.is_null() {
            Ok(None)
        } else {
            Err(refuse("icon must be a text or null."))
        };
    };
    if let Some(done) = check_named(text) {
        return done.map(|()| Some(text.to_owned()));
    }
    if !is_image_icon(text) {
        return Err(refuse(
            "icon must be brand:<app>, icon:<name>, or a png or svg data URL.",
        ));
    }
    check_image(text).map_err(refuse)?;
    Ok(Some(text.to_owned()))
}

/// Set or clear the board's icon.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    authorize(who, &Check::new("hand", "threads", None))?;
    let icon = check(&op["icon"])?;
    if icon == nullish(board, "icon") {
        return Ok(OpResult::applied("the icon is unchanged"));
    }
    let what = if icon.is_some() { "set" } else { "cleared" };
    set(board, "icon", icon);
    audit(
        board,
        &who.actor_of(),
        &format!("{} {what} the icon", who.label()),
    );
    Ok(OpResult::applied(format!("icon {what}")))
}
