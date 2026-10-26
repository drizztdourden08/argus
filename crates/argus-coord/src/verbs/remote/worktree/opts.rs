//! The option forms of the worktree verbs: one value, or a repeatable list.

use crate::cli::exit::CoordError;
use crate::cli::{Opt, Options, Val};

/// `textOpt`: the value of an option that takes one value, `None` when it is not given.
///
/// # Errors
/// The option is a bare flag or was given twice.
pub(super) fn text_opt(options: &Options, name: &str) -> Result<Option<String>, CoordError> {
    match options.get(name) {
        None => Ok(None),
        Some(Opt::One(Val::Text(text))) => Ok(Some(text.clone())),
        Some(_) => {
            let clears = if name == "note" {
                " (\"\" clears it)"
            } else {
                ""
            };
            Err(CoordError::refuse(format!(
                "--{name} takes one value{clears}."
            )))
        }
    }
}

/// `listOpt`: the values of a repeatable name pattern, `None` when it is not given.
///
/// # Errors
/// One of the values is a bare flag.
pub(super) fn list_opt(options: &Options, name: &str) -> Result<Option<Vec<String>>, CoordError> {
    let list = match options.get(name) {
        None => return Ok(None),
        Some(Opt::One(val)) => vec![val.clone()],
        Some(Opt::Many(list)) => list.clone(),
    };
    list.into_iter()
        .map(|val| match val {
            Val::Text(text) => Ok(text),
            Val::Flag => Err(CoordError::refuse(format!(
                "--{name} takes a name pattern such as \"web-*\"."
            ))),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}
