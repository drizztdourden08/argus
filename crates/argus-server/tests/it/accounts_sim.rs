//! The account switch end to end without the real Claude: the real switch code (close, save the
//! login, sync the sidebars, restore the login, start, confirm, sidebar check) runs against a
//! fake Claude Desktop, a child process (`argus-fakeclaude`) that holds the lock of a made-up
//! Local Storage and reconciles its sidebar the way Claude does. Nothing of the owner's Claude,
//! app data or logins is read, written or started.
// test-tag: standard

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use argus_accounts::desktop::{Launch, Limits, ProcessApp};
use argus_accounts::{AuthProbe, CliStatus, Credentials, DesktopApp, Usage, UsageSource};
use argus_core::Coll;
use argus_core::ids::PORTFOLIO;
use argus_core::process::hidden_command;
use argus_server::{ServeMode, ServeOptions};
use argus_testkit::{Request, TestOptions, TestServer, Tweak};
use serde_json::{Value, json};

use super::sim_home::{ACCOUNT_A, ACCOUNT_B, ORG_A, ORG_B, SECOND, SimHome};
use super::wait::until;

struct NoProbe;

impl AuthProbe for NoProbe {
    fn status(&self, _exe: &Path, _config_dir: &Path) -> Option<CliStatus> {
        None
    }
}

struct FlatUsage;

impl UsageSource for FlatUsage {
    fn fetch(&self, _access_token: &str) -> Result<Usage, String> {
        Ok(Usage::default())
    }
}

struct FakeLogins;

impl Credentials for FakeLogins {
    fn access_token(&self, _cli_dir: &Path) -> Result<String, String> {
        Ok("fake-access".to_owned())
    }

    fn expires_at(&self, _cli_dir: &Path) -> Option<i64> {
        None
    }

    fn refresh(&self, _cli_dir: &Path) -> Result<String, String> {
        Ok("fake-access".to_owned())
    }
}

/// The built fake app. The test binary belongs to `argus-server`, so Cargo does not hand it the
/// path of another package's binary; it sits in the profile folder next to `deps`, and a plain
/// `cargo build` of the package makes sure it is there and current.
fn fake_exe() -> PathBuf {
    static EXE: OnceLock<PathBuf> = OnceLock::new();
    EXE.get_or_init(|| {
        let test_exe = std::env::current_exe().expect("test exe");
        let profile = test_exe
            .parent()
            .and_then(Path::parent)
            .expect("profile dir");
        let name = format!("argus-fakeclaude{}", std::env::consts::EXE_SUFFIX);
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
        let mut build = hidden_command(cargo);
        build.args(["build", "--quiet", "-p", "argus-fakeclaude"]);
        if profile.file_name().is_some_and(|n| n == "release") {
            build.arg("--release");
        }
        assert!(
            build.status().expect("cargo runs").success(),
            "the fake app builds"
        );
        profile.join(name)
    })
    .clone()
}

fn text(path: &Path) -> String {
    path.display().to_string()
}

fn launch(home: &SimHome, tray: bool) -> Launch {
    let mut args = vec![
        "--app".to_owned(),
        text(&home.app),
        "--remote".to_owned(),
        text(&home.remote),
        "--quit-file".to_owned(),
        text(&home.quit_file),
        "--fail-once".to_owned(),
        text(&home.fail_marker),
    ];
    for (account, org) in [(ACCOUNT_A, ORG_A), (ACCOUNT_B, ORG_B)] {
        args.extend(["--org".to_owned(), format!("{account}={org}")]);
    }
    if tray {
        args.push("--tray".to_owned());
    }
    Launch {
        exe: fake_exe(),
        args,
        quit_file: home.quit_file.clone(),
        version: Some("9.9.9-fake".to_owned()),
    }
}

/// A tray app ignores the quit file, so its close waits only a little before giving up.
fn limits(tray: bool) -> Limits {
    let (grace, limit) = if tray { (1, 3) } else { (5, 10) };
    Limits {
        grace: Duration::from_secs(grace),
        limit: Duration::from_secs(limit),
        start: Duration::from_secs(15),
        poll: Duration::from_millis(50),
    }
}

/// Everything outside the temp folder is cut off: the user's `.claude` is the temp `user`
/// folder, the coord home and boards are the server's throwaway ones, and no toast is shown.
fn tweak(app: &Arc<ProcessApp>, user_home: PathBuf) -> Tweak {
    let app = Arc::clone(app);
    Arc::new(move |opts: &mut ServeOptions| {
        opts.mode = ServeMode::Live;
        opts.notify.enabled = false;
        opts.notify.system = false;
        let accounts = &mut opts.accounts;
        accounts.enabled = true;
        accounts.timer = false;
        accounts.probe = Some(Arc::new(NoProbe));
        accounts.usage = Some(Arc::new(FlatUsage));
        accounts.credentials = Some(Arc::new(FakeLogins));
        accounts.sidebar_settle = Some(Duration::from_millis(300));
        accounts.user_home = Some(user_home.clone());
        let shared: Arc<dyn DesktopApp> = app.clone();
        accounts.desktop = Some(Arc::new(move |_app: &Path| -> Arc<dyn DesktopApp> {
            Arc::clone(&shared)
        }));
    })
}

/// A server over a made-up home, with the fake Claude running as the first account.
struct Rig {
    t: TestServer,
    app: Arc<ProcessApp>,
    home: SimHome,
    _dir: tempfile::TempDir,
}

impl Drop for Rig {
    fn drop(&mut self) {
        self.app.kill();
    }
}

async fn rig(tray: bool, force_quit: bool) -> Rig {
    let dir = tempfile::tempdir().expect("dir");
    let user_home = dir.path().join("user");
    let dashboard = dir.path().join("dashboard");
    let logins = dashboard.join("accounts").join("logins");
    let home = SimHome::build(dir.path(), &logins);
    let app = Arc::new(ProcessApp::new(
        &home.app,
        launch(&home, tray),
        limits(tray),
    ));
    app.start().expect("the fake Claude starts");
    let t = TestServer::start(TestOptions {
        config: home.accounts_setting(force_quit),
        dashboard_home: Some(dashboard),
        tweak: Some(tweak(&app, user_home)),
        ..TestOptions::default()
    })
    .await;
    Rig {
        t,
        app,
        home,
        _dir: dir,
    }
}

fn switch_doc(t: &TestServer) -> Option<Value> {
    let row = t
        .svc
        .db
        .docs()
        .get(PORTFOLIO, Coll::AccountsState, "switch");
    row.ok().flatten().map(|row| row.data)
}

fn ended(doc: &Value) -> bool {
    matches!(doc["phase"].as_str(), Some("done" | "failed" | "cancelled"))
}

async fn ask_switch(r: &Rig, to: &str, handover: bool) {
    let body = json!({ "to": to, "handover": handover });
    let ask = Request::new("POST", "/api/accounts/switch").json(&body);
    let reply = r.t.send(r.t.page(ask)).await;
    assert_eq!(reply.status, 202, "{}", reply.text());
}

/// Asks for the switch as the page does and waits (up to twenty seconds) for its doc to end.
async fn switch_to(r: &Rig, to: &str) -> Value {
    ask_switch(r, to, false).await;
    wait_end(r).await
}

async fn wait_end(r: &Rig) -> Value {
    let t = &r.t;
    until("the switch to end", || async move {
        switch_doc(t).filter(ended)
    })
    .await
}

fn error_of(doc: &Value) -> String {
    doc["error"].as_str().unwrap_or_default().to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_switch_closes_the_app_swaps_the_login_and_starts_it_as_the_other_account() {
    let r = rig(false, true).await;
    assert_eq!(r.home.signed_in(), ACCOUNT_A);
    let doc = switch_to(&r, SECOND).await;
    // The sidebar check reads the store after the fake Claude reconciled it at its start.
    // I-259 (flags.rs, branch i259a): drift is 0 only once that branch is merged; it passes only when Argus leaves the flags Claude reads (`ccd-sync-owner` and
    // `ccd-sync-org` set to the target, `ccd-sync-pending:ccd/dframe-store` "1", the pending
    // base removed). Without them the fake takes the server copy and the check fails with the
    // folders of the first account missing again.
    assert_eq!(doc["phase"], "done", "the switch doc: {doc}");
    assert_eq!(error_of(&doc), "", "the check result: {doc}");
    assert_eq!(r.home.signed_in(), ACCOUNT_B);
    assert!(r.app.is_running(), "Claude runs again");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_start_that_fails_puts_the_old_login_back_and_starts_the_old_account_again() {
    let r = rig(false, true).await;
    std::fs::write(&r.home.fail_marker, "fail").expect("marker");
    let doc = switch_to(&r, SECOND).await;
    assert_eq!(doc["phase"], "failed", "the switch doc: {doc}");
    assert!(
        error_of(&doc).contains("started again as before"),
        "the rollback is told: {doc}"
    );
    assert_eq!(r.home.signed_in(), ACCOUNT_A, "the old login is back");
    assert!(r.app.is_running(), "the old account runs again");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_tray_app_that_stays_up_fails_the_switch_without_force_quit() {
    let r = rig(true, false).await;
    let doc = switch_to(&r, SECOND).await;
    assert_eq!(doc["phase"], "failed", "the switch doc: {doc}");
    assert!(error_of(&doc).contains("did not quit"), "{doc}");
    assert_eq!(r.home.signed_in(), ACCOUNT_A, "nothing was swapped");
    assert!(r.app.is_running(), "the app was left alone");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_tray_app_is_ended_by_its_own_pid_with_force_quit_and_the_switch_goes_on() {
    let r = rig(true, true).await;
    let doc = switch_to(&r, SECOND).await;
    assert!(
        !error_of(&doc).contains("did not quit"),
        "the forced close worked: {doc}"
    );
    assert_eq!(
        r.home.signed_in(),
        ACCOUNT_B,
        "the login was swapped: {doc}"
    );
    assert!(r.app.is_running(), "Claude was started again");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_switch_with_handover_asks_the_fake_lead_in_the_temp_home_and_goes_on_when_cut_off() {
    let r = rig(false, true).await;
    ask_switch(&r, SECOND, true).await;
    let t = &r.t;
    let waiting = until("the handover wait or the end", || async move {
        switch_doc(t).filter(|d| ended(d) || d["phase"] == "handover")
    })
    .await;
    if !ended(&waiting) {
        let cut = Request::new("POST", "/api/accounts/switch/anyway");
        assert_eq!(r.t.send(r.t.page(cut)).await.status, 200);
    }
    let doc = wait_end(&r).await;
    assert_eq!(doc["phase"], "done", "the switch doc: {doc}");
    assert_eq!(r.home.signed_in(), ACCOUNT_B);
}
