//! `board name` and `board prefixes`: how the sessions of a project are titled.

use argus_board::json::{display, set, unset};
use argus_board::names::session_name;
use argus_core::boards;
use argus_core::js;
use serde_json::json;

use crate::cli::exit::{CoordError, Outcome};
use crate::who::refuse_while_migrating;
use crate::verbs::thread::docs::list;
use crate::verbs::thread::frame::Frame;

const NAME_MAX: usize = 30;

/// A board of the identity table is renamed there; its aliases stay.
fn rename_in_table(frame: &mut Frame<'_>, value: &str) -> Outcome {
    let (home, project) = (frame.home(), frame.project);
    refuse_while_migrating(home)?;
    boards::rename(home, project, value).map_err(|e| CoordError::refuse(e.to_string()))?;
    frame.say(&[&format!(
        "{project} is shown as \"{value}\". Sessions: the lead \"{value}\" and \"{value} - <Subject>\"."
    )]);
    Ok(())
}

/// The verb `board name <Name>`.
///
/// # Errors
/// Refused for an empty name or one over 30 characters, and no board.
pub fn name(frame: &mut Frame<'_>) -> Outcome {
    let value = js::trim(frame.arg(0).unwrap_or("")).to_owned();
    if value.is_empty() || js::len(&value) > NAME_MAX {
        return Err(CoordError::refuse(
            "pass the project display name, 30 characters at most (e.g. \"RotP\").",
        ));
    }
    let project = frame.project;
    if boards::load(frame.home()).is_ok_and(|t| t.boards.contains_key(project)) {
        return rename_in_table(frame, &value);
    }
    frame.mutate(false, |board, fr| {
        set(board, "name", value.as_str());
        fr.say(&[&format!(
            "{project} is shown as \"{value}\". Sessions: the lead \"{value}\" and \"{value} - <Subject>\"."
        )]);
        Ok(())
    })
}

/// The verb `board prefixes [<Product>...] [--clear]`.
///
/// # Errors
/// Refused for a prefix over 30 characters, and no board.
pub fn prefixes(frame: &mut Frame<'_>) -> Outcome {
    let mut values: Vec<String> = Vec::new();
    for word in frame
        .args
        .iter()
        .map(|v| js::trim(v))
        .filter(|v| !v.is_empty())
    {
        if js::len(word) > NAME_MAX {
            return Err(CoordError::refuse(
                "each thread-title prefix is 30 characters at most.",
            ));
        }
        if !values.iter().any(|v| v == word) {
            values.push(word.to_owned());
        }
    }
    let clear = frame.options.json("clear").is_some_and(|v| js::truthy(&v));
    let project = frame.project;
    frame.mutate(false, |board, fr| {
        if !values.is_empty() {
            set(board, "prefixes", json!(values));
        } else if clear {
            unset(board, "prefixes");
        }
        let listed: Vec<String> = list(&board["prefixes"]).iter().map(display).collect();
        let shown = if listed.is_empty() {
            vec![session_name(
                &fr.paths.coord_home,
                project,
                board["name"].as_str(),
            )]
        } else {
            listed
        };
        let titles: Vec<String> = shown
            .iter()
            .map(|p| format!("\"{p} - <Subject>\""))
            .collect();
        fr.say(&[&format!(
            "thread titles on {project} start with: {}.",
            titles.join(", ")
        )]);
        Ok(())
    })
}
