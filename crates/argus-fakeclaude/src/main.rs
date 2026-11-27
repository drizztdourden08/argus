//! A fake Claude Desktop for the headless account switch tests. It holds the lock of its Local
//! Storage like the real app, signs in as the account of `config.json`, reconciles its sidebar
//! with the server copy, rewrites `config.json` and runs until a quit file appears (or, with
//! `--tray`, until it is killed). It touches nothing outside the folders it is given.

mod args;
mod reconcile;
mod storage;

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use args::Args;
use serde_json::Value;
use storage::Storage;

/// How often the app looks for the quit file.
const POLL: Duration = Duration::from_millis(50);
/// A forgotten fake ends by itself after this long, so a failed test leaves no lock behind.
const MAX_LIFE: Duration = Duration::from_secs(180);

fn now_ms() -> i64 {
    let since = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(since.as_millis()).unwrap_or(i64::MAX)
}

fn config_of(app: &Path) -> Result<Value, String> {
    let file = app.join("config.json");
    let text = fs::read_to_string(&file).map_err(|e| format!("cannot read config.json: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("config.json is not JSON: {e}"))
}

/// The account `config.json` says is logged in.
fn signed_in(config: &Value) -> Result<String, String> {
    config
        .get("lastKnownAccountUuid")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "nobody is signed in".to_owned())
}

/// Rewrites `config.json` as the app does once it has signed in.
fn write_config(app: &Path, config: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(app.join("config.json"), text).map_err(|e| format!("cannot write config.json: {e}"))
}

/// A failed start: the marker is consumed and the app exits without holding the lock.
fn fails_now(args: &Args) -> bool {
    args.fail_once
        .as_ref()
        .is_some_and(|marker| fs::remove_file(marker).is_ok())
}

/// Runs until the quit file appears; a tray app never quits that way.
fn wait_for_quit(args: &Args) {
    let started = Instant::now();
    while started.elapsed() < MAX_LIFE {
        if !args.tray && args.quit_file.exists() {
            return;
        }
        std::thread::sleep(POLL);
    }
}

fn run() -> Result<(), String> {
    let args = args::parse(std::env::args().skip(1))?;
    if fails_now(&args) {
        return Err("the sign-in failed".to_owned());
    }
    let config = config_of(&args.app)?;
    let account = signed_in(&config)?;
    let org = args.org_of(&account)?.to_owned();
    let mut store = Storage::open(&args.app)?;
    let keys = reconcile::run(&mut store, &args.remote, (&account, &org))?;
    store.write_keys(&keys, now_ms())?;
    write_config(&args.app, &config)?;
    wait_for_quit(&args);
    store.write_keys(&keys, now_ms())?;
    store.close()
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("argus-fakeclaude: {why}");
            ExitCode::FAILURE
        }
    }
}
