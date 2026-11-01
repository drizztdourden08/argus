//! KEEP: K4.
//! A rename refused because something holds the folder names who holds it: Argus's own live run working at or above
//! the folder, and on Windows the process that has a file in it open.

// test-tag: e2e

use crate::error::ErrorCode;
use crate::ports::LiveRun;
use crate::sync::locked;
use crate::testkit::{Fixture, Harness, backdate, exists_itself, put, rule_for};

/// A fixture with an old leftover folder `name` (one file), scanned.
fn leftover(name: &str) -> (Fixture, Harness) {
    let fx = Fixture::new();
    put(&fx.wts, &format!("{name}/a.txt"), "hello\n");
    backdate(&fx.path(name), 10.0);
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    (fx, h)
}

fn run(id: &str, cwd: &str) -> LiveRun {
    LiveRun {
        id: id.to_string(),
        project: "alpha".to_string(),
        cmd: "pnpm   dev".to_string(),
        cwd: cwd.to_string(),
    }
}

#[test]
#[ignore = "e2e"]
fn a_live_run_working_above_the_folder_is_named_and_no_rescan_is_asked() {
    let (fx, h) = leftover("junk-run");
    *locked(&h.procs.1) = vec![
        run("R-17", &fx.root),
        run("R-3", &format!("{}/elsewhere", fx.root)),
    ];
    *h.fs.rename_error.lock().unwrap() = Some(std::io::ErrorKind::PermissionDenied);
    let _ = h.svc.take_rescans();
    let err = h.refused(&h.id_of(&fx.path("junk-run")), |_| {});
    assert_eq!(err.code, ErrorCode::InUse);
    let said = format!(
        "is held by R-17 (alpha, pnpm dev in {}): stop it and ask again; nothing was deleted",
        fx.root
    );
    let renamed = "could not be renamed for deletion (EPERM): ";
    assert!(err.message.contains(renamed), "{}", err.message);
    assert!(err.message.ends_with(&said), "{}", err.message);
    assert!(!err.message.contains("R-3"), "{}", err.message);
    assert!(exists_itself(&fx.path("junk-run")));
    assert!(
        h.svc.take_rescans().is_empty(),
        "a folder in use changed nothing"
    );
}

#[cfg(windows)]
#[test]
#[ignore = "e2e"]
fn a_file_held_open_without_share_delete_names_this_process() {
    use crate::ports::ProcInfo;
    use std::os::windows::fs::OpenOptionsExt;
    let (fx, h) = leftover("junk-held");
    let me = std::process::id();
    *locked(&h.procs.0) = vec![ProcInfo {
        pid: me,
        name: "the-test.exe".to_string(),
        ..ProcInfo::default()
    }];
    let file = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2)
        .open(format!("{}/a.txt", fx.path("junk-held")))
        .unwrap();
    let err = h.refused(&h.id_of(&fx.path("junk-held")), |_| {});
    drop(file);
    assert_eq!(err.code, ErrorCode::InUse);
    assert!(
        err.message.contains(&format!(
            "is held by pid {me} the-test.exe: stop it and ask again"
        )),
        "{}",
        err.message
    );
    assert!(exists_itself(&fx.path("junk-held")));
}
