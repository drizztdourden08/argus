//! The refusals of `coord demo seed`: it writes only into a preview home that is set on purpose,
//! never into the live boards, and never over a board that is already there.

use std::path::{Path, PathBuf};

use argus_core::Paths;

use crate::cli::Env;
use crate::cli::exit::CoordError;

/// A path made comparable: absolute, resolved when it exists, one slash, lower case.
fn norm(path: &Path) -> String {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let real = absolute.canonicalize().unwrap_or(absolute);
    let text = real.to_string_lossy().replace('\\', "/").to_lowercase();
    text.trim_start_matches("//?/")
        .trim_end_matches('/')
        .to_owned()
}

/// Whether `home` is the live home or lies inside it.
fn is_live(home: &Path, live: &Path) -> bool {
    let (home, live) = (norm(home), norm(live));
    home == live || home.starts_with(&format!("{live}/"))
}

/// The preview coord home of the call: `COORD_HOME` must be set and must not be the live home
/// (`~/.coord`, as this process and as the call environment name it).
///
/// # Errors
/// `COORD_HOME` is not set, or is the live home or inside it.
pub(super) fn preview_home(env: &Env, cwd: &Path) -> Result<PathBuf, CoordError> {
    if env.get("COORD_HOME").is_none() {
        return Err(CoordError::refuse(
            "coord demo seed needs COORD_HOME set to a preview home, never the live boards. Example: COORD_HOME=<preview home> coord demo seed",
        ));
    }
    let home = env.paths(cwd).coord_home;
    let lives = [Paths::live_coord_home(), env.home_dir().join(".coord")];
    if lives.iter().any(|live| is_live(&home, live)) {
        return Err(CoordError::refuse(format!(
            "coord demo seed refuses the live home {}: set COORD_HOME to a separate preview home.",
            home.display()
        )));
    }
    Ok(home)
}

/// A project key: lowercase letters, digits and dashes, not starting with a dash.
fn is_key(text: &str) -> bool {
    let plain = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
    let mut chars = text.chars();
    chars.next().is_some_and(plain) && chars.all(|c| plain(c) || c == '-')
}

/// The key of the board to fill: `--project`, else `demo`.
///
/// # Errors
/// The key is not a project key, or it names a board the seed would collide with.
pub(super) fn board_key(option: Option<&str>) -> Result<String, CoordError> {
    let key = option.unwrap_or(argus_board::demo::DEFAULT_KEY);
    let taken = argus_board::demo::TARGETS.contains(&key) || key == argus_core::ids::PORTFOLIO;
    if !is_key(key) || taken {
        return Err(CoordError::refuse(format!(
            "\"{key}\" cannot be the demo board: use a plain key that is not portfolio, tessera, brock or archipelia."
        )));
    }
    Ok(key.to_owned())
}

/// Refuses a home that already holds the board: the seed never writes over a board. To seed
/// again, delete the folder of the board in the preview home.
///
/// # Errors
/// The board file exists.
pub(super) fn fresh(home: &Path, key: &str) -> Result<(), CoordError> {
    let file = argus_board::store::board_path(home, key);
    if file.exists() {
        return Err(CoordError::refuse(format!(
            "the demo board already exists at {}. Delete the folder {} to seed again.",
            file.display(),
            home.join(key).display()
        )));
    }
    Ok(())
}
