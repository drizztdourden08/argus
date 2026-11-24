//! `coord ping "<text>" [--title "<t>"] [--level info|ping|urgent]`: ping the owner (Argus >
//! Notifications, and an OS toast). Any registered thread may ping, the Overlord too.

use argus_board::limits::{require_text, require_text_max};
use argus_board::names::display_name;
use argus_board::ops::Role;
use argus_core::js;
use argus_notify::{Level, Outcome as Pinged, ToastState, sentence};
use serde_json::{Value, json};

use super::support::link::{connect, project_of};
use super::support::scope::{coord_home, require_board};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::who::caller;

const LEVELS: [&str; 3] = ["info", "ping", "urgent"];
const TITLE_MAX: usize = 60;

/// `--level` as typed, `ping` by default; a flag or a list is read as JavaScript would.
fn level_of(call: &Call<'_>) -> Result<(String, Level), CoordError> {
    let typed = call
        .options
        .json("level")
        .map_or_else(|| "ping".to_owned(), |v| js::string(&v));
    let level = match typed.as_str() {
        "info" => Level::Info,
        "ping" => Level::Ping,
        "urgent" => Level::Urgent,
        _ => {
            return Err(CoordError::refuse(format!(
                "--level must be one of {}, not \"{typed}\".",
                LEVELS.join(", ")
            )));
        }
    };
    Ok((typed, level))
}

fn toast_of(answer: &Value) -> ToastState {
    match answer["toast"].as_str() {
        Some("shown") => ToastState::Shown,
        Some("limited") => ToastState::Limited,
        Some("failed") => ToastState::Failed,
        _ => ToastState::Skipped,
    }
}

fn pinged(answer: &Value) -> Pinged {
    Pinged {
        notification: None,
        toast: toast_of(answer),
        why: answer["why"].as_str().map(str::to_owned),
    }
}

/// The title: `--title` when given, else `<Project>: <thread>`.
fn title_of(call: &Call<'_>, board_name: Option<&str>, thread: &str) -> Result<String, CoordError> {
    if let Some(title) = call
        .options
        .text("title")
        .filter(|t| !js::trim(t).is_empty())
    {
        return Ok(require_text_max("title", &json!(title), TITLE_MAX)?);
    }
    let named = display_name(&coord_home(call), project_of(call), board_name, None);
    Ok(js::slice(&format!("{named}: {thread}"), TITLE_MAX).to_owned())
}

/// `coord ping`.
///
/// # Errors
/// A refusal, or the dashboard's own.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let text = require_text("ping", &json!(call.positional.first()))?;
    let (level_name, level) = level_of(call)?;
    let project = project_of(call).to_owned();
    let board = require_board(call, &project)?;
    let who = caller(&board, &call.inv.cwd, &call.inv.env, &call.options)?;
    let title = title_of(call, board["name"].as_str(), &who.thread)?;
    let by =
        if argus_board::roles::is_overlord(&coord_home(call), &project) && who.role == Role::Lead {
            "overlord"
        } else {
            "thread"
        };
    let client = connect(call, true)?;
    let body = json!({
        "title": title, "text": text, "level": level_name,
        "project": project, "thread": who.thread, "by": by,
    });
    let answer = client.post("/api/notify", &body)?;
    call.inv.say(&[&sentence(&pinged(&answer), &title, level)]);
    Ok(())
}
