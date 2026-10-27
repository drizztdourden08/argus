//! `tray.json`: the pid and start time of the process that holds the icon.

use std::fs;
use std::io;
use std::time::SystemTime;

use argus_core::fs::write_atomic;
use serde_json::Value;

use super::paths::TrayFiles;
use super::timestamp::iso_utc;

/// Who holds the icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayRecord {
    /// The process id of the tray.
    pub pid: u32,
    /// When it started, as ISO text, when the file says.
    pub started_at: Option<String>,
    /// The tooltip the icon shows now, when the file says. A tray that wrote this holds a
    /// registered icon: `tray.json` is written only after Windows accepted the icon.
    pub tooltip: Option<String>,
}

/// Writes `tray.json` for this process. It is called once the icon is registered with Windows
/// (`Shell_NotifyIcon` accepted it), so the file is the proof of a registered icon.
///
/// # Errors
///
/// The home cannot be created or the file cannot be written.
pub fn write(files: &TrayFiles, pid: u32) -> io::Result<()> {
    let started = iso_utc(SystemTime::now());
    write_atomic(
        &files.tray_json(),
        format!("{{\"pid\":{pid},\"startedAt\":\"{started}\",\"registered\":true}}\n"),
    )
}

/// Adds the tooltip the icon shows to the record of this process, keeping the rest. Does nothing
/// when the record names another process.
///
/// # Errors
///
/// The file could not be written.
pub fn set_tooltip(files: &TrayFiles, pid: u32, tooltip: &str) -> io::Result<()> {
    let Some(Value::Object(mut json)) = fs::read_to_string(files.tray_json())
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
    else {
        return Ok(());
    };
    if json.get("pid").and_then(Value::as_u64) != Some(u64::from(pid)) {
        return Ok(());
    }
    json.insert("tooltip".to_owned(), Value::String(tooltip.to_owned()));
    write_atomic(&files.tray_json(), format!("{}\n", Value::Object(json)))
}

/// The record of a tray whose process is alive, else `None`. A stale `tray.json` is ignored, not
/// deleted.
#[must_use]
pub fn read_live(files: &TrayFiles, alive: impl Fn(u32) -> bool) -> Option<TrayRecord> {
    let text = fs::read_to_string(files.tray_json()).ok()?;
    let json: Value = serde_json::from_str(&text).ok()?;
    let pid = u32::try_from(json.get("pid")?.as_u64()?).ok()?;
    alive(pid).then(|| TrayRecord {
        pid,
        started_at: json
            .get("startedAt")
            .and_then(Value::as_str)
            .map(str::to_owned),
        tooltip: json
            .get("tooltip")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

/// Removes `tray.json` when it names `pid`, so a newer tray's record is left alone.
pub fn remove_if_owned(files: &TrayFiles, pid: u32) {
    let named = fs::read_to_string(files.tray_json())
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|json| json.get("pid").and_then(Value::as_u64));
    if named == Some(u64::from(pid)) {
        let _removed = fs::remove_file(files.tray_json());
    }
}

#[cfg(test)]
mod tests {
    use super::{read_live, remove_if_owned, set_tooltip, write};
    use crate::tray::paths::TrayFiles;

    #[test]
    fn a_live_record_reads_back_and_a_dead_one_does_not_count() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        std::fs::write(
            files.tray_json(),
            r#"{"pid":777,"startedAt":"2026-10-22T10:00:00.000Z"}"#,
        )
        .unwrap();
        let live = read_live(&files, |pid| pid == 777).unwrap();
        assert_eq!(live.pid, 777);
        assert_eq!(live.started_at.as_deref(), Some("2026-10-22T10:00:00.000Z"));
        assert!(read_live(&files, |_| false).is_none());
        assert!(
            files.tray_json().exists(),
            "a stale file is ignored, not deleted"
        );
    }

    #[test]
    fn the_record_proves_a_registered_icon_and_carries_its_tooltip() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        write(&files, 8).unwrap();
        assert!(
            std::fs::read_to_string(files.tray_json())
                .unwrap()
                .contains("\"registered\":true")
        );
        assert!(read_live(&files, |_| true).unwrap().tooltip.is_none());
        set_tooltip(&files, 9, "other").unwrap();
        assert!(read_live(&files, |_| true).unwrap().tooltip.is_none());
        set_tooltip(&files, 8, "Argus (shadow): stopped").unwrap();
        let live = read_live(&files, |_| true).unwrap();
        assert_eq!(live.pid, 8);
        assert_eq!(live.tooltip.as_deref(), Some("Argus (shadow): stopped"));
        assert!(live.started_at.is_some());
    }

    #[test]
    fn a_missing_or_broken_file_is_no_tray() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        assert!(read_live(&files, |_| true).is_none());
        std::fs::write(files.tray_json(), "not json").unwrap();
        assert!(read_live(&files, |_| true).is_none());
    }

    #[test]
    fn the_record_is_written_through_the_atomic_writer_into_a_missing_home() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path().join("not").join("yet"));
        write(&files, 5).unwrap();
        let names: Vec<_> = std::fs::read_dir(files.home())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["tray.json"], "no temp file is left");
        assert_eq!(read_live(&files, |_| true).unwrap().pid, 5);
    }

    #[test]
    fn only_the_owner_removes_the_record() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        write(&files, 41).unwrap();
        remove_if_owned(&files, 42);
        assert!(files.tray_json().exists());
        remove_if_owned(&files, 41);
        assert!(!files.tray_json().exists());
    }
}
