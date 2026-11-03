//! Files of the record stores the tests fill: a file written under a root, and a check run record.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// Writes `body` to `rel` under `root`, making the folders.
///
/// # Panics
/// When the folders or the file cannot be written.
pub fn put_file(root: &Path, rel: &str, body: impl AsRef<[u8]>) {
    let file = root.join(rel);
    fs::create_dir_all(file.parent().expect("parent")).expect("folders");
    fs::write(file, body).expect("file");
}

/// A passing check run record of `project` with one lint step whose log is `<id>/lint.log`.
#[must_use]
pub fn check_run_record(project: &str, id: &str, started: &str, commit: &str, pr: &str) -> Value {
    json!({
        "kind": "check-run", "id": "", "project": project, "commit": commit, "pr": pr,
        "status": "pass", "startedAt": started, "finishedAt": started,
        "steps": [{ "name": "lint", "command": "pnpm lint", "exitCode": 0, "log": format!("{id}/lint.log") }],
        "replay": { "state": "pending" },
    })
}
