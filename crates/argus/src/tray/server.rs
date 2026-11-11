//! The server as the tray sees it: is it up, how busy, start it, stop it, where its page is.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use argus_core::config::load_config;
use argus_core::page_url::page;
use argus_core::{Config, Paths};
use argus_native::{AccountLine, AccountLines, TrayState};
use argus_server::instance::{PidRecord, call, probe, read_pid_file};
use serde_json::Value;
use tokio::time::sleep;

use argus::detach;
use argus_core::launch;

const START_WAIT: Duration = Duration::from_secs(15);
const STOP_WAIT: Duration = Duration::from_secs(10);
const STEP: Duration = Duration::from_millis(250);

/// The server of one dashboard home.
pub struct Server {
    paths: Paths,
    /// True from the moment a start is expected until it has ended, so the icon says "starting"
    /// and never "stopped" while the server is on its way up.
    starting: AtomicBool,
}

impl Server {
    /// The server that keeps its files in `paths`.
    pub fn new(paths: Paths) -> Self {
        Server {
            paths,
            starting: AtomicBool::new(false),
        }
    }

    /// Notes that a start is about to run, before the task that does it has been scheduled, so
    /// the first probe of the icon already says "starting".
    pub fn expect_start(&self) {
        self.starting.store(true, Ordering::SeqCst);
    }

    fn config(&self) -> Option<Config> {
        load_config(&self.paths, None).ok()
    }

    /// The pid file's record and the health answer of the server it names, when that server is
    /// the one that wrote it.
    async fn live(&self) -> Option<(PidRecord, Value)> {
        let record = read_pid_file(&self.paths.pid_file)?;
        let config = self.config()?;
        let token = config.require_token.then_some(config.token.as_str());
        let health = probe(record.port, token).await?;
        (health["instance"] == record.instance.as_str()).then_some((record, health))
    }

    /// What the icon shows: the server up with its runs, on its way up, or down.
    pub async fn state(&self) -> TrayState {
        let shadow = launch::is_shadow(&self.paths.home);
        self.state_of_server().await.shadowed(shadow)
    }

    async fn state_of_server(&self) -> TrayState {
        match self.live().await {
            Some((_, health)) => {
                let runs = health["runs"]["running"].as_u64().unwrap_or(0);
                TrayState::running(u32::try_from(runs).unwrap_or(u32::MAX))
            }
            None if self.starting.load(Ordering::SeqCst) => TrayState::starting(),
            None => TrayState::stopped(),
        }
    }

    /// Whether the server is on its way up and has not answered yet.
    pub fn is_starting(&self) -> bool {
        self.starting.load(Ordering::SeqCst)
    }

    /// Starts `argus serve` apart from the tray, unless a server answers already, and waits for
    /// it to answer.
    ///
    /// # Errors
    ///
    /// The server could not be started or did not answer in time.
    pub async fn start(&self) -> Result<(), String> {
        self.expect_start();
        let done = self.start_now().await;
        self.starting.store(false, Ordering::SeqCst);
        done
    }

    async fn start_now(&self) -> Result<(), String> {
        if self.live().await.is_some() {
            return Ok(());
        }
        detach::argus(&launch::serve(&self.paths.home, None))
            .map_err(|e| format!("the server could not be started: {e}"))?;
        self.wait(true, START_WAIT)
            .await
            .then_some(())
            .ok_or_else(|| "the server did not answer in time".to_owned())
    }

    /// Asks the server to stop and waits for it to; one that does not is ended.
    pub async fn stop(&self) {
        let Some((record, _)) = self.live().await else {
            return;
        };
        let token = self.config().map(|c| c.token);
        let path = "/api/admin/shutdown";
        let _asked = call(record.port, "POST", path, token.as_deref()).await;
        if !self.wait(false, STOP_WAIT).await {
            detach::kill(record.pid);
        }
    }

    async fn wait(&self, up: bool, limit: Duration) -> bool {
        let mut waited = Duration::ZERO;
        loop {
            if self.live().await.is_some() == up {
                return true;
            }
            if waited >= limit {
                return false;
            }
            sleep(STEP).await;
            waited += STEP;
        }
    }

    /// The accounts section of the menu: read from the server's `GET /api/accounts/tray`. Empty
    /// when the server is down, has no accounts service, or answers something else.
    pub async fn accounts(&self) -> AccountLines {
        let Some((record, _)) = self.live().await else {
            return AccountLines::default();
        };
        let token = self.config().map(|c| c.token);
        let answer = call(record.port, "GET", "/api/accounts/tray", token.as_deref()).await;
        answer
            .filter(|(status, _)| *status == 200)
            .map(|(_, body)| account_lines(&body))
            .unwrap_or_default()
    }

    /// Asks the server to cancel the switch countdown.
    ///
    /// # Errors
    ///
    /// The server does not answer or refuses.
    pub async fn cancel_switch(&self) -> Result<(), String> {
        let (record, _) = self
            .live()
            .await
            .ok_or("the server did not answer its health check")?;
        let token = self.config().map(|c| c.token);
        let path = "/api/accounts/switch/cancel";
        match call(record.port, "POST", path, token.as_deref()).await {
            Some((status, _)) if status < 300 => Ok(()),
            Some((status, _)) => Err(format!("the server answered {status}")),
            None => Err("the cancel call got no answer".to_owned()),
        }
    }

    /// The address of a page of the running server: under the localhost names when the server
    /// says its localhost door is bound (`http://argus.localhost/`), else on `127.0.0.1`. With
    /// the token hardening on, it carries a one-time code that signs the page in.
    ///
    /// # Errors
    ///
    /// Why there is no address: the server does not answer, the config cannot be read, or the
    /// sign-in code was refused.
    pub async fn page(&self, next: &str) -> Result<String, String> {
        let (record, health) = self
            .live()
            .await
            .ok_or("the server did not answer its health check")?;
        let base = page(&health, record.port, None).origin;
        let config = load_config(&self.paths, None).map_err(|e| format!("config.json: {e}"))?;
        if !config.require_token {
            return Ok(format!("{base}{next}"));
        }
        let code = sign_in_code(record.port, &config.token).await?;
        let next = next.replace('#', "%23");
        Ok(format!("{base}/auth?code={code}&next={next}"))
    }
}

/// The menu lines of a `GET /api/accounts/tray` body; nothing readable gives no lines.
fn account_lines(body: &str) -> AccountLines {
    let Ok(reply) = serde_json::from_str::<Value>(body) else {
        return AccountLines::default();
    };
    let lines = reply["accounts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|account| {
            let key = account["key"].as_str()?;
            Some(AccountLine {
                key: key.to_owned(),
                label: account["label"].as_str().unwrap_or(key).to_owned(),
                weekly: account["weekly"].as_f64().map(percent),
                active: account["active"].as_bool().unwrap_or(false),
            })
        })
        .collect();
    AccountLines {
        lines,
        switching: reply["switching"].as_bool().unwrap_or(false),
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a percent is clamped to 0..=999 first"
)]
fn percent(value: f64) -> u32 {
    value.round().clamp(0.0, 999.0) as u32
}

async fn sign_in_code(port: u16, token: &str) -> Result<String, String> {
    let (status, body) = call(port, "POST", "/api/auth/code", Some(token))
        .await
        .ok_or("the sign-in code call got no answer")?;
    serde_json::from_str::<Value>(&body)
        .ok()
        .and_then(|reply| reply["code"].as_str().map(str::to_owned))
        .ok_or_else(|| format!("the sign-in code call answered {status} without a code"))
}

// test-tag: standard
#[cfg(test)]
mod tests {
    //! S2: the tray reads the server state from a home and an in-process server.
    use argus_core::Paths;
    use argus_native::TrayState;
    use argus_testkit::{TestOptions, TestServer};

    use super::Server;

    fn idle_server() -> (tempfile::TempDir, Server) {
        let dir = tempfile::tempdir().expect("a temp home");
        let paths = Paths::new(dir.path().join("coord"), Some(dir.path().join("dashboard")));
        (dir, Server::new(paths))
    }

    #[tokio::test]
    async fn a_server_that_is_down_reads_stopped_until_a_start_is_expected() {
        let (_dir, server) = idle_server();
        assert_eq!(server.state().await, TrayState::stopped());
        server.expect_start();
        assert!(server.is_starting());
        assert_eq!(server.state().await, TrayState::starting());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_server_that_answers_reads_running_at_once_and_ends_the_starting() {
        let test = TestServer::start(TestOptions::default()).await;
        let paths = Paths::new(&test.coord_home, Some(test.dashboard_home.clone()));
        let server = Server::new(paths);
        server.expect_start();
        assert_eq!(server.state().await, TrayState::running(0));
        server
            .start()
            .await
            .expect("a running server needs no start");
        assert!(!server.is_starting());
        test.stop(false).await;
    }

    #[test]
    fn the_tray_answer_becomes_menu_lines() {
        let body = r#"{"accounts":[{"key":"a","label":"Alpha","weekly":41.6,"active":true},
            {"key":"b","weekly":null,"active":false},{"label":"no key"}],"switching":true}"#;
        let lines = super::account_lines(body);
        assert!(lines.switching);
        assert_eq!(lines.lines.len(), 2);
        assert_eq!(lines.lines[0].weekly, Some(42));
        assert!(lines.lines[0].active);
        assert_eq!(lines.lines[1].label, "b");
        assert_eq!(lines.lines[1].weekly, None);
        assert_eq!(
            super::account_lines("nonsense"),
            argus_native::AccountLines::default()
        );
    }
}
