//! The real [`DesktopApp`]: Claude Desktop, the MSIX app, on this PC.
//!
//! `close` asks the main window of each process that lives in the app's install folder to close,
//! the way the close button does, and waits. The app often stays in the system tray; then `close`
//! waits for the owner to quit it from the tray, or, when told to force, ends the processes of
//! the install folder by PID (never by image name). `start` goes through the Start menu app id,
//! the way a click on the tile does.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use argus_core::clock::now_ms;
use argus_core::process::hidden_command;
use serde_json::Value;

use crate::desktop::vault::last_known_account;
use crate::ports::{DesktopApp, QUIT_BY_HAND, QUIT_CANCELLED, Quit};
use crate::sync::lock_running;

/// How long `close` waits for the polite close before it asks the owner to quit by hand.
pub const CLOSE_WAIT: Duration = Duration::from_secs(20);
/// How long `close` waits for the owner to quit the app from its tray.
pub const QUIT_WAIT: Duration = Duration::from_secs(600);
/// How often the wait for the quit looks at the lock.
pub const QUIT_POLL: Duration = Duration::from_secs(2);
/// How long a forced close waits after the plain `taskkill` before it adds `/F`.
pub const KILL_GRACE: Duration = Duration::from_secs(10);
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

/// The script that lists the process ids inside `install`, one per line.
#[must_use]
pub fn pids_script(install: &str) -> String {
    let folder = format!("{}\\", install.trim_end_matches('\\')).replace('\'', "''");
    format!(
        "$root = '{folder}'; \
         Get-Process | Where-Object {{ $_.Path -and \
         $_.Path.StartsWith($root, [StringComparison]::OrdinalIgnoreCase) }} | \
         ForEach-Object {{ $_.Id }}"
    )
}

/// The process ids a [`pids_script`] printed; lines that are not numbers are dropped.
#[must_use]
pub fn parse_pids(output: &str) -> Vec<u32> {
    output
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect()
}

/// The `taskkill` arguments for one PID: with its children, forced only when asked.
#[must_use]
pub fn taskkill_args(pid: u32, force: bool) -> Vec<String> {
    let mut args = vec!["/PID".to_owned(), pid.to_string(), "/T".to_owned()];
    if force {
        args.push("/F".to_owned());
    }
    args
}

/// Waits for `gone` after the polite close: `grace`, then (with `force`) `kill` once and `grace`
/// again, then up to `limit` for the owner, looking every `poll`. `waiting` hears the detail when
/// the wait for the owner starts. Cancel is asked while waiting for the owner.
///
/// # Errors
/// [`QUIT_CANCELLED`] when cancelled, [`NOT_QUIT`] when `limit` passed.
pub fn await_quit(
    quit: &mut Quit,
    (grace, limit, poll): (Duration, Duration, Duration),
    gone: &dyn Fn() -> bool,
    kill: &mut dyn FnMut(),
) -> Result<(), String> {
    if wait_until(grace, poll.min(POLL), gone) {
        return Ok(());
    }
    if quit.force {
        kill();
        if wait_until(grace, poll.min(POLL), gone) {
            return Ok(());
        }
    }
    (quit.waiting)(QUIT_BY_HAND);
    let mut cancelled = false;
    let freed = wait_until(limit, poll, || {
        cancelled = (quit.cancelled)();
        cancelled || gone()
    });
    if cancelled && !gone() {
        Err(QUIT_CANCELLED.to_owned())
    } else if freed {
        Ok(())
    } else {
        Err(NOT_QUIT.to_owned())
    }
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

/// Whether `<app>\config.json` was written after `since_ms` (epoch milliseconds).
#[must_use]
pub fn config_written_after(app: &Path, since_ms: i64) -> bool {
    std::fs::metadata(app.join("config.json"))
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .is_some_and(|age| i64::try_from(age.as_millis()).is_ok_and(|ms| ms > since_ms))
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

/// Ends the processes of the install folder by PID: `taskkill /T` first, `/T /F` on what is
/// left after [`KILL_GRACE`]. Failures are ignored; the wait for the lock says what happened.
fn kill_claude(install: &str) {
    let pids = |script: &str| {
        powershell(script)
            .map(|out| parse_pids(&out))
            .unwrap_or_default()
    };
    let list = pids(&pids_script(install));
    let run = |pid: u32, force: bool| {
        let _ = hidden_command("taskkill")
            .args(taskkill_args(pid, force))
            .output();
    };
    for pid in &list {
        run(*pid, false);
    }
    std::thread::sleep(KILL_GRACE);
    for pid in pids(&pids_script(install)) {
        run(pid, true);
    }
}

fn is_chat_record(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.starts_with("local_")
        && path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
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

    fn close(&self, quit: &mut Quit) -> Result<(), String> {
        if !self.is_running() {
            return Ok(());
        }
        let install = self.install_folder()?;
        powershell(&close_script(&install))?;
        let limits = (CLOSE_WAIT, QUIT_WAIT, QUIT_POLL);
        await_quit(quit, limits, &|| !self.is_running(), &mut || {
            kill_claude(&install);
        })
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
        config_written_after(&self.app, since_ms)
    }

    fn version(&self) -> Option<String> {
        powershell(VERSION_SCRIPT)
            .ok()
            .filter(|text| !text.is_empty())
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;

    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        ClaudeDesktop, NOT_QUIT, await_quit, close_script, looks_mid_turn, parse_pids, pids_script,
        start_target, taskkill_args, wait_until,
    };
    use crate::ports::{DesktopApp, QUIT_BY_HAND, QUIT_CANCELLED, Quit};

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

    const MS: Duration = Duration::from_millis(1);
    const LIMITS: (Duration, Duration, Duration) =
        (Duration::from_millis(5), Duration::from_millis(40), MS);

    fn quit_never_killing(told: &mut Vec<String>, gone: &dyn Fn() -> bool) -> Result<(), String> {
        await_quit(
            &mut Quit {
                force: false,
                cancelled: &|| false,
                waiting: &mut |t| told.push(t.to_owned()),
            },
            LIMITS,
            gone,
            &mut || panic!("no kill"),
        )
    }

    #[test]
    fn a_quit_in_the_grace_needs_no_waiting_text() {
        let mut told = Vec::new();
        let result = quit_never_killing(&mut told, &|| true);
        assert_eq!(result, Ok(()));
        assert_eq!(told.len(), 0);
    }

    #[test]
    fn a_tray_app_makes_the_close_wait_and_say_so() {
        let mut told = Vec::new();
        let result = quit_never_killing(&mut told, &|| false);
        assert_eq!(result, Err(NOT_QUIT.to_owned()));
        assert_eq!(told, [QUIT_BY_HAND]);
    }

    #[test]
    fn the_owner_quitting_during_the_wait_ends_it_well() {
        let asked = std::cell::Cell::new(false);
        let result = await_quit(
            &mut Quit {
                force: false,
                cancelled: &|| false,
                waiting: &mut |_| asked.set(true),
            },
            LIMITS,
            &|| asked.get(),
            &mut || {},
        );
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn cancel_ends_the_wait() {
        let result = await_quit(
            &mut Quit {
                force: false,
                cancelled: &|| true,
                waiting: &mut |_| {},
            },
            LIMITS,
            &|| false,
            &mut || {},
        );
        assert_eq!(result, Err(QUIT_CANCELLED.to_owned()));
    }

    #[test]
    fn force_kills_once_and_a_freed_lock_ends_the_close() {
        let killed = std::cell::Cell::new(0);
        let result = await_quit(
            &mut Quit {
                force: true,
                cancelled: &|| false,
                waiting: &mut |_| panic!("no wait"),
            },
            LIMITS,
            &|| killed.get() > 0,
            &mut || killed.set(killed.get() + 1),
        );
        assert_eq!(result, Ok(()));
        assert_eq!(killed.get(), 1);
    }

    #[test]
    fn pids_come_from_the_install_folder_only() {
        let script = pids_script("C:\\Apps\\Claude_1\\");
        assert!(script.contains("'C:\\Apps\\Claude_1\\'"));
        assert!(script.contains("StartsWith($root"));
        assert_eq!(parse_pids("12\r\n\nx\n 7 \n"), [12, 7]);
    }

    #[test]
    fn taskkill_is_never_forced_before_the_grace_and_never_by_name() {
        assert_eq!(taskkill_args(5, false), ["/PID", "5", "/T"]);
        assert_eq!(taskkill_args(5, true), ["/PID", "5", "/T", "/F"]);
    }
}
