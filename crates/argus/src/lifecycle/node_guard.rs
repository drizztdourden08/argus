//! `argus serve` never opens a dashboard home that the Node dashboard is using: both would write
//! the same database and the same `server.pid`. The Node server records itself in that pid file
//! like argus does, so the guard reads it: a live process there that does not answer as argus is
//! the Node dashboard (or at least not ours), and a port that answers health without the argus
//! `impl` field is the Node dashboard for certain.

use argus_core::Paths;
use argus_core::config::peek_config;
use argus_server::instance::{pid_alive, probe, read_pid_file};

/// The refusal for `argus serve`, or `None` when the home is free of a Node server.
pub async fn refusal(paths: &Paths) -> Option<String> {
    let record = read_pid_file(&paths.pid_file)?;
    let config = peek_config(paths, None);
    let token = config
        .as_ref()
        .filter(|c| c["requireToken"] == true)
        .and_then(|c| c["token"].as_str().map(str::to_owned));
    let health = match record.port {
        0 => None,
        port => probe(port, token.as_deref()).await,
    };
    let node = match &health {
        Some(h) if h["impl"] == "argus" => return None,
        Some(_) => true,
        None => pid_alive(record.pid),
    };
    node.then(|| {
        format!(
            "the Node dashboard seems to be using {} (pid {}, port {}) and `argus serve` would \
             write the same database. Give argus its own folder with --data-dir <folder>, or stop \
             the Node dashboard first. (If pid {} is not the dashboard, delete {}.)",
            paths.home.display(),
            record.pid,
            record.port,
            record.pid,
            paths.pid_file.display()
        )
    })
}
