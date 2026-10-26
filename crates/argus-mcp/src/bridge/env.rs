//! What the stdio bridge needs from the machine it runs on: where the server is, whether the
//! owner quit it from the tray, how to start it and where log lines go. The binary implements
//! [`BridgeEnv`] over `config.json`, the quit flag and the tray launcher.

use std::time::Duration;

/// Where the running server answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    /// `http://127.0.0.1:<port>`.
    pub base: String,
    /// The bearer token, only when `config.json` says `requireToken`.
    pub token: Option<String>,
}

impl Endpoint {
    /// The loopback endpoint of `port`. A loopback caller needs no credential, so the token
    /// is carried only when the config asks for it.
    #[must_use]
    pub fn loopback(port: u16, token: Option<&str>, require_token: bool) -> Self {
        Self {
            base: format!("http://127.0.0.1:{port}"),
            token: token
                .filter(|t| require_token && !t.is_empty())
                .map(str::to_owned),
        }
    }
}

/// The machine the bridge runs on.
pub trait BridgeEnv: Send + Sync + 'static {
    /// The server's endpoint from `config.json`, read on every call; none without a config.
    fn endpoint(&self) -> Option<Endpoint>;

    /// Whether the owner chose Quit in the tray, so nothing may restart the server.
    fn quit_flag_active(&self) -> bool;

    /// Start the server hidden and detached, without waiting for it. An error says why not.
    ///
    /// # Errors
    /// The text of whatever stopped the start.
    fn launch(&self) -> Result<(), String>;

    /// One log line, for stderr; stdout carries JSON-RPC and nothing else.
    fn log(&self, line: &str);
}

/// How the bridge behaves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeOptions {
    /// The default project (`--project`); an invalid key is ignored with a log line.
    pub project: Option<String>,
    /// Start the server as soon as the bridge runs.
    pub boot_launch: bool,
    /// How long a call waits for a start (8 s).
    pub start_wait: Duration,
    /// How long a forwarded call may take (120 s).
    pub call_timeout: Duration,
}

impl Default for BridgeOptions {
    fn default() -> Self {
        Self {
            project: None,
            boot_launch: true,
            start_wait: Duration::from_secs(8),
            call_timeout: Duration::from_secs(120),
        }
    }
}
