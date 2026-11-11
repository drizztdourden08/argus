//! The real [`DesktopApp`]: Claude Desktop, the MSIX app, on this PC.
//!
//! Nothing here kills a process. `close` asks the main window of each process that lives in the
//! app's install folder to close, the way the close button does, and waits. `start` goes through
//! the Start menu app id, the way a click on the tile does.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use argus_core::clock::now_ms;
use argus_core::process::hidden_command;
use serde_json::Value;

use crate::desktop::vault::last_known_account;
use crate::ports::DesktopApp;
use crate::sync::lock_running;

/// How long `close` waits for the app to go away.
pub const CLOSE_WAIT: Duration = Duration::from_secs(20);
/// How long `start` waits for the app to hold its lock.
pub const START_WAIT: Duration = Duration::from_secs(60);
const POLL: Duration = Duration::from_millis(500);
/// Why a close that left the app running failed.
pub const NOT_QUIT: &str = "Claude did not quit; it was probably kept in the system tray. \
    Quit it from its tray icon, or turn off its keep-running-in-the-tray option in Claude's \
    settings, then switch again";
/// A chat whose record changed this recently counts as in a turn.
const ACTIVE_WITHIN_MS: i64 = 90_000;
/// A user message with no turn end after it counts as a running turn for this long.
const OPEN_FRAME_MS: i64 = 30 * 60 * 1000;

/// The folder the app is installed in, from the package list.
const INSTALL_SCRIPT: &str =
    "(Get-AppxPackage -Name Claude | Select-Object -First 1).InstallLocation";
/// The version of the installed package.
const VERSION_SCRIPT: &str = "(Get-AppxPackage -Name Claude | Select-Object -First 1).Version";
/// The id of the app's Start menu tile.
const APP_ID_SCRIPT: &str =
    "(Get-StartApps | Where-Object Name -eq 'Claude' | Select-Object -First 1).AppID";

/// The script that closes the main windows of the processes inside `install`. A process counts
/// when its file is under the install folder and it owns a main window; nothing else is touched.
#[must_use]
pub fn close_script(install: &str) -> String {
    let folder = format!("{}\\", install.trim_end_matches('\\')).replace('\'', "''");
    format!(
        "$root = '{folder}'; \
         Get-Process | Where-Object {{ $_.Path -and $_.MainWindowHandle -ne 0 -and \
         $_.Path.StartsWith($root, [StringComparison]::OrdinalIgnoreCase) }} | \
         ForEach-Object {{ [void]$_.CloseMainWindow() }}"
    )
}

/// The argument that makes Explorer start the app: `shell:AppsFolder\<id>`.
///
/// # Errors
/// When the id holds characters an app id never has.
pub fn start_target(app_id: &str) -> Result<String, String> {
    let sane = !app_id.is_empty()
        && app_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '!' | '-'));
    if sane {
        Ok(format!("shell:AppsFolder\\{app_id}"))
    } else {
        Err("the Claude app id looks wrong".to_owned())
    }
}

/// Runs one PowerShell command and returns its trimmed output.
fn powershell(script: &str) -> Result<String, String> {
    let out = hidden_command("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| format!("PowerShell did not start: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    } else {
        let why = String::from_utf8_lossy(&out.stderr);
        Err(format!("PowerShell failed: {}", why.trim()))
    }
}

/// Polls `done` every `step` until it is true or `limit` has passed. Returns the last answer.
pub fn wait_until(limit: Duration, step: Duration, mut done: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    loop {
        if done() {
            return true;
        }
        if start.elapsed() >= limit {
            return false;
        }
        std::thread::sleep(step);
    }
}

/// Whether a chat record looks like a turn is running. Two signals, either one counts: the
/// record changed in the last 90 seconds (a running turn writes it all the time), or the latest
/// user message is newer than the last turn edge and under 30 minutes old (a turn was started
/// and has not ended). Neither is exact, so a false answer is never a promise.
#[must_use]
pub fn looks_mid_turn(record: &Value, now: i64) -> bool {
    let at = |key: &str| record.get(key).and_then(Value::as_i64).unwrap_or(0);
    if record.get("isArchived").and_then(Value::as_bool) == Some(true) {
        return false;
    }
    let activity = at("lastActivityAt");
    let recent = activity > 0 && now - activity < ACTIVE_WITHIN_MS;
    let frame = at("latestUserFrameAt");
    let open = frame > at("lastTurnEdgeAt") && now - frame < OPEN_FRAME_MS;
    recent || open
}

/// Claude Desktop with its data folder.
pub struct ClaudeDesktop {
    app: PathBuf,
    app_id: Mutex<Option<String>>,
    install: Mutex<Option<String>>,
}

impl ClaudeDesktop {
    /// The app whose data lives in `app` (`...\Claude-Data\claude_<id>`).
    #[must_use]
    pub fn new(app: &Path) -> Self {
        ClaudeDesktop {
            app: app.to_path_buf(),
            app_id: Mutex::new(None),
            install: Mutex::new(None),
        }
    }

    /// The cached answer of `ask`, asked once and kept only when it worked.
    fn cached(
        slot: &Mutex<Option<String>>,
        ask: impl FnOnce() -> Result<String, String>,
    ) -> Result<String, String> {
        let mut held = slot.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(value) = held.as_ref() {
            return Ok(value.clone());
        }
        let value = ask()?;
        *held = Some(value.clone());
        Ok(value)
    }

    fn install_folder(&self) -> Result<String, String> {
        Self::cached(&self.install, || {
            let found = powershell(INSTALL_SCRIPT)?;
            if found.is_empty() {
                Err("the Claude app is not installed as a Windows app".to_owned())
            } else {
                Ok(found)
            }
        })
    }

    fn app_id(&self) -> Result<String, String> {
        Self::cached(&self.app_id, || {
            let found = powershell(APP_ID_SCRIPT)?;
            start_target(&found).map(|_| found)
        })
    }

    /// The chat records of the account that is logged in now, over its scopes.
    fn session_files(&self) -> Vec<PathBuf> {
        let Some(account) = last_known_account(&self.app) else {
            return Vec::new();
        };
        let root = self.app.join("claude-code-sessions").join(account);
        let orgs = std::fs::read_dir(root).into_iter().flatten().flatten();
        orgs.flat_map(|org| {
            std::fs::read_dir(org.path())
                .into_iter()
                .flatten()
                .flatten()
        })
        .map(|entry| entry.path())
        .filter(|path| is_chat_record(path))
        .collect()
    }
}

fn is_chat_record(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.starts_with("local_") && name.ends_with(".json")
}

impl DesktopApp for ClaudeDesktop {
    fn is_running(&self) -> bool {
        lock_running(&self.app)
    }

    fn turn_in_progress(&self) -> bool {
        let now = now_ms();
        self.session_files().iter().any(|path| {
            std::fs::read_to_string(path)
                .ok()
                .and_then(|text| serde_json::from_str::<Value>(&text).ok())
                .is_some_and(|record| looks_mid_turn(&record, now))
        })
    }

    fn close(&self) -> Result<(), String> {
        if !self.is_running() {
            return Ok(());
        }
        powershell(&close_script(&self.install_folder()?))?;
        if wait_until(CLOSE_WAIT, POLL, || !self.is_running()) {
            Ok(())
        } else {
            Err(NOT_QUIT.to_owned())
        }
    }

    /// Asks Explorer to open the app's tile. Explorer often exits with code 1 even when the app
    /// started, so its exit code says nothing: the app holding its LOCK within 60 seconds is the
    /// only success signal. Only a launch that could not be made at all is an error right away.
    fn start(&self) -> Result<(), String> {
        let target = start_target(&self.app_id()?)?;
        hidden_command("explorer.exe")
            .arg(target)
            .status()
            .map_err(|e| format!("Claude did not start: {e}"))?;
        if wait_until(START_WAIT, POLL, || self.is_running()) {
            Ok(())
        } else {
            Err("Claude did not start within 60 seconds".to_owned())
        }
    }

    fn config_touched_since(&self, since_ms: i64) -> bool {
        std::fs::metadata(self.app.join("config.json"))
            .and_then(|meta| meta.modified())
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .is_some_and(|age| i64::try_from(age.as_millis()).is_ok_and(|ms| ms > since_ms))
    }

    fn version(&self) -> Option<String> {
        powershell(VERSION_SCRIPT)
            .ok()
            .filter(|text| !text.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;

    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{ClaudeDesktop, NOT_QUIT, close_script, looks_mid_turn, start_target, wait_until};
    use crate::ports::DesktopApp;

    fn now_ms() -> i64 {
        let age = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        i64::try_from(age.as_millis()).unwrap()
    }

    #[test]
    fn a_close_that_leaves_claude_in_the_tray_says_so_and_never_kills() {
        assert!(NOT_QUIT.contains("tray") && NOT_QUIT.contains("settings"));
    }

    #[test]
    fn config_touched_since_compares_the_file_time() {
        let dir = tempfile::TempDir::new().unwrap();
        let desktop = ClaudeDesktop::new(dir.path());
        assert!(!desktop.config_touched_since(0));
        std::fs::write(dir.path().join("config.json"), "{}").unwrap();
        assert!(desktop.config_touched_since(now_ms() - 60_000));
        assert!(!desktop.config_touched_since(now_ms() + 60_000));
    }

    #[test]
    fn close_script_is_limited_to_the_install_folder() {
        let script = close_script("C:\\Program Files\\WindowsApps\\Claude_1_x64__abc");
        assert!(script.contains("'C:\\Program Files\\WindowsApps\\Claude_1_x64__abc\\'"));
        assert!(script.contains("CloseMainWindow"));
        assert!(!script.contains("Stop-Process") && !script.contains("taskkill"));
    }

    #[test]
    fn close_script_doubles_quotes_and_one_trailing_slash() {
        let script = close_script("C:\\it's\\");
        assert!(script.contains("'C:\\it''s\\'"));
    }

    #[test]
    fn start_target_names_the_apps_folder() {
        assert_eq!(
            start_target("Claude_pzs8sxrjxfjjc!Claude").unwrap(),
            "shell:AppsFolder\\Claude_pzs8sxrjxfjjc!Claude"
        );
    }

    #[test]
    fn start_target_refuses_odd_ids() {
        assert!(start_target("").is_err());
        assert!(start_target("a b & calc").is_err());
    }

    #[test]
    fn wait_until_stops_when_true_or_late() {
        let step = Duration::from_millis(1);
        let mut calls = 0;
        assert!(wait_until(Duration::from_secs(1), step, || {
            calls += 1;
            calls == 3
        }));
        assert!(!wait_until(Duration::from_millis(5), step, || false));
    }

    #[test]
    fn a_recent_record_looks_mid_turn() {
        let now = 10_000_000;
        assert!(looks_mid_turn(
            &json!({ "lastActivityAt": now - 5_000 }),
            now
        ));
        assert!(!looks_mid_turn(
            &json!({ "lastActivityAt": now - 600_000 }),
            now
        ));
    }

    #[test]
    fn an_open_user_frame_looks_mid_turn_for_half_an_hour() {
        let now = 100_000_000;
        let open = json!({ "latestUserFrameAt": now - 60_000, "lastTurnEdgeAt": now - 120_000 });
        let closed = json!({ "latestUserFrameAt": now - 60_000, "lastTurnEdgeAt": now - 10_000 });
        let stale = json!({ "latestUserFrameAt": now - 3_600_000, "lastTurnEdgeAt": 0 });
        assert!(looks_mid_turn(&open, now));
        assert!(!looks_mid_turn(&closed, now));
        assert!(!looks_mid_turn(&stale, now));
    }

    #[test]
    fn an_archived_chat_is_never_mid_turn() {
        let now = 1_000_000;
        assert!(!looks_mid_turn(
            &json!({ "isArchived": true, "lastActivityAt": now }),
            now
        ));
    }
}
