//! The accounts service in shadow mode and in a preview is read-only: nothing syncs, switches,
//! saves a login or writes a scheduled task, usage is not read from the endpoint, and a login is
//! never renewed. Reads, dry runs and the cancel stay open.
//! K2, K3: shadow and preview have no outward effect, accounts included.
// test-tag: critical

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use argus_accounts::{Usage, UsageSource};
use argus_server::{ServeMode, ServeOptions};
use argus_testkit::{Request, TestOptions, TestServer, Tweak};
use serde_json::{Value, json};

struct CountingUsage(Arc<AtomicUsize>);

impl UsageSource for CountingUsage {
    fn fetch(&self, _token: &str) -> Result<Usage, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(Usage::default())
    }
}

const LOGIN: &str = r#"{"claudeAiOauth":{"accessToken":"tok","refreshToken":"ref","expiresAt":1}}"#;

fn slashed(path: &Path) -> String {
    path.display().to_string().replace(char::from(92), "/")
}

fn setup(root: &Path) -> Value {
    let cli = root.join("cli-a");
    fs::create_dir_all(&cli).expect("cli dir");
    fs::write(cli.join(".credentials.json"), LOGIN).expect("login");
    let app = root.join("app");
    fs::create_dir_all(&app).expect("app dir");
    json!({ "accounts": {
        "appData": slashed(&app),
        "cliRoot": slashed(&root.join("none")),
        "accounts": [{
            "key": "a", "label": "A", "color": "#112233",
            "cliDir": slashed(&cli),
        }],
    } })
}

fn tweak(mode: ServeMode, calls: &Arc<AtomicUsize>) -> Tweak {
    let calls = Arc::clone(calls);
    Arc::new(move |opts: &mut ServeOptions| {
        opts.mode = mode;
        opts.accounts.enabled = true;
        opts.accounts.timer = true;
        opts.accounts.usage = Some(Arc::new(CountingUsage(Arc::clone(&calls))));
    })
}

async fn start(shadow: bool, mode: ServeMode, dir: &Path, calls: &Arc<AtomicUsize>) -> TestServer {
    TestServer::start(TestOptions {
        shadow,
        config: setup(dir),
        tweak: Some(tweak(mode, calls)),
        ..TestOptions::default()
    })
    .await
}

async fn post(t: &TestServer, path: &str, body: &Value) -> (u16, String) {
    let r = t.send(t.page(Request::new("POST", path).json(body))).await;
    (r.status, r.code())
}

const REFUSED: [(&str, &str); 7] = [
    ("/api/accounts/sync", r#"{"dryRun":false}"#),
    ("/api/accounts/review/apply", r#"{"decisions":{}}"#),
    ("/api/accounts/sync/undo", "{}"),
    ("/api/accounts/switch", r#"{"to":"a"}"#),
    ("/api/accounts/switch/anyway", "{}"),
    ("/api/accounts/login/save", "{}"),
    ("/api/accounts/settings", r#"{"pollMinutes":9}"#),
];

#[tokio::test(flavor = "multi_thread")]
async fn shadow_refuses_every_accounts_write_and_reads_no_live_usage() {
    let dir = tempfile::tempdir().expect("dir");
    let calls = Arc::new(AtomicUsize::new(0));
    let t = start(true, ServeMode::Shadow, dir.path(), &calls).await;
    let config_before = fs::read_to_string(t.svc.paths.config.clone()).ok();
    for (path, body) in REFUSED {
        let body: Value = serde_json::from_str(body).expect("json");
        assert_eq!(
            post(&t, path, &body).await,
            (403, "read_only".to_owned()),
            "{path}"
        );
    }
    let cancel = post(&t, "/api/accounts/switch/cancel", &json!({})).await;
    assert_eq!(cancel.0, 200, "cancel stays open");
    assert_eq!(t.get("/api/accounts/settings").await.status, 200);
    let refresh = post(&t, "/api/accounts/usage/refresh", &json!({})).await;
    assert_eq!(refresh.0, 200, "a refresh reads the Desktop log");
    let dry = post(&t, "/api/accounts/sync", &json!({ "dryRun": true })).await;
    assert_ne!(dry.0, 403, "a dry run is not refused");
    assert_eq!(calls.load(Ordering::SeqCst), 0, "no usage endpoint call");
    let kept = fs::read_to_string(dir.path().join("cli-a/.credentials.json")).expect("login");
    assert_eq!(kept, LOGIN, "no login renewed");
    let config_after = fs::read_to_string(t.svc.paths.config.clone()).ok();
    assert_eq!(config_before, config_after, "no config written");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_preview_writes_its_settings_but_applies_no_sync_or_switch() {
    let dir = tempfile::tempdir().expect("dir");
    let calls = Arc::new(AtomicUsize::new(0));
    let t = start(true, ServeMode::Preview, dir.path(), &calls).await;
    for (path, body) in REFUSED.iter().filter(|(p, _)| !p.ends_with("/settings")) {
        let body: Value = serde_json::from_str(body).expect("json");
        assert_eq!(
            post(&t, path, &body).await,
            (403, "read_only".to_owned()),
            "{path}"
        );
    }
    let saved = post(&t, "/api/accounts/settings", &json!({ "pollMinutes": 9 })).await;
    assert_eq!(saved.0, 200, "the preview home is a copy");
    assert_eq!(
        post(&t, "/api/accounts/usage/refresh", &json!({})).await.0,
        200
    );
    if std::env::var("ARGUS_PREVIEW_ACCOUNTS_USAGE").as_deref() != Ok("1") {
        assert_eq!(
            calls.load(Ordering::SeqCst),
            0,
            "no live usage without the opt-in"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_live_server_still_reads_usage() {
    let dir = tempfile::tempdir().expect("dir");
    let calls = Arc::new(AtomicUsize::new(0));
    let t = start(false, ServeMode::Live, dir.path(), &calls).await;
    assert_eq!(
        post(&t, "/api/accounts/usage/refresh", &json!({})).await.0,
        200
    );
    assert!(calls.load(Ordering::SeqCst) > 0, "the endpoint was asked");
}
