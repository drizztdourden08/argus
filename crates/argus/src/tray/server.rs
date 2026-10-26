//! The server as the tray sees it: is it up, how busy, start it, stop it, where its page is.

use std::ffi::OsStr;
use std::time::Duration;

use argus_core::config::load_config;
use argus_core::{Config, Paths};
use argus_native::TrayState;
use argus_server::instance::{PidRecord, call, probe, read_pid_file};
use serde_json::Value;
use tokio::time::sleep;

use super::detach;

const START_WAIT: Duration = Duration::from_secs(15);
const STOP_WAIT: Duration = Duration::from_secs(10);
const STEP: Duration = Duration::from_millis(250);

/// The server of one dashboard home.
pub struct Server {
    paths: Paths,
}

impl Server {
    /// The server that keeps its files in `paths`.
    pub fn new(paths: Paths) -> Self {
        Server { paths }
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

    /// What the icon shows: the server down, or up with its runs.
    pub async fn state(&self) -> TrayState {
        match self.live().await {
            Some((_, health)) => {
                let runs = health["runs"]["running"].as_u64().unwrap_or(0);
                TrayState::running(u32::try_from(runs).unwrap_or(u32::MAX))
            }
            None => TrayState::stopped(),
        }
    }

    /// Starts `argus serve` apart from the tray, unless a server answers already, and waits for
    /// it to answer.
    ///
    /// # Errors
    ///
    /// The server could not be started or did not answer in time.
    pub async fn start(&self) -> Result<(), String> {
        if self.live().await.is_some() {
            return Ok(());
        }
        let home = self.paths.home.as_os_str();
        detach::argus(&[OsStr::new("serve"), OsStr::new("--data-dir"), home])
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

    /// The address of a page of the running server. With the token hardening on, it carries a
    /// one-time code that signs the page in.
    pub async fn page(&self, next: &str) -> Option<String> {
        let (record, _) = self.live().await?;
        let base = format!("http://127.0.0.1:{}", record.port);
        let config = self.config()?;
        if !config.require_token {
            return Some(format!("{base}{next}"));
        }
        let reply = call(record.port, "POST", "/api/auth/code", Some(&config.token)).await?;
        let code = serde_json::from_str::<Value>(&reply.1).ok()?["code"]
            .as_str()?
            .to_owned();
        let next = next.replace('#', "%23");
        Some(format!("{base}/auth?code={code}&next={next}"))
    }
}
