//! What the server client tests share: an Argus test server on an ephemeral port, the temp homes
//! of its `COORD_HOME` and `DASHBOARD_HOME` as the environment of in-process `coord` calls, and
//! boards written as files. Each test crate uses only part of it.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use argus_coord::Env;
use argus_testkit::{TestOptions, TestServer, Tweak, Wakes};
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::common::{Ran, ThreadSpec, board_with, coord};

pub const LEAD: &str = "sess-lead";
pub const MAIN: &str = "sess-main";

/// A test server, its homes, and a folder for checkouts.
pub struct Rig {
    pub t: TestServer,
    pub dir: TempDir,
    _coord: TempDir,
}

/// The config every rig starts with: OS toasts off, so no ping ever shows a window.
fn quiet(extra: &Value, silenced: bool) -> Value {
    let mut config = if silenced {
        json!({})
    } else {
        json!({ "notify": { "os": false } })
    };
    if let Some(more) = extra.as_object() {
        for (k, v) in more {
            config[k] = v.clone();
        }
    }
    config
}

impl Rig {
    /// A server with the given config merged over the quiet one, started on a coord home that
    /// `seed(coord_home, folder)` filled first.
    pub async fn start(extra: Value, seed: impl FnOnce(&Path, &Path)) -> Rig {
        Rig::start_with(extra, seed, None).await
    }

    /// [`Rig::start`] with a last say on the serve options (a fake toaster, a fake machine). The
    /// tweak is then in charge of OS toasts.
    pub async fn start_with(
        extra: Value,
        seed: impl FnOnce(&Path, &Path),
        tweak: Option<Tweak>,
    ) -> Rig {
        Rig::start_all(extra, seed, tweak, Wakes::Live).await
    }

    /// [`Rig::start`] with the wake engine in the test's hands: no timer, a manual clock, moved by
    /// `rig.t.advance_wakes(ms)`.
    pub async fn start_manual_wakes(extra: Value, seed: impl FnOnce(&Path, &Path)) -> Rig {
        Rig::start_all(extra, seed, None, Wakes::Manual).await
    }

    async fn start_all(
        extra: Value,
        seed: impl FnOnce(&Path, &Path),
        tweak: Option<Tweak>,
        wakes: Wakes,
    ) -> Rig {
        let (coord, dir) = (
            TempDir::new().expect("coord"),
            TempDir::new().expect("folder"),
        );
        seed(coord.path(), dir.path());
        let options = TestOptions {
            config: quiet(&extra, tweak.is_some()),
            coord_home: Some(coord.path().to_path_buf()),
            tweak,
            wakes,
            ..TestOptions::default()
        };
        Rig {
            t: TestServer::start(options).await,
            dir,
            _coord: coord,
        }
    }

    /// The environment of a call: the server's homes and, when given, a session id.
    pub fn env(&self, session: Option<&str>) -> Env {
        let coord = self.t.coord_home.to_string_lossy().into_owned();
        let dash = self.t.dashboard_home.to_string_lossy().into_owned();
        let mut pairs = vec![
            ("COORD_HOME", coord.as_str()),
            ("DASHBOARD_HOME", dash.as_str()),
        ];
        if let Some(session) = session {
            pairs.push(("CLAUDE_CODE_SESSION_ID", session));
        }
        Env::of(&pairs)
    }

    /// Runs `coord` as a session, from `cwd`.
    pub fn run_in(&self, argv: &[&str], session: Option<&str>, cwd: &Path) -> Ran {
        coord(argv, self.env(session), cwd)
    }

    /// Runs `coord` with no session, from the rig folder.
    pub fn run(&self, argv: &[&str]) -> Ran {
        self.run_in(argv, None, self.dir.path())
    }

    /// The same rig with its server stopped, homes kept for a look.
    pub async fn stop(self) {
        self.t.stop(false).await;
    }
}

/// What a call printed, as lines.
pub fn lines(text: &str) -> Vec<&str> {
    text.lines().map(str::trim_end).collect()
}

/// A call that ended with this code and wrote the text on stderr.
#[track_caller]
pub fn refused(ran: &Ran, code: i32, text: &str) {
    assert_eq!(ran.code, code, "out: {} err: {}", ran.out, ran.err);
    assert!(
        ran.err.contains(text),
        "stderr {:?} lacks {text:?}",
        ran.err
    );
}

/// A call that succeeded.
#[track_caller]
pub fn ok(ran: &Ran) -> &str {
    assert_eq!(ran.code, 0, "out: {} err: {}", ran.out, ran.err);
    &ran.out
}

/// A checkout of a project under `base`: a folder with an `.ai-config.json` naming it.
pub fn make_checkout(base: &Path, key: &str) -> PathBuf {
    let dir = base.join(format!("checkout-{key}"));
    fs::create_dir_all(&dir).expect("checkout");
    let text = json!({ "projectKey": key }).to_string();
    fs::write(dir.join(".ai-config.json"), text).expect("ai-config");
    dir
}

/// A board of a project whose lead (a lead thread named `lead`) works in `root`.
pub fn lead_board(key: &str, root: &Path) -> Value {
    board_with(key, &[("lead", "lead", "active", None, Some(root))])
}

/// The board file of a project under a coord home, parsed.
pub fn read_board_at(coord_home: &Path, key: &str) -> Value {
    let text = fs::read_to_string(coord_home.join(key).join("board.json")).expect("board file");
    serde_json::from_str(&text).expect("board json")
}

/// An environment with more variables than `env` holds.
pub fn env_plus(env: &Env, extra: &[(&str, &str)]) -> Env {
    let mut merged: std::collections::BTreeMap<&str, &str> = env.iter().collect();
    merged.extend(extra.iter().copied());
    let pairs: Vec<(&str, &str)> = merged.into_iter().collect();
    Env::of(&pairs)
}

/// A board of `alpha` with a lead that has no folder and a worker working in `root`.
pub fn alpha_with_worker(root: &Path) -> Value {
    let threads = [
        ("lead", "lead", "active", None, None),
        ("worker", "thread", "active", None, Some(root)),
    ];
    board_with("alpha", &threads)
}

pub const ALPHA_LEAD: &str = "sess-alpha-lead";
pub const UI: &str = "sess-ui";
pub const HELPER: &str = "sess-helper";

/// Boards `alpha` (a lead, `ui`, `api` and `web`) and `portfolio` (the Overlord and a helper),
/// each lead working in the checkout of its project under `base`.
pub fn seed_standard(coord_home: &Path, base: &Path) {
    let alpha_root = make_checkout(base, "alpha");
    let main_root = make_checkout(base, "portfolio");
    let alpha: [ThreadSpec<'_>; 4] = [
        (
            "lead",
            "lead",
            "active",
            Some(ALPHA_LEAD),
            Some(&alpha_root),
        ),
        ("ui", "thread", "active", Some(UI), None),
        ("api", "thread", "active", Some("sess-api"), None),
        ("web", "thread", "active", Some("sess-web"), None),
    ];
    let portfolio: [ThreadSpec<'_>; 2] = [
        ("lead", "lead", "active", Some(MAIN), Some(&main_root)),
        ("helper", "thread", "active", Some(HELPER), None),
    ];
    argus_testkit::write_board(coord_home, "alpha", &board_with("alpha", &alpha));
    argus_testkit::write_board(
        coord_home,
        "portfolio",
        &board_with("portfolio", &portfolio),
    );
}
