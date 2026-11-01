//! `coord remote [status|local|remote]` and `coord remote ci-root` against a temp `COORD_HOME`:
//! the texts and the file are the Node verb's. The real `~/.coord/remote.json` is never touched.

mod common;

use argus_core::remote::{ci_root_of, ci_roots};
use argus_core::remote_state::{LOCAL, REMOTE, read_state};
use common::{Home, coord};
use serde_json::Value;

const FROZEN: &str = "GitHub is not used: no gh, no API, no push, no publish, no GitHub registry.";

fn file_of(home: &Home) -> Value {
    let text = std::fs::read_to_string(home.coord_home().join("remote.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn without_a_file_the_status_is_local_by_default() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let ran = coord(&["remote"], home.env(), &cwd);
    assert_eq!(ran.code, 0);
    assert_eq!(
        ran.out,
        format!(
            "remote mode: local (no file: the default while the freeze lasts). {FROZEN}\nno remote sync recorded yet.\n"
        )
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
    let written = file_of(&home);
    assert_eq!(written["mode"], REMOTE);
    assert_eq!(written["by"], "owner");
    assert!(written.get("ciRoots").is_none());
    let since = written["since"].as_str().unwrap();
    let file = home.coord_home().join("remote.json");
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
    let state = read_state(&home.coord_home());
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
            .contains("usage: coord remote [status|local|remote|synced|ci-root <project> <dir>].")
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

#[test]
fn ci_root_keeps_the_checkout_and_the_status_lists_it() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let lag = home.folder("lagging");
    let dir = lag.to_string_lossy().replace('\\', "/");
    let set = coord(
        &["remote", "ci-root", "brock", lag.to_str().unwrap()],
        home.env(),
        &cwd,
    );
    assert_eq!(set.code, 0, "{}", set.err);
    assert_eq!(
        set.out,
        format!("ci-root of brock: {dir} (coord check and coord pr merge run Brock there).\n")
    );
    let written = file_of(&home);
    assert_eq!(written["mode"], LOCAL);
    assert_eq!(written["ciRoots"]["brock"], dir.as_str());
    assert!(written["since"].is_string());
    assert!(written["by"].is_null());
    let status = coord(&["remote"], home.env(), &cwd);
    assert!(
        status.out.contains(&format!("\n  ci-root brock: {dir}\n")),
        "{}",
        status.out
    );
    assert_eq!(
        ci_roots(&home.coord_home()),
        vec![("brock".into(), dir.clone())]
    );
    assert_eq!(ci_root_of(&home.coord_home(), "brock"), Some(dir));
    assert_eq!(ci_root_of(&home.coord_home(), "argus"), None);
}

#[test]
fn switching_the_mode_keeps_the_roots_and_clear_drops_one() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let (one, two) = (home.folder("one"), home.folder("two"));
    for (key, dir) in [("alpha", &one), ("beta", &two)] {
        let ran = coord(
            &["remote", "ci-root", key, dir.to_str().unwrap()],
            home.env(),
            &cwd,
        );
        assert_eq!(ran.code, 0, "{}", ran.err);
    }
    let env = home.env();
    assert_eq!(
        coord(&["remote", "remote", "--as", "o"], env.clone(), &cwd).code,
        0
    );
    let after = file_of(&home);
    assert_eq!(after["mode"], REMOTE);
    assert_eq!(after["ciRoots"].as_object().unwrap().len(), 2);
    assert_eq!(
        after.as_object().unwrap().keys().collect::<Vec<_>>(),
        ["mode", "since", "by", "ciRoots"]
    );
    let cleared = coord(
        &["remote", "ci-root", "alpha", "--clear"],
        env.clone(),
        &cwd,
    );
    assert_eq!(
        cleared.out,
        "ci-root of alpha cleared: the project root is used again.\n"
    );
    let state = read_state(&home.coord_home());
    assert_eq!(
        (state.mode.as_str(), state.by.as_deref()),
        (REMOTE, Some("o"))
    );
    assert_eq!(state.ci_roots.len(), 1);
    coord(&["remote", "ci-root", "beta", "--clear"], env, &cwd);
    assert!(file_of(&home).get("ciRoots").is_none());
}

#[test]
fn ci_root_refuses_a_bad_project_a_missing_folder_and_a_missing_dir() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let usage = "usage: coord remote ci-root <project> <dir> | --clear.";
    for argv in [
        &["remote", "ci-root"][..],
        &["remote", "ci-root", "Bad_Key", "x"][..],
        &["remote", "ci-root", "brock"][..],
    ] {
        let ran = coord(argv, home.env(), &cwd);
        assert_eq!(ran.code, 1, "{argv:?}");
        assert!(ran.err.contains(usage), "{}", ran.err);
    }
    let gone = coord(&["remote", "ci-root", "brock", "nowhere"], home.env(), &cwd);
    assert_eq!(gone.code, 1);
    assert!(gone.err.contains("nowhere does not exist."), "{}", gone.err);
    assert!(!home.coord_home().join("remote.json").exists());
}

#[test]
fn synced_records_the_project_sync_and_the_status_lists_it() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let note = "pushed 2 PRs, released 0.1.0";
    let argv = ["remote", "synced", "--project", "demo", "--note", note];
    let ran = coord(&argv, home.env(), &cwd);
    assert_eq!(ran.code, 0, "{}", ran.err);
    let record = file_of(&home)["syncs"]["demo"].clone();
    let at = record["at"].as_str().unwrap();
    assert!(argus_core::clock::parse_ms(at).is_some());
    assert_eq!(record["by"], "demo");
    assert_eq!(record["note"], note);
    assert_eq!(
        ran.out,
        format!("remote sync of demo recorded at {at}: {note}. Due again in 24 h.\n")
    );
    let status = coord(&["remote", "status"], home.env(), &cwd);
    let listed = format!("\nlast remote sync per lead:\n  demo: {at} ({note})\n");
    assert!(status.out.ends_with(&listed), "{}", status.out);
    assert_eq!(read_state(&home.coord_home()).syncs.len(), 1);
}

#[test]
fn a_sync_without_a_note_says_so_and_a_mode_switch_keeps_the_records() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let argv = ["remote", "synced", "--project", "demo", "--as", "lead-1"];
    let ran = coord(&argv, home.env(), &cwd);
    assert_eq!(ran.code, 0, "{}", ran.err);
    let record = file_of(&home)["syncs"]["demo"].clone();
    assert_eq!(record["by"], "lead-1");
    assert_eq!(record["note"], "");
    let at = record["at"].as_str().unwrap();
    assert_eq!(
        ran.out,
        format!("remote sync of demo recorded at {at}. Due again in 24 h.\n")
    );
    let env = home.env();
    let to_remote = coord(&["remote", "remote", "--as", "owner"], env.clone(), &cwd);
    assert_eq!(to_remote.code, 0);
    assert_eq!(coord(&["remote", "local"], env, &cwd).code, 0);
    assert_eq!(file_of(&home)["syncs"]["demo"], record, "a switch keeps it");
}

#[test]
fn a_long_note_is_cut_to_400_characters() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let long = "x".repeat(450);
    let argv = ["remote", "synced", "--project", "demo", "--note", long.as_str()];
    assert_eq!(coord(&argv, home.env(), &cwd).code, 0);
    let note = file_of(&home)["syncs"]["demo"]["note"].clone();
    assert_eq!(note.as_str().unwrap().len(), 400);
}

#[test]
fn synced_without_a_project_is_refused_and_writes_nothing() {
    let home = Home::new();
    let cwd = home.folder("checkout");
    let ran = coord(&["remote", "synced"], home.env(), &cwd);
    assert_eq!(ran.code, 1);
    assert!(!home.coord_home().join("remote.json").exists());
}
