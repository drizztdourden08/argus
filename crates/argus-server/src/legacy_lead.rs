//! The one-time rename of what the old lead name left behind: the listen stamps in the dashboard
//! home and the inbox read marks in the key-value table. Both are idempotent, so they run on every
//! start and do nothing once the old names are gone.

use std::fs;
use std::path::Path;

use argus_core::lead::{LEAD, LEGACY};
use argus_store::Db;

const STAMP_PREFIX: &str = "listen-";
const STAMP_SUFFIX: &str = ".json";
const HOSTED: &str = "-hosted";

/// The new name of a listen stamp that carries the old lead name, when it does.
fn renamed_stamp(name: &str) -> Option<String> {
    let middle = name
        .strip_prefix(STAMP_PREFIX)?
        .strip_suffix(STAMP_SUFFIX)?;
    let (board, tail) = match middle.strip_suffix(&format!("-{LEGACY}{HOSTED}")) {
        Some(board) => (board, format!("{LEAD}{HOSTED}")),
        None => (middle.strip_suffix(&format!("-{LEGACY}"))?, LEAD.to_owned()),
    };
    Some(format!("{STAMP_PREFIX}{board}-{tail}{STAMP_SUFFIX}"))
}

/// Renames every `listen-<key>-coordinator.json` of the dashboard home to the lead's name. A stamp
/// whose new name exists is the older one and is removed.
pub fn rename_listen_stamps(home: &Path) {
    let Ok(entries) = fs::read_dir(home) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(new) = renamed_stamp(&name) else {
            continue;
        };
        let to = home.join(new);
        if to.exists() {
            fs::remove_file(entry.path()).ok();
        } else {
            fs::rename(entry.path(), to).ok();
        }
    }
}

/// Renames every `inbox-read:<key>:coordinator` key to the lead's name; a mark whose new name
/// exists is the older one and is dropped.
pub fn rename_read_marks(db: &Db) {
    let like = format!("inbox-read:%:{LEGACY}");
    let cut = LEGACY.len();
    let update = format!(
        "UPDATE OR IGNORE kv SET key = substr(key, 1, length(key) - {cut}) || ?1 WHERE key LIKE ?2"
    );
    db.with_conn(|conn| {
        conn.execute(&update, [LEAD, like.as_str()]).ok();
        conn.execute("DELETE FROM kv WHERE key LIKE ?1", [like.as_str()])
            .ok();
    });
}

/// Both renames, for a started server.
pub fn migrate(home: &Path, db: &Db) {
    rename_listen_stamps(home);
    rename_read_marks(db);
}

#[cfg(test)]
mod tests {
    use super::*;
    use argus_core::Bus;

    #[test]
    fn only_the_old_lead_stamps_are_renamed() {
        assert_eq!(
            renamed_stamp("listen-rotp-coordinator.json").as_deref(),
            Some("listen-rotp-lead.json")
        );
        assert_eq!(
            renamed_stamp("listen-rotp-coordinator-hosted.json").as_deref(),
            Some("listen-rotp-lead-hosted.json")
        );
        assert_eq!(renamed_stamp("listen-rotp-lead.json"), None);
        assert_eq!(renamed_stamp("listen-rotp-ui.json"), None);
        assert_eq!(renamed_stamp("config.json"), None);
    }

    #[test]
    fn a_stamp_is_moved_and_an_older_duplicate_is_dropped() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("listen-a-coordinator.json");
        let twin_old = dir.path().join("listen-b-coordinator.json");
        let twin_new = dir.path().join("listen-b-lead.json");
        fs::write(&old, "1").unwrap();
        fs::write(&twin_old, "old").unwrap();
        fs::write(&twin_new, "new").unwrap();
        rename_listen_stamps(dir.path());
        let moved = fs::read_to_string(dir.path().join("listen-a-lead.json")).unwrap();
        assert_eq!(moved, "1");
        assert_eq!(fs::read_to_string(&twin_new).unwrap(), "new");
        assert!(!old.exists());
        assert!(!twin_old.exists());
    }

    #[test]
    fn a_read_mark_is_moved_to_the_lead_key() {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(&dir.path().join("a.db"), Bus::new()).unwrap();
        db.kv_set("inbox-read:alpha:coordinator", "5").unwrap();
        db.kv_set("inbox-read:alpha:ui", "7").unwrap();
        db.kv_set("inbox-read:beta:coordinator", "1").unwrap();
        db.kv_set("inbox-read:beta:lead", "2").unwrap();
        rename_read_marks(&db);
        let alpha = db.kv_get("inbox-read:alpha:lead").unwrap();
        assert_eq!(alpha.as_deref(), Some("5"));
        assert_eq!(db.kv_get("inbox-read:alpha:coordinator").unwrap(), None);
        let ui = db.kv_get("inbox-read:alpha:ui").unwrap();
        assert_eq!(ui.as_deref(), Some("7"));
        let beta = db.kv_get("inbox-read:beta:lead").unwrap();
        assert_eq!(beta.as_deref(), Some("2"));
        assert_eq!(db.kv_get("inbox-read:beta:coordinator").unwrap(), None);
    }
}
