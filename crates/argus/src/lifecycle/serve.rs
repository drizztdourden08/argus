//! `argus serve`: the server in the foreground, until Ctrl-C or `POST /api/admin/shutdown`.

use std::path::PathBuf;
use std::sync::Arc;

use argus_core::Paths;
use argus_core::config::env_port;
use argus_server::{ServeOptions, serve};

/// The port of `argus serve` while the Node server keeps 47615.
const DEFAULT_PORT: u16 = 47616;
/// The port the Node server keeps until the switch-over.
const NODE_PORT: u16 = 47615;

/// The folders of the dashboard: `COORD_HOME` from the environment, and the given home or the
/// default one.
pub fn paths_of(data_dir: Option<PathBuf>) -> Paths {
    let env = Paths::from_env();
    let home = data_dir.map(|d| std::path::absolute(&d).unwrap_or(d));
    Paths::new(env.coord_home, home.or(Some(env.home)))
}

/// The port to listen on: the flag, else `DASHBOARD_PORT`, else the default. While the Node
/// server keeps 47615, a `DASHBOARD_PORT` of 47615 is ignored for the default; only an explicit
/// `--port 47615` binds it.
///
/// # Errors
///
/// `DASHBOARD_PORT` is set to something that is not a port.
pub fn port_of(flag: Option<u16>) -> Result<u16, String> {
    let env = env_port().map_err(|e| e.to_string())?;
    Ok(pick_port(flag, env))
}

fn pick_port(flag: Option<u16>, env: Option<u16>) -> u16 {
    flag.or(env.filter(|p| *p != NODE_PORT))
        .unwrap_or(DEFAULT_PORT)
}

/// Runs the server until it is told to stop.
///
/// # Errors
///
/// The port is refused or the server does not start.
pub async fn run(port: Option<u16>, data_dir: Option<PathBuf>, shadow: bool) -> Result<u8, String> {
    let mut opts = ServeOptions::new(paths_of(data_dir));
    opts.port = Some(port_of(port)?);
    opts.shadow = shadow;
    opts.notify.system = true;
    let server = serve(opts).await.map_err(|e| e.to_string())?;
    println!(
        "dashboard listening on {}/ (pid {}){}",
        server.url,
        std::process::id(),
        if shadow { " (shadow)" } else { "" }
    );
    let svc = Arc::clone(&server.svc);
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            svc.stop.cancel();
        }
    });
    server.wait().await;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::pick_port;

    #[test]
    fn the_node_port_is_only_bound_when_asked_for_by_flag() {
        assert_eq!(pick_port(None, None), 47616);
        assert_eq!(pick_port(None, Some(47615)), 47616);
        assert_eq!(pick_port(Some(47615), Some(47615)), 47615);
        assert_eq!(pick_port(None, Some(0)), 0);
        assert_eq!(pick_port(Some(9), Some(47615)), 9);
    }
}
