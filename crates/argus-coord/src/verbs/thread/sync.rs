//! `coord sync import <dir>` (`verbs/sync.mjs`): replay the owner's entries saved as
//! `<dir>/entries/*.json` onto the board, offline. The other `sync` verbs are retired.

use std::fs;
use std::path::Path;

use argus_board::sync::import_entries_at;
use serde_json::{Value, json};

use super::frame::{Frame, with_frame};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::resolve::resolve;
use crate::cli::route::Call;

const RETIRED: [&str; 4] = ["export", "pushed", "versions", "asset"];

/// The entries saved under `<dir>/entries`, by file name.
fn read_entries(dir: &Path) -> Result<Vec<Value>, CoordError> {
    let folder = dir.join("entries");
    if !folder.exists() {
        return Ok(Vec::new());
    }
    let mut files: Vec<_> = fs::read_dir(&folder)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort_by_key(|p| p.to_string_lossy().to_lowercase());
    files
        .iter()
        .map(|file| {
            let text = fs::read_to_string(file)?;
            serde_json::from_str(&text).map_err(|e| CoordError::refuse(e.to_string()))
        })
        .collect()
}

fn import(frame: &mut Frame<'_>) -> Outcome {
    let home = frame.home();
    let dir = resolve(frame.cwd, Path::new(frame.arg(1).unwrap_or("")));
    frame.mutate(false, |board, fr| {
        let result = import_entries_at(home, board, read_entries(&dir)?, &|_| false);
        fr.json(&json!({
            "imported": result.imported,
            "deliveries": result.deliveries,
            "ops": {
                "applied": result.ops.applied,
                "conflict": result.ops.conflict,
                "refused": result.ops.refused,
            },
            "notes": result.notes,
        }));
        Ok(())
    })
}

/// The verb `sync`.
///
/// # Errors
/// Refused for any verb but `import`, an entry file that is not JSON and no board.
pub fn run(call: &mut Call<'_>) -> Outcome {
    with_frame(call, |frame| match frame.arg(0) {
        Some("import") => import(frame),
        Some(sub) if RETIRED.contains(&sub) => {
            frame.say(&[&format!(
                "retired: coord sync {sub} is gone; the dashboard mirrors the board live (coord board open). Only coord sync import <dir> remains, to replay saved entries."
            )]);
            Ok(())
        }
        _ => Err(CoordError::refuse(
            "sync verb must be: import <dir> (replays <dir>/entries/*.json onto the board).",
        )),
    })
}
