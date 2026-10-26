//! The `--confirm <project key>` of a purge and a restore: true with a matching key, false
//! without one, and a wrong key stops the call.

use crate::cli::exit::CoordError;
use crate::cli::{Opt, Val};
use crate::verbs::thread::frame::Frame;

fn shown(val: &Val) -> String {
    match val {
        Val::Flag => "true".to_owned(),
        Val::Text(text) => text.clone(),
    }
}

/// What a wrong `--confirm` is named as: a bare flag is empty, a list is joined by commas.
fn given(opt: &Opt) -> String {
    match opt {
        Opt::One(Val::Flag) => String::new(),
        Opt::One(val) => shown(val),
        Opt::Many(list) => list.iter().map(shown).collect::<Vec<_>>().join(","),
    }
}

/// Whether the call carries the project key as its `--confirm`.
///
/// # Errors
/// Refused when `--confirm` is given and is not the project key.
pub fn confirmed(frame: &Frame<'_>) -> Result<bool, CoordError> {
    let Some(opt) = frame.options.get("confirm") else {
        return Ok(false);
    };
    if frame.options.text("confirm") == Some(frame.project) {
        return Ok(true);
    }
    Err(CoordError::refuse(format!(
        "--confirm must be the project key \"{}\", not \"{}\". Nothing was changed.",
        frame.project,
        given(opt)
    )))
}
