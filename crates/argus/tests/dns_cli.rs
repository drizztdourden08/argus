//! `argus dns` end to end: the real binary on throwaway homes. The scripts are only printed. The
//! status test asks an in-process server whose responder and door sit on port 0, never 53 or 80,
//! and the owner's 47615 is never touched.

mod common;

use std::path::Path;
use std::process::Command;

use argus_testkit::{TestOptions, TestServer};
use common::Out;
use serde_json::{Value, json};
use tempfile::TempDir;

const ADDRESS: &str = "127.65.82.73";

fn argus(coord_home: &Path, home: &Path, args: &[&str]) -> Out {
    Command::new(env!("CARGO_BIN_EXE_argus"))
        .args(args)
        .env("COORD_HOME", coord_home)
        .env("DASHBOARD_HOME", home)
        .env_remove("DASHBOARD_PORT")
        .output()
        .expect("argus runs")
        .into()
}

fn home_with(config: &Value) -> TempDir {
    let dir = tempfile::tempdir().expect("a temp home");
    if !config.is_null() {
        std::fs::write(dir.path().join("config.json"), config.to_string()).expect("a config");
    }
    dir
}

fn dns(home: &TempDir, args: &[&str]) -> Out {
    let mut line = vec!["dns"];
    line.extend_from_slice(args);
    argus(home.path(), home.path(), &line)
}

#[test]
fn install_and_remove_print_the_administrator_script_and_change_nothing() {
    let home = home_with(&Value::Null);
    let install = dns(&home, &["install", "--print"]);
    assert_eq!(install.code, 0, "{}", install.stderr);
    assert!(
        install
            .stdout
            .contains("Add-DnsClientNrptRule -Namespace \".argus\" -NameServers \"127.65.82.71\"")
    );
    assert!(install.stdout.contains("Chrome and Edge honour NRPT"));
    assert!(install.stdout.contains("http://argus.argus/"));
    let remove = dns(&home, &["remove", "--print"]);
    assert_eq!(remove.code, 0, "{}", remove.stderr);
    assert!(remove.stdout.contains("Remove-DnsClientNrptRule"));
    assert!(
        !home.path().join("config.json").exists(),
        "no config was written"
    );
}

#[test]
fn the_script_follows_the_suffix_and_address_of_the_config() {
    let config = json!({ "localNames": { "suffix": "home.argus", "address": ADDRESS } });
    let home = home_with(&config);
    let text = dns(&home, &["install", "--print"]).stdout;
    assert!(text.contains(&format!(
        "Add-DnsClientNrptRule -Namespace \".home.argus\" -NameServers \"{ADDRESS}\""
    )));
}

#[test]
fn nothing_is_printed_or_run_without_print_and_a_refused_key_is_named() {
    let home = home_with(&Value::Null);
    assert_eq!(dns(&home, &["install"]).code, 2);
    assert_eq!(dns(&home, &["remove"]).code, 2);
    let bad = home_with(&json!({ "localNames": { "address": "10.0.0.1" } }));
    let refused = dns(&bad, &["install", "--print"]);
    assert_ne!(refused.code, 0);
    assert!(
        refused.stderr.contains("localNames.address"),
        "{}",
        refused.stderr
    );
}

#[test]
fn status_says_off_until_the_config_turns_the_names_on() {
    let home = home_with(&Value::Null);
    let status = dns(&home, &["status"]);
    assert_eq!(status.code, 0, "{}", status.stderr);
    assert!(
        status.stdout.contains("local names: off"),
        "{}",
        status.stdout
    );
}

#[test]
fn status_with_the_names_on_and_no_server_says_the_dns_is_silent() {
    let config = json!({ "localNames": { "enabled": true, "address": ADDRESS } });
    let home = home_with(&config);
    let status = dns(&home, &["status"]);
    assert_eq!(status.code, 1, "{}", status.stderr);
    assert!(
        status.stdout.contains("local names: on"),
        "{}",
        status.stdout
    );
    assert!(
        status
            .stdout
            .contains(&format!("dns: not answering at {ADDRESS}:53"))
    );
    assert!(status.stdout.contains("the server is not running"));
}

#[tokio::test(flavor = "multi_thread")]
async fn status_asks_the_real_responder_of_a_running_server() {
    let options = TestOptions {
        sync: false,
        config: json!({ "localNames": { "enabled": true, "address": "127.65.82.74" } }),
        tweak: Some(std::sync::Arc::new(|o| {
            o.local_names.dns_port = 0;
            o.local_names.door_port = 0;
            o.localhost_door_port = Some(0);
        })),
        ..TestOptions::default()
    };
    let t = TestServer::start(options).await;
    let live = t.svc.names().expect("the names are up").clone();
    let (coord, home) = (t.coord_home.clone(), t.dashboard_home.clone());
    let status = tokio::task::spawn_blocking(move || argus(&coord, &home, &["dns", "status"]))
        .await
        .expect("the verb ends");
    assert_eq!(status.code, 0, "{}{}", status.stdout, status.stderr);
    assert!(
        status
            .stdout
            .contains(&format!("dns: answering at {}", live.dns))
    );
    assert!(
        status
            .stdout
            .contains(&format!("front door: port {}", live.door.port))
    );
    let door = t.svc.localhost_door().expect("the localhost door is bound");
    assert!(
        status
            .stdout
            .contains(&format!("localhost door: port {door}"))
    );
    t.stop(false).await;
}
