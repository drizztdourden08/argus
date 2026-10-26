//! The refusal of a taken board lock. The Node CLI waits 5 s for `board.lock` and then throws
//! ``board is locked by another coord call (${join(dir, 'board.lock')}); retry.`` (`acquireLock`
//! in `store.mjs`). The path is the one `join` gives, never resolved or canonicalised: the coord
//! home as the environment spells it, joined with the project key and the file name, with `.`
//! and `..` folded and the separators of the platform.

use std::path::Path;

use argus_board::BoardError;

use crate::cli::CoordError;

/// The refusal sentence for the lock of a project.
#[must_use]
pub fn lock_sentence(home: &Path, project: &str) -> String {
    let path = node_join(&[&home.to_string_lossy(), project, "board.lock"]);
    format!("board is locked by another coord call ({path}); retry.")
}

/// A board error as a refusal: a taken lock says the Node sentence, anything else keeps its own
/// message. Every verb that locks the board maps its errors through this.
#[must_use]
pub fn board_refusal(e: BoardError, home: &Path, project: &str) -> CoordError {
    match e {
        BoardError::Locked(_) => CoordError::refuse(lock_sentence(home, project)),
        other => other.into(),
    }
}

/// `path.join(...parts)` of Node: the parts joined with `/`, folded and written with the
/// platform separator.
fn node_join(parts: &[&str]) -> String {
    let windows = cfg!(windows);
    let is_sep = |c: char| c == '/' || (windows && c == '\\');
    let sep = if windows { "\\" } else { "/" };
    let joined = parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join("/");
    let rooted = joined.starts_with(is_sep);
    let unc = windows && rooted && joined[1..].starts_with(is_sep);
    let mut kept: Vec<&str> = Vec::new();
    for seg in joined.split(is_sep) {
        match seg {
            "" | "." => {}
            ".." if kept.last().is_some_and(|l| *l != ".." && !l.ends_with(':')) => {
                kept.pop();
            }
            ".." if rooted => {}
            other => kept.push(other),
        }
    }
    let prefix = match (unc, rooted) {
        (true, _) => "\\\\",
        (false, true) => sep,
        _ => "",
    };
    format!("{prefix}{}", kept.join(sep))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEP: &str = if cfg!(windows) { "\\" } else { "/" };

    #[test]
    fn the_sentence_names_the_unresolved_lock_path() {
        let s = lock_sentence(Path::new("rel/./home"), "argus");
        let path = ["rel", "home", "argus", "board.lock"].join(SEP);
        assert_eq!(
            s,
            format!("board is locked by another coord call ({path}); retry.")
        );
    }

    #[test]
    fn join_folds_dots_and_keeps_roots() {
        assert_eq!(
            node_join(&["/a/b/", "../c", "d.lock"]),
            format!("{SEP}a{SEP}c{SEP}d.lock")
        );
        assert_eq!(node_join(&["..", "x"]), format!("..{SEP}x"));
    }
}
