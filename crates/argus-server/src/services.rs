//! The services every route reaches: config, paths, database, bus, log, sync, the streams and
//! the instance facts. A service that is off is `None` and its routes answer 503.

use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering};
use std::sync::{Arc, PoisonError, RwLock};
use std::time::Duration;

use argus_core::log::Logger;
use argus_core::{Bus, Config, Paths, Subscription};
use argus_store::Db;
use argus_sync::{ScopeQueue, SyncService};
use tokio_util::sync::CancellationToken;

use crate::security::LoginCodes;
use crate::sse::Fanout;
use crate::web::WebAssets;

/// The composed services.
pub struct Services {
    /// The live config.
    pub config: Arc<RwLock<Config>>,
    /// `DASHBOARD_PORT`, which a save must not write.
    pub env_port: Option<u16>,
    /// The data folders.
    pub paths: Paths,
    /// The database.
    pub db: Arc<Db>,
    /// The event bus.
    pub bus: Bus,
    /// The server log.
    pub log: Arc<Logger>,
    /// The sync service, when on.
    pub sync: Option<Arc<SyncService>>,
    /// The per-scope mutation queues.
    pub queue: ScopeQueue,
    /// This run's random id.
    pub instance: String,
    /// When it started.
    pub started_at: String,
    /// Shadow mode: no outward effect, owner posts answer `read_only`.
    pub shadow: bool,
    /// The SSE heartbeat.
    pub heartbeat: Duration,
    /// The web build.
    pub web: WebAssets,
    /// The doc stream hub.
    pub fanout: Arc<Fanout>,
    /// Open SSE streams.
    pub streams: Arc<AtomicUsize>,
    /// One-time login codes.
    pub codes: LoginCodes,
    /// Rings when the server should stop.
    pub stop: CancellationToken,
    pub(crate) port: AtomicU16,
    pub(crate) _bus_links: Vec<Subscription>,
}

impl Services {
    /// The port the listener got.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port.load(Ordering::SeqCst)
    }

    /// A copy of the config.
    #[must_use]
    pub fn config(&self) -> Config {
        self.config
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The current token.
    #[must_use]
    pub fn token(&self) -> String {
        self.config
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .token
            .clone()
    }

    /// Whether the token is required.
    #[must_use]
    pub fn require_token(&self) -> bool {
        self.config
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .require_token
    }

    /// The server version.
    #[must_use]
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
