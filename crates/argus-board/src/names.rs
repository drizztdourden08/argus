//! The names the owner sees for the main coordinator and its project. Only what is shown
//! differs from the ids: the project key stays `portfolio`, the thread id `coordinator`.

use argus_core::js;

/// The key of the Overlord's own project.
pub const MAIN_KEY: &str = "portfolio";
/// The dashboard app and its overview page.
pub const ARGUS: &str = "Argus";
/// The main coordinator and its project, as shown.
pub const OVERLORD: &str = "Overlord";

fn is_legacy(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.strip_prefix("portfolio").is_some_and(|rest| {
        rest.chars()
            .next()
            .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_'))
    })
}

fn usable<'a>(key: &str, name: Option<&'a str>) -> Option<&'a str> {
    let name = js::trim(name?);
    (!name.is_empty() && name != key && !is_legacy(name)).then_some(name)
}

/// The display name of a project: its board name, else its registry name, else the key. The
/// Overlord's project shows the registry or board name when it is a real choice, else
/// `Overlord`.
#[must_use]
pub fn display_name(key: &str, board_name: Option<&str>, registry_name: Option<&str>) -> String {
    if key == MAIN_KEY {
        return usable(key, registry_name)
            .or_else(|| usable(key, board_name))
            .unwrap_or(OVERLORD)
            .to_owned();
    }
    board_name.or(registry_name).unwrap_or(key).to_owned()
}

/// What a session title starts with: `Overlord` for the main project, else the board name.
#[must_use]
pub fn session_name(key: &str, board_name: Option<&str>) -> String {
    if key == MAIN_KEY {
        OVERLORD.to_owned()
    } else {
        board_name.unwrap_or(key).to_owned()
    }
}
