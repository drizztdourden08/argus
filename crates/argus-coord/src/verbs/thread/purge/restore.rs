//! `coord restore --from <backup.json> [--thread T]... --confirm <project key>`: put a purge's
//! records back. Only records whose id is absent come back; nothing on the board is overwritten.

use std::fs;
use std::path::Path;

use argus_board::json::{display, truthy};
use argus_board::ops::{count_line, restore_from};
use serde_json::{Map, Value};

use super::confirm::confirmed;
use super::need_coordinator;
use super::text_of;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::resolve::resolve;
use crate::verbs::thread::frame::Frame;

fn read_backup(file: &Path) -> Result<Value, CoordError> {
    let text = fs::read_to_string(file).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => CoordError::refuse(format!(
            "ENOENT: no such file or directory, open '{}'",
            file.display()
        )),
        _ => CoordError::from(e),
    })?;
    serde_json::from_str(&text).map_err(|e| CoordError::refuse(e.to_string()))
}

fn said(counts: &Map<String, Value>) -> String {
    let skipped = counts.get("skipped").and_then(Value::as_u64).unwrap_or(0);
    let back: Map<String, Value> = counts
        .iter()
        .filter(|(k, _)| *k != "skipped")
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let note = if skipped > 0 {
        format!(", skipped {skipped} already present")
    } else {
        String::new()
    };
    if back.values().any(argus_core::js::truthy) {
        format!(
            "restored {} with their original ids{note}. Records already on the board were left alone.",
            count_line(&back)
        )
    } else {
        format!("restored 0{note}: every record in the file is already on the board.")
    }
}

/// The verb `restore`.
///
/// # Errors
/// Refused without `--from` and `--confirm`, for a backup that cannot be read or belongs to
/// another project, a caller that is not the coordinator, a thread the backup does not hold and
/// no board.
pub fn run(frame: &mut Frame<'_>) -> Outcome {
    let Some(from) = frame.options.text("from") else {
        return Err(CoordError::refuse(
            "pass --from <backup.json>, the file coord purge printed.",
        ));
    };
    if !confirmed(frame)? {
        return Err(CoordError::refuse(format!(
            "restore needs --confirm {} (the project key).",
            frame.project
        )));
    }
    let file = resolve(frame.cwd, Path::new(from));
    let backup = read_backup(&file)?;
    if truthy(&backup, "project") && backup["project"] != frame.project {
        return Err(CoordError::refuse(format!(
            "{} is a backup of \"{}\", this board is \"{}\".",
            file.display(),
            display(&backup["project"]),
            frame.project
        )));
    }
    let threads: Vec<String> = frame.options.many("thread").iter().map(text_of).collect();
    frame.mutate(false, |board, fr| {
        let who = fr.caller(board)?;
        need_coordinator(&who, "restore a purge")?;
        let counts = restore_from(board, &file, &threads)?;
        fr.say(&[&said(&counts)]);
        Ok(())
    })
}
