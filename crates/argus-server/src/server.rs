//! Boot and shutdown: config, log, database, bus links, sync, routes, the listener on
//! `127.0.0.1`, the pid file and the hourly prune. Shutdown ends the streams, lets requests
//! finish for a second, stops sync and removes the pid file.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, AtomicUsize};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use argus_core::clock::now_iso;
use argus_core::config::{ConfigError, HOST, load_config};
use argus_core::log::{LOG_MAX_BYTES, Logger};
use argus_core::redact::Redactor;
use argus_core::{Bus, Config, Paths};
use argus_native::TrayFiles;
use argus_store::{Db, PruneOptions, StoreError};
use argus_sync::{RunsView, ScopeQueue};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::gate::{App, dispatch};
use crate::instance::{read_pid_file, running_instance, write_pid_file};
use crate::security::LoginCodes;
use crate::services::{activity, compose};
use crate::sse::Fanout;
use crate::web::WebAssets;
use crate::{ProcsOptions, Router, Services, routes};

const PRUNE_EVERY: Duration = Duration::from_secs(3600);
const FORCE_AFTER: Duration = Duration::from_secs(1);

/// How to start.
#[derive(Clone)]
pub struct ServeOptions {
    /// The data folders.
    pub paths: Paths,
    /// A port that overrides the config's without being saved (0 picks a free one).
    pub port: Option<u16>,
    /// The SSE heartbeat.
    pub heartbeat: Duration,
    /// A folder to serve the web build from instead of the embedded one.
    pub web_root: Option<PathBuf>,
    /// Run the sync service.
    pub sync: bool,
    /// Shadow mode: no board writes.
    pub shadow: bool,
    /// Also print the log to stderr.
    pub echo_log: bool,
    /// Run counts for the cards.
    pub runs: Option<Arc<dyn RunsView>>,
    /// The process monitor.
    pub procs: ProcsOptions,
}

impl ServeOptions {
    /// Defaults for a home: sync on, 15 s heartbeat, the embedded build.
    #[must_use]
    pub fn new(paths: Paths) -> Self {
        ServeOptions {
            paths,
            port: None,
            heartbeat: Duration::from_secs(15),
            web_root: None,
            sync: true,
            shadow: false,
            echo_log: false,
            runs: None,
            procs: ProcsOptions::default(),
        }
    }
}

/// Why the server did not start.
#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    /// The config was refused.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// The database could not open.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// Another server of this home answers.
    #[error("the dashboard is already running (pid {pid}, port {port})")]
    AlreadyRunning {
        /// Its pid.
        pid: u32,
        /// Its port.
        port: u16,
    },
    /// The port is taken.
    #[error("port {0} is in use by another program (or a dashboard of another DASHBOARD_HOME)")]
    PortInUse(u16),
    /// A file could not be written.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// A running server.
pub struct ServerHandle {
    /// `http://127.0.0.1:<port>`.
    pub url: String,
    /// The port.
    pub port: u16,
    /// The services.
    pub svc: Arc<Services>,
    task: JoinHandle<()>,
}

impl ServerHandle {
    /// Stops the server and waits for its shutdown.
    pub async fn stop(self) {
        self.svc.stop.cancel();
        let _ = self.task.await;
    }

    /// Waits until the server stops (a shutdown request, or [`Self::stop`]).
    pub async fn wait(self) {
        let _ = self.task.await;
    }
}

fn random_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    getrandom::fill(&mut b).unwrap_or_default();
    argus_core::config::hex(&b)
}

async fn checked_config(paths: &Paths, port: Option<u16>) -> Result<Config, ServeError> {
    std::fs::create_dir_all(&paths.home)?;
    let config = load_config(paths, port)?;
    let token = config.require_token.then_some(config.token.as_str());
    match running_instance(&paths.pid_file, token).await {
        Some(live) => Err(ServeError::AlreadyRunning {
            pid: live.pid,
            port: live.port,
        }),
        None => Ok(config),
    }
}

struct Core {
    paths: Paths,
    config: Arc<RwLock<Config>>,
    log: Arc<Logger>,
    bus: Bus,
    db: Arc<Db>,
}

async fn open_core(opts: &ServeOptions) -> Result<Core, ServeError> {
    let paths = opts.paths.clone();
    let config = checked_config(&paths, opts.port).await?;
    let log = Arc::new(
        Logger::new(
            paths.logs.join("server.log"),
            Redactor::new(std::slice::from_ref(&config.token)),
            LOG_MAX_BYTES,
        )
        .echo(opts.echo_log),
    );
    let bus = Bus::new();
    let db = Arc::new(Db::open(&paths.db, bus.clone())?);
    Ok(Core {
        paths,
        config: Arc::new(RwLock::new(config)),
        log,
        bus,
        db,
    })
}

fn assemble(core: Core, opts: &ServeOptions) -> Result<Services, ServeError> {
    let Core {
        paths,
        config,
        log,
        bus,
        db,
    } = core;
    let instance = random_hex(8);
    db.kv_set("instance", &instance)?;
    let fanout = Arc::new(Fanout::default());
    let (f1, f2) = (Arc::clone(&fanout), Arc::clone(&fanout));
    let links = vec![
        bus.on_docs(move |e| f1.on_docs(e)),
        bus.on_reset(move |e| f2.on_reset(e)),
    ];
    Ok(Services {
        config,
        env_port: opts.port,
        tray: TrayFiles::new(&paths.home),
        paths,
        activity: activity::build(&db, &log),
        db,
        bus,
        log,
        sync: None,
        queue: ScopeQueue::new(),
        runs: None,
        launcher_stale: None,
        wake: None,
        agent_watcher: None,
        agent_reaper: None,
        agent_reconciler: None,
        procs: None,
        worktrees: None,
        notify: None,
        notifier: None,
        instance,
        started_at: now_iso(),
        shadow: opts.shadow,
        heartbeat: opts.heartbeat,
        web: WebAssets::new(opts.web_root.clone()),
        fanout,
        streams: Arc::new(AtomicUsize::new(0)),
        codes: LoginCodes::default(),
        stop: CancellationToken::new(),
        port: AtomicU16::new(0),
        links: Mutex::new(links),
    })
}

async fn build(opts: &ServeOptions) -> Result<Services, ServeError> {
    let mut svc = assemble(open_core(opts).await?, opts)?;
    compose::build_all(&mut svc, opts);
    Ok(svc)
}

async fn bind(port: u16) -> Result<TcpListener, ServeError> {
    TcpListener::bind((HOST, port)).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::AddrInUse {
            ServeError::PortInUse(port)
        } else {
            e.into()
        }
    })
}

fn prune(svc: &Services) {
    if let Err(e) = svc.db.docs().prune(PruneOptions::default()) {
        svc.log
            .warn("prune failed", json!({ "err": e.to_string() }));
    }
}

fn spawn_prune(svc: &Arc<Services>) {
    let svc = Arc::clone(svc);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                () = tokio::time::sleep(PRUNE_EVERY) => prune(&svc),
                () = svc.stop.cancelled() => return,
            }
        }
    });
}

fn close(svc: &Services) {
    svc.log.info("stopping", json!({}));
    compose::stop_all(svc);
    svc.db.checkpoint();
    if read_pid_file(&svc.paths.pid_file).is_some_and(|r| r.instance == svc.instance) {
        let _ = std::fs::remove_file(&svc.paths.pid_file);
    }
}

async fn run(listener: TcpListener, svc: Arc<Services>) {
    let mut router = Router::new();
    routes::register_all(&mut router);
    let app = Arc::new(App {
        router,
        svc: Arc::clone(&svc),
    });
    let make = axum::Router::new()
        .fallback(dispatch)
        .with_state(app)
        .into_make_service_with_connect_info::<SocketAddr>();
    let signal = {
        let svc = Arc::clone(&svc);
        async move {
            svc.stop.cancelled().await;
            svc.fanout.close_all();
        }
    };
    let served = axum::serve(listener, make)
        .with_graceful_shutdown(signal)
        .into_future();
    let stop = svc.stop.clone();
    tokio::select! {
        _ = served => {}
        () = async { stop.cancelled().await; tokio::time::sleep(FORCE_AFTER).await } => {}
    }
    close(&svc);
}

/// Starts the server.
///
/// # Errors
/// A refused config, an unopenable database, a running server of this home, or a taken port.
pub async fn serve(opts: ServeOptions) -> Result<ServerHandle, ServeError> {
    let svc = build(&opts).await?;
    let listener = match bind(svc.config().port).await {
        Ok(l) => l,
        Err(e) => {
            svc.db.checkpoint();
            return Err(e);
        }
    };
    let port = listener.local_addr()?.port();
    svc.port.store(port, std::sync::atomic::Ordering::SeqCst);
    write_pid_file(&svc.paths.pid_file, port, &svc.started_at, &svc.instance)?;
    let svc = Arc::new(svc);
    compose::start_all(&svc).await;
    prune(&svc);
    spawn_prune(&svc);
    svc.log.info("listening", json!({ "port": port, "pid": std::process::id(), "sync": svc.sync.is_some(), "shadow": svc.shadow }));
    let task = tokio::spawn(run(listener, Arc::clone(&svc)));
    Ok(ServerHandle {
        url: format!("http://{HOST}:{port}"),
        port,
        svc,
        task,
    })
}
