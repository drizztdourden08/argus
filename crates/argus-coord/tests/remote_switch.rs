//! `coord remote [status|local|remote]` against a temp `COORD_HOME`: the texts and the file are
//! the Node verb's. The real `~/.coord/remote.json` is never touched.

mod common;

use argus_core::remote::{LOCAL, REMOTE, read_remote};
use common::{Home, coord};
use serde_json::Value;

const FROZEN: &str = "GitHub is not used: no gh, no API, no push, no publish, no GitHub registry.";

#[test]
fn without_a_file_the_status_is_local_by_default() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let ran = coord(&["remote"], home.env(), &cwd);
    assert_eq!(ran.code, 0);
    assert_eq!(
        ran.out,
        format!("remote mode: local (no file: the default while the freeze lasts). {FROZEN}\n")
    );
    assert!(!home.coord_home().join("remote.json").exists());
    assert_eq!(ran.out, coord(&["remote", "status"], home.env(), &cwd).out);
}

#[test]
fn setting_remote_writes_the_file_and_the_status_reads_it_back() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let ran = coord(&["remote", "remote", "--as", "owner"], home.env(), &cwd);
    assert_eq!(ran.code, 0);
    let file = home.coord_home().join("remote.json");
    let written: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(written["mode"], REMOTE);
    assert_eq!(written["by"], "owner");
    let since = written["since"].as_str().unwrap();
    assert_eq!(
        ran.out,
        format!(
            "remote mode: remote since {since} (set by owner). GitHub is used again. ({})\n",
            file.display()
        )
    );
    let status = coord(&["remote", "status"], home.env(), &cwd);
    assert!(status.out.starts_with(&format!(
        "remote mode: remote since {since} (set by owner). GitHub is used again."
    )));
}

#[test]
fn setting_local_again_and_a_missing_setter_fall_back_to_the_user_name() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let env = home.env_with("USERNAME", "drizz");
    assert_eq!(coord(&["remote", "remote"], env.clone(), &cwd).code, 0);
    assert_eq!(coord(&["remote", "local"], env, &cwd).code, 0);
    let state = read_remote(&home.coord_home());
    assert_eq!(state.mode, LOCAL);
    assert_eq!(state.by.as_deref(), Some("drizz"));
}

#[test]
fn an_unknown_word_is_refused_with_the_usage_and_writes_nothing() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let ran = coord(&["remote", "sideways"], home.env(), &cwd);
    assert_eq!(ran.code, 1);
    assert!(
        ran.err
            .contains("usage: coord remote [status|local|remote].")
    );
    assert!(!home.coord_home().join("remote.json").exists());
}

#[test]
fn a_garbled_file_reads_as_local() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    std::fs::write(home.coord_home().join("remote.json"), "{not json").unwrap();
    let ran = coord(&["remote"], home.env(), &cwd);
    assert!(ran.out.starts_with("remote mode: local (no file:"));
}
