//! The `argus tray` verbs end to end: the real binary in child processes on a throwaway dashboard
//! home. Ported from the `dashboard tray (CLI module)` cases and the stop cases of
//! `tray.test.mjs`; the state, `quit.flag`, launch and stop logic have unit tests next to the code
//! in `argus-native`. No case starts the tray itself (that would put an icon on the owner's
//! desktop): the start that spawns it, the real Quit and the PowerShell script cases of the Node
//! file have no counterpart, because the tray is native code in `argus-native` now.

use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::TempDir;

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

fn argus(home: &Path, args: &[&str]) -> Out {
    let out = Command::new(env!("CARGO_BIN_EXE_argus"))
        .arg("tray")
        .args(args)
        .arg("--data-dir")
        .arg(home)
        .output()
        .expect("argus runs");
    Out {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).trim().to_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
    }
}

fn home() -> TempDir {
    tempfile::tempdir().expect("a temp home")
}

fn write_tray(home: &Path, pid: u32) {
    let json = format!("{{\"pid\":{pid},\"startedAt\":\"2026-10-26T09:00:00.000Z\"}}\n");
    std::fs::write(home.join("tray.json"), json).expect("tray.json");
}

#[cfg(windows)]
mod windows {
    use super::*;

    /// A harmless process that outlives the test unless the guard ends it.
    struct Sleeper(std::process::Child);

    impl Sleeper {
        fn start() -> Sleeper {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            let child = Command::new("ping")
                .args(["-n", "60", "127.0.0.1"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .creation_flags(CREATE_NO_WINDOW)
                .spawn()
                .expect("ping starts");
            Sleeper(child)
        }
    }

    impl Drop for Sleeper {
        fn drop(&mut self) {
            let _ended = self.0.kill();
            let _reaped = self.0.wait();
        }
    }

    #[test]
    fn status_says_not_running_and_exits_1_with_no_tray() {
        let home = home();
        let out = argus(home.path(), &["status"]);
        assert_eq!((out.code, out.stdout.as_str()), (1, "tray not running"));
    }

    #[test]
    fn status_says_when_the_dashboard_was_closed_from_the_tray() {
        let home = home();
        std::fs::write(home.path().join("quit.flag"), "{}\n").expect("quit.flag");
        let out = argus(home.path(), &["status"]);
        assert_eq!(out.code, 1);
        assert!(
            out.stdout.contains("closed from the tray"),
            "{}",
            out.stdout
        );
    }

    #[test]
    fn status_exits_0_for_a_running_tray_and_ignores_a_dead_one() {
        let home = home();
        write_tray(home.path(), std::process::id());
        let running = argus(home.path(), &["status"]);
        let line = format!("tray running (pid {}", std::process::id());
        assert_eq!(running.code, 0);
        assert!(running.stdout.starts_with(&line), "{}", running.stdout);
        let gone = Sleeper::start();
        let dead = gone.0.id();
        drop(gone);
        write_tray(home.path(), dead);
        assert_eq!(argus(home.path(), &["status"]).code, 1);
    }

    #[test]
    fn start_with_a_tray_already_running_spawns_nothing() {
        let home = home();
        write_tray(home.path(), std::process::id());
        let before = std::fs::read_to_string(home.path().join("tray.json")).unwrap();
        let out = argus(home.path(), &["start"]);
        let line = format!("tray already running (pid {})", std::process::id());
        assert_eq!((out.code, out.stdout.as_str()), (0, line.as_str()));
        let after = std::fs::read_to_string(home.path().join("tray.json")).unwrap();
        assert_eq!(before, after, "no second tray replaced the record");
    }

    #[test]
    fn stop_with_no_tray_is_a_no_op_and_clears_a_stale_record() {
        let home = home();
        let gone = Sleeper::start();
        let dead = gone.0.id();
        drop(gone);
        write_tray(home.path(), dead);
        let out = argus(home.path(), &["stop"]);
        assert_eq!((out.code, out.stdout.as_str()), (0, "tray is not running"));
        assert!(!home.path().join("tray.json").exists());
    }

    #[test]
    fn a_tray_that_ignores_the_request_is_killed_and_the_request_file_is_removed() {
        let home = home();
        let stubborn = Sleeper::start();
        let pid = stubborn.0.id();
        write_tray(home.path(), pid);
        let out = argus(home.path(), &["stop"]);
        let line = format!("tray stopped (pid {pid}, forced); the server keeps running");
        assert_eq!((out.code, out.stdout.as_str()), (0, line.as_str()));
        assert!(!home.path().join("tray.stop").exists());
        assert!(!home.path().join("tray.json").exists());
        assert!(!argus_native::process_alive(pid), "the process is gone");
    }
}

#[cfg(not(windows))]
#[test]
fn start_fails_clearly_on_a_platform_without_a_tray() {
    let home = home();
    let out = argus(home.path(), &["start"]);
    assert_eq!(out.code, 1);
    assert!(out.stderr.contains("Windows only"), "{}", out.stderr);
}

#[test]
fn an_unknown_tray_verb_is_a_usage_error() {
    let home = home();
    let out = argus(home.path(), &["frobnicate"]);
    assert_eq!(out.code, 2);
    assert!(out.stderr.contains("frobnicate"), "{}", out.stderr);
}
