//! What the flags of `coord launcher set` name, nothing else (`patchFrom`): `""` clears a text
//! field, `--current` / `--not-current` and `--window` / `--no-window` are yes or no pairs.

use argus_launchers::launcher::{EVERY_HELP, KINDS, LauncherPatch, parse_every};

use crate::cli::Options;
use crate::cli::exit::CoordError;
use crate::verbs::agent::base::opt::{Text, given, is_list, js_string, not_one_text, one_text};
use crate::verbs::agent::rules::refused;

const OFF: [&str; 4] = ["false", "0", "no", "off"];

/// A yes or no flag and its opposite: `--x` (or `--x=false`) and `--no-x`. `None` when neither
/// is given.
fn on_off(options: &Options, on: &str, off: &str) -> Result<Option<bool>, CoordError> {
    let (yes, no) = (given(options, on), given(options, off));
    if !yes && !no {
        return Ok(None);
    }
    if yes && no {
        return Err(CoordError::refuse(format!(
            "pass --{on} or --{off}, not both."
        )));
    }
    if is_list(options, on) || is_list(options, off) {
        return Err(CoordError::refuse(format!("--{on} takes no value.")));
    }
    Ok(Some(
        !no && !OFF.contains(&js_string(options, on).to_lowercase().as_str()),
    ))
}

fn kind_flag(options: &Options) -> Result<Option<String>, CoordError> {
    if !given(options, "kind") {
        return Ok(None);
    }
    let kind = js_string(options, "kind").trim().to_owned();
    if KINDS.contains(&kind.as_str()) {
        return Ok(Some(kind));
    }
    Err(CoordError::refuse(format!(
        "--kind must be one of {}.",
        KINDS.join(", ")
    )))
}

fn every_flag(options: &Options) -> Result<Text, CoordError> {
    if !given(options, "use-every") {
        return Ok(None);
    }
    if not_one_text(options, "use-every") {
        return Err(CoordError::refuse(format!(
            "--use-every takes one duration: {EVERY_HELP}."
        )));
    }
    let every = parse_every(&js_string(options, "use-every")).map_err(refused)?;
    Ok(Some(Some(every.text)))
}

/// The fields the flags name.
///
/// # Errors
/// A flag with no value or too many, a text over its limit, a kind or a duration that is not
/// one, a yes and its no together.
pub fn patch_from(options: &Options) -> Result<LauncherPatch, CoordError> {
    Ok(LauncherPatch {
        title: one_text(options, "title", "title", true)?,
        cmd: one_text(options, "cmd", "launchercmd", false)?,
        cwd: one_text(options, "cwd", "launchercwd", true)?,
        branch: one_text(options, "branch", "branch", true)?,
        commit: one_text(options, "commit", "commit", true)?,
        note: one_text(options, "note", "launchernote", true)?,
        kind: kind_flag(options)?,
        current: on_off(options, "current", "not-current")?,
        window: on_off(options, "window", "no-window")?,
        use_every: every_flag(options)?,
    })
}
