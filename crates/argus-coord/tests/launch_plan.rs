//! How the CLI brings the dashboard up, as plans: the program, the arguments and the variables,
//! checked without starting anything (a started tray would put an icon on the desktop). Covers
//! the client autostart and the session hooks, which share the plans.

mod common;

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use argus_coord::client::launch::plan_for;
use argus_core::Paths;
use common::Home;
use serde_json::json;

/// A Node install with a `bin/dashboard.mjs`.
fn node_install(home: &Home) -> PathBuf {
    let install = home.folder("install");
    fs::create_dir_all(install.join("bin")).unwrap();
    fs::write(install.join("bin").join("dashboard.mjs"), "").unwrap();
    install
}

fn strings(args: &[OsString]) -> Vec<String> {
    args.iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect()
}

fn paths_of(home: &Home, dashboard: &Path) -> Paths {
    Paths::new(home.coord_home(), Some(dashboard.to_path_buf()))
}

/// A home whose config carries `shadow` as given, names an existing Argus binary and a Node
/// install. Returns the home and the binary.
fn home_with_marker(home: &Home, shadow: bool) -> (PathBuf, PathBuf) {
    let install = node_install(home);
    let exe = home.path().join("argus.exe");
    fs::write(&exe, "").unwrap();
    let config = json!({ "shadow": shadow, "exe": exe, "installDir": install });
    (shadow_home(home, &config), exe)
}

fn shadow_home(home: &Home, config: &serde_json::Value) -> PathBuf {
    let shadow = home.folder("shadow");
    fs::create_dir_all(&shadow).unwrap();
    fs::write(shadow.join("config.json"), config.to_string()).unwrap();
    shadow
}

#[test]
fn a_node_home_starts_through_the_node_tray_on_windows_and_plain_elsewhere() {
    let home = Home::new();
    let install = node_install(&home);
    let dash = home.dashboard_home();
    fs::write(
        dash.join("config.json"),
        json!({ "installDir": install }).to_string(),
    )
    .unwrap();
    let paths = paths_of(&home, &dash);
    let bin = install.join("bin").join("dashboard.mjs");
    let on_windows = plan_for(&home.env(), &paths, true).unwrap();
    assert_eq!(on_windows.program, PathBuf::from("node"));
    assert_eq!(
        strings(&on_windows.args),
        [bin.to_string_lossy().as_ref(), "tray", "start"]
    );
    let elsewhere = plan_for(&home.env(), &paths, false).unwrap();
    assert_eq!(
        strings(&elsewhere.args),
        [bin.to_string_lossy().as_ref(), "start"]
    );
}

#[test]
fn a_shadow_home_starts_the_argus_tray_in_shadow_mode_never_the_node_bin() {
    let home = Home::new();
    let (shadow, exe) = home_with_marker(&home, true);
    let paths = paths_of(&home, &shadow);
    let plan = plan_for(&home.env(), &paths, true).unwrap();
    assert_eq!(plan.program, exe);
    let at = shadow.to_string_lossy().into_owned();
    assert_eq!(
        strings(&plan.args),
        ["tray", "start", "--data-dir", at.as_str(), "--shadow"]
    );
    assert_eq!(plan.vars, [("DASHBOARD_HOME".to_owned(), at.clone())]);
    let bare = plan_for(&home.env(), &paths, false).unwrap();
    assert_eq!(
        (bare.program, strings(&bare.args)),
        (exe, vec!["start".to_owned()])
    );
}

#[test]
fn a_shadow_home_with_no_argus_binary_starts_nothing() {
    let home = Home::new();
    let install = node_install(&home);
    let missing = home.path().join("gone").join("argus.exe");
    for config in [
        json!({ "shadow": true, "installDir": install }),
        json!({ "shadow": true, "exe": missing, "installDir": install }),
    ] {
        let shadow = shadow_home(&home, &config);
        let paths = paths_of(&home, &shadow);
        assert_eq!(plan_for(&home.env(), &paths, true), None);
    }
}

#[test]
fn only_a_true_marker_makes_a_home_a_shadow() {
    let home = Home::new();
    let (shadow, _exe) = home_with_marker(&home, false);
    let plan = plan_for(&home.env(), &paths_of(&home, &shadow), true).unwrap();
    assert_eq!(plan.program, PathBuf::from("node"));
}
