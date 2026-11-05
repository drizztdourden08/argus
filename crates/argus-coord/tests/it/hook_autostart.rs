//! E2: the hook that starts the dashboard, with a stub install run by node.

// test-tag: e2e

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use argus_core::process::hidden_command;

use crate::common::Home;
use crate::hook_support::hook_with;
use argus_coord::Env;
use serde_json::{Value, json};

struct Case {
    home: Home,
    marker: PathBuf,
}

fn node_runs() -> bool {
    let ran = hidden_command("node")
        .arg("--version")
        .stdout(Stdio::null())
        .status();
    ran.is_ok_and(|s| s.success())
}

impl Case {
    fn new() -> Option<Case> {
        if !node_runs() {
            eprintln!("skipped: node is not installed");
            return None;
        }
        let home = Home::new();
        let install = home.folder("install");
        let marker = home.path().join("started.json");
        fs::create_dir_all(install.join("bin")).unwrap();
        let stub = format!(
            "import {{ writeFileSync }} from 'node:fs'; writeFileSync({}, JSON.stringify(process.argv.slice(2)));\n",
            json!(marker.to_string_lossy())
        );
        fs::write(install.join("bin").join("dashboard.mjs"), stub).unwrap();
        let config = json!({ "port": 51999, "installDir": install });
        fs::write(
            home.dashboard_home().join("config.json"),
            config.to_string(),
        )
        .unwrap();
        Some(Case { home, marker })
    }

    fn env(&self, autostart: &str) -> Env {
        let coord = self.home.coord_home().to_string_lossy().into_owned();
        let dash = self.home.dashboard_home().to_string_lossy().into_owned();
        Env::of(&[
            ("COORD_HOME", &coord),
            ("DASHBOARD_HOME", &dash),
            ("COORD_DASHBOARD_AUTOSTART", autostart),
        ])
    }

    fn hook(&self, event: &str, autostart: &str) {
        let input = json!({ "session_id": "sid-x", "cwd": self.home.path() }).to_string();
        let ran = hook_with(self.env(autostart), event, &input, self.home.path());
        assert_eq!((ran.code, ran.out.as_str()), (0, ""));
    }

    fn dash(&self, name: &str) -> PathBuf {
        self.home.dashboard_home().join(name)
    }

    /// The arguments the stub was started with, waiting up to 3 s for it.
    fn started(&self) -> Option<Value> {
        for _ in 0..60 {
            if let Ok(text) = fs::read_to_string(&self.marker) {
                if let Ok(args) = serde_json::from_str(&text) {
                    return Some(args);
                }
            }
            sleep(Duration::from_millis(50));
        }
        None
    }

    fn not_started(&self) -> bool {
        sleep(Duration::from_millis(400));
        !self.marker.exists()
    }
}

fn age(file: &Path, by: Duration) {
    let at = SystemTime::now() - by;
    File::options()
        .write(true)
        .open(file)
        .unwrap()
        .set_modified(at)
        .unwrap();
}

#[test]
#[ignore = "e2e"]
fn a_session_start_with_no_server_starts_it_once() {
    let Some(case) = Case::new() else { return };
    case.hook("session-start", "1");
    let expected = if cfg!(windows) {
        json!(["tray", "start"])
    } else {
        json!(["start"])
    };
    assert_eq!(case.started(), Some(expected));
    assert!(case.dash("start.lock").exists());
    fs::remove_file(&case.marker).unwrap();
    case.hook("prompt", "1");
    assert!(
        case.not_started(),
        "a second start inside a minute is not made"
    );
}

#[test]
#[ignore = "e2e"]
fn a_prompt_starts_it_too_and_an_old_lock_does_not_hold_it_back() {
    let Some(case) = Case::new() else { return };
    fs::write(case.dash("start.lock"), "old").unwrap();
    age(&case.dash("start.lock"), Duration::from_secs(300));
    case.hook("prompt", "1");
    assert!(case.started().is_some());
}

#[test]
#[ignore = "e2e"]
fn a_live_server_is_left_alone() {
    let Some(case) = Case::new() else { return };
    let pid = json!({ "pid": std::process::id(), "port": 51999 });
    fs::write(case.dash("server.pid"), pid.to_string()).unwrap();
    case.hook("session-start", "1");
    assert!(case.not_started());
}

#[test]
#[ignore = "e2e"]
fn a_quit_flag_of_this_boot_keeps_it_down_and_one_from_before_the_boot_does_not() {
    let Some(case) = Case::new() else { return };
    fs::write(case.dash("quit.flag"), "{}").unwrap();
    case.hook("session-start", "1");
    assert!(case.not_started());
    age(
        &case.dash("quit.flag"),
        Duration::from_secs(365 * 24 * 3600),
    );
    case.hook("session-start", "1");
    assert!(case.started().is_some());
}

#[test]
#[ignore = "e2e"]
fn other_events_never_start_it_and_a_home_override_needs_the_opt_in() {
    let Some(case) = Case::new() else { return };
    case.hook("tool", "1");
    case.hook("stop", "1");
    case.hook("guard", "1");
    assert!(case.not_started());
    case.hook("session-start", "");
    assert!(case.not_started());
    case.hook("session-start", "yes");
    assert!(case.not_started(), "only 1 opts in");
}

#[test]
#[ignore = "e2e"]
fn no_autostart_turns_it_off_and_a_missing_install_starts_nothing() {
    let Some(case) = Case::new() else { return };
    let coord = case.home.coord_home().to_string_lossy().into_owned();
    let dash = case.home.dashboard_home().to_string_lossy().into_owned();
    let off = Env::of(&[
        ("COORD_HOME", &coord),
        ("DASHBOARD_HOME", &dash),
        ("COORD_DASHBOARD_AUTOSTART", "1"),
        ("COORD_DASHBOARD_NO_AUTOSTART", "1"),
    ]);
    let input = json!({ "session_id": "sid-x", "cwd": case.home.path() }).to_string();
    hook_with(off, "session-start", &input, case.home.path());
    assert!(case.not_started());
    assert!(!case.dash("start.lock").exists());
    fs::write(
        case.dash("config.json"),
        r#"{"port":51999,"installDir":"Z:/nowhere"}"#,
    )
    .unwrap();
    case.hook("session-start", "1");
    assert!(case.not_started());
    assert!(
        !case.dash("start.lock").exists(),
        "no lock without an install to start"
    );
}
