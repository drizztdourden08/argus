//! Every proof path a board names that existed in the backup exists under the new folder.

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

use super::Check;
use crate::error::Result;
use crate::journal::Entry;
use crate::jsonio;

static PATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\.coord[\\/]([a-z0-9][a-z0-9-]*)[\\/]([^\s"'`)\]]+)"#)
        .expect("the path pattern")
});

fn strings_of<'a>(v: &'a Value, out: &mut Vec<&'a str>) {
    match v {
        Value::String(s) => out.push(s),
        Value::Array(a) => a.iter().for_each(|e| strings_of(e, out)),
        Value::Object(m) => m.values().for_each(|e| strings_of(e, out)),
        _ => {}
    }
}

/// Checks the proof paths of the board `e`, read from its backup file `before`.
pub fn check(check: &Check, e: &Entry, before: &Path, problems: &mut Vec<String>) -> Result<()> {
    let Some(backup) = check.backup else {
        return Ok(());
    };
    let board = jsonio::read(before)?;
    let mut texts = Vec::new();
    strings_of(&board, &mut texts);
    for text in texts {
        for found in PATH.captures_iter(text) {
            let tail = found[2]
                .trim_end_matches(['.', ',', ';', ':'])
                .replace('\\', "/");
            let tail = tail.as_str();
            let key = found[1].to_lowercase();
            let Some(id) = check.map.get(&key) else {
                continue;
            };
            let existed = backup.join(&key).join(tail).is_file();
            if existed && !check.home.join(id).join(tail).is_file() {
                problems.push(format!("{}: the proof {tail} is missing", e.id));
            }
        }
    }
    Ok(())
}
