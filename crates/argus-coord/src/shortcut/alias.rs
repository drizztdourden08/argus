//! The alias hosts of the board shortcut. A url with a localhost host may carry no path, so a
//! board is a host name the dashboard answers as a loopback alias:
//!
//! - `http://<key>.localhost:<port>` is the lead's board
//! - `http://<thread>.<key>.localhost:<port>` is a thread's board
//!
//! The Overlord (the coordinator thread of project `portfolio`) gets two rows, `Argus`
//! (`http://argus.localhost:<port>`, the overview) and `Overlord`
//! (`http://overlord.localhost:<port>`, its own board). `argus` and `overlord` are reserved
//! host names: no project key of those names can be an alias, and the project `argus` (the
//! app's own repo) is `argus-app.localhost`.

use argus_board::names::{ARGUS, MAIN_KEY, OVERLORD};
use serde_json::{Value, json};

use crate::cli::exit::CoordError;

const LEAD: &str = "coordinator";
const RESERVED: [&str; 2] = ["argus", "overlord"];

fn is_label_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'
}

/// `^[a-z0-9][a-z0-9-]{0,62}$`: a project key as the router of the dashboard has it.
fn is_key_label(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && s.len() <= 63
        && chars.all(is_label_char)
}

/// `^[a-z0-9-]{1,63}$`: a thread id.
fn is_thread_label(s: &str) -> bool {
    !s.is_empty() && s.len() <= 63 && s.chars().all(is_label_char)
}

/// The host label of a project: its key, except `argus` (the app itself) whose `argus` is
/// the overview.
fn host_label(project: &str) -> Option<&str> {
    match project {
        "argus" => Some("argus-app"),
        p if RESERVED.contains(&p) => None,
        p => Some(p),
    }
}

/// The alias url of a thread's board; the lead (the thread `coordinator`) gets the short one.
///
/// # Errors
/// Refused with a sentence when the key or the thread id cannot be a host name, or the key is
/// a reserved host name.
pub fn alias_url(project: &str, thread: &str, port: u16) -> Result<String, CoordError> {
    if !is_key_label(project) {
        return Err(CoordError::refuse(format!(
            "the project key \"{project}\" cannot be a host name (lowercase letters, digits and dashes, 63 characters at most)."
        )));
    }
    let Some(label) = host_label(project) else {
        return Err(CoordError::refuse(format!(
            "the project key \"{project}\" is a reserved host name (the Overlord's argus.localhost and overlord.localhost): no shortcut for it."
        )));
    };
    if thread == LEAD {
        return Ok(format!("http://{label}.localhost:{port}"));
    }
    if !is_thread_label(thread) {
        return Err(CoordError::refuse(format!(
            "the thread id \"{thread}\" cannot be a host name (lowercase letters, digits and dashes, 63 characters at most): no shortcut for it."
        )));
    }
    Ok(format!("http://{thread}.{label}.localhost:{port}"))
}

/// Is this the Overlord, the thread that gets the two rows?
#[must_use]
pub fn is_overlord(project: &str, thread: &str) -> bool {
    project == MAIN_KEY && thread == LEAD
}

/// The configuration to merge for a thread: a lead is named `<Project> board`, any other thread
/// `<Project> board: <thread>`.
///
/// # Errors
/// As [`alias_url`].
pub fn shortcut_config(
    name: &str,
    project: &str,
    thread: &str,
    port: u16,
) -> Result<Value, CoordError> {
    let title = if thread == LEAD {
        format!("{name} board")
    } else {
        format!("{name} board: {thread}")
    };
    Ok(json!({ "name": title, "port": port, "url": alias_url(project, thread, port)? }))
}

/// The configurations to merge for a thread: one row, but the Overlord gets two (`Argus` the
/// overview, `Overlord` its own board).
///
/// # Errors
/// As [`alias_url`].
pub fn shortcut_configs(
    name: &str,
    project: &str,
    thread: &str,
    port: u16,
) -> Result<Vec<Value>, CoordError> {
    if is_overlord(project, thread) {
        return Ok(vec![
            json!({ "name": ARGUS, "port": port, "url": format!("http://argus.localhost:{port}") }),
            json!({ "name": OVERLORD, "port": port, "url": format!("http://overlord.localhost:{port}") }),
        ]);
    }
    Ok(vec![shortcut_config(name, project, thread, port)?])
}
