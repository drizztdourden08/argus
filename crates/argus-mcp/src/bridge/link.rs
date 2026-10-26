//! The bridge's link to the server: whether it is up, the start it triggered and what to say
//! when it is down. The bridge is spawned when Claude starts, so it brings the server up at
//! once, in the background, and a call that finds it down waits for that start (once per down
//! episode) instead of starting it again.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use argus_core::ids::is_key;
use tokio::time::{Instant, sleep};

use super::env::{BridgeEnv, BridgeOptions, Endpoint};
use crate::rpc::PROTOCOL_VERSION;

const HEALTH_TIMEOUT: Duration = Duration::from_millis(1500);
const POLL: Duration = Duration::from_millis(250);

fn line(message: &str) -> String {
    format!("[coord-dashboard mcp] {message}")
}

struct State {
    client: String,
    protocol: String,
    start_tried: bool,
    launched_at: Option<Instant>,
    closed_by_owner: bool,
    last_why: String,
}

/// The server as the bridge sees it.
pub struct Link<E: BridgeEnv> {
    pub(super) env: E,
    pub(super) opts: BridgeOptions,
    pub(super) http: reqwest::Client,
    state: Mutex<State>,
}

impl<E: BridgeEnv> Link<E> {
    pub(super) fn new(env: E, mut opts: BridgeOptions) -> Self {
        if let Some(p) = opts.project.take_if(|p| !is_key(p)) {
            env.log(&line(&format!(
                "ignoring --project \"{p}\": not a project key"
            )));
        }
        let http = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_default();
        let state = State {
            client: "client".to_owned(),
            protocol: PROTOCOL_VERSION.to_owned(),
            start_tried: false,
            launched_at: None,
            closed_by_owner: false,
            last_why: String::new(),
        };
        Self {
            env,
            opts,
            http,
            state: Mutex::new(state),
        }
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn set_session(&self, client: Option<String>, protocol: Option<String>) {
        let mut s = self.state();
        if let Some(c) = client {
            s.client = c;
        }
        if let Some(p) = protocol {
            s.protocol = p;
        }
    }

    pub(super) fn session(&self) -> (String, String) {
        let s = self.state();
        (s.client.clone(), s.protocol.clone())
    }

    pub(super) fn say(&self, message: &str) {
        self.env.log(&line(message));
    }

    pub(super) fn why(&self) -> String {
        self.state().last_why.clone()
    }

    pub(super) fn set_why(&self, why: String) {
        self.state().last_why = why;
    }

    pub(super) fn closed_by_owner(&self) -> bool {
        self.state().closed_by_owner
    }

    pub(super) fn auth(endpoint: &Endpoint) -> Option<String> {
        endpoint.token.as_ref().map(|t| format!("Bearer {t}"))
    }

    async fn healthy(&self) -> bool {
        let Some(endpoint) = self.env.endpoint() else {
            return false;
        };
        let mut request = self
            .http
            .get(format!("{}/api/health", endpoint.base))
            .timeout(HEALTH_TIMEOUT);
        if let Some(auth) = Self::auth(&endpoint) {
            request = request.header("authorization", auth);
        }
        request
            .send()
            .await
            .is_ok_and(|res| res.status().is_success())
    }

    fn claim_launch(&self, force: bool) -> bool {
        let mut s = self.state();
        if s.start_tried && !force {
            return false;
        }
        s.start_tried = true;
        s.launched_at = Some(Instant::now());
        true
    }

    /// Start the server in the background, unless a start is already pending.
    pub(super) fn launch(&self, force: bool) {
        if !self.claim_launch(force) {
            return;
        }
        self.say("starting the dashboard");
        if let Err(why) = self.env.launch() {
            self.set_why(why);
        }
    }

    fn seen_up(&self) {
        let mut s = self.state();
        s.start_tried = false;
        s.closed_by_owner = false;
    }

    fn wait_until(&self) -> Option<Instant> {
        self.state().launched_at.map(|t| t + self.opts.start_wait)
    }

    /// Whether the server answers, starting it once when it is down and waiting for it. The
    /// owner's Quit from the tray stops any start.
    pub(super) async fn ensure_up(&self) -> bool {
        if self.healthy().await {
            self.seen_up();
            return true;
        }
        let closed = self.env.quit_flag_active();
        self.state().closed_by_owner = closed;
        if closed {
            return false;
        }
        self.launch(false);
        let until = self.wait_until();
        while until.is_some_and(|u| Instant::now() < u) {
            sleep(POLL).await;
            if self.healthy().await {
                self.state().start_tried = false;
                return true;
            }
        }
        let mut s = self.state();
        if s.last_why.is_empty() {
            "it did not answer within 8 seconds".clone_into(&mut s.last_why);
        }
        false
    }
}
