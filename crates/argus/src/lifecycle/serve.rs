//! `argus serve`: the server in the foreground, until Ctrl-C or `POST /api/admin/shutdown`.

use std::path::PathBuf;
use std::sync::Arc;

use argus_core::Paths;
use argus_core::config::env_port;
use argus_core::names::HTTP_PORT;
use argus_server::{ServeOptions, serve};

/// The folders of the dashboard: `COORD_HOME` from the environment, and the given home or the
/// default one.
pub fn paths_of(data_dir: Option<PathBuf>) -> Paths {
    let env = Paths::from_env();
    let home = data_dir.map(|d| std::path::absolute(&d).unwrap_or(d));
    Paths::new(env.coord_home, home.or(Some(env.home)))
}

/// The port override of the server: the flag, else `DASHBOARD_PORT`, else none, which leaves the
/// port to the `port` of the home's `config.json` (47615 by default), as Node does.
///
/// # Errors
///
/// `DASHBOARD_PORT` is set to something that is not a port.
pub fn port_of(flag: Option<u16>) -> Result<Option<u16>, String> {
    let env = env_port().map_err(|e| e.to_string())?;
    Ok(pick_port(flag, env))
}

fn pick_port(flag: Option<u16>, env: Option<u16>) -> Option<u16> {
    flag.or(env)
}

/// Runs the server until it is told to stop.
///
/// # Errors
///
/// The port is refused or the server does not start.
pub async fn run(port: Option<u16>, data_dir: Option<PathBuf>, shadow: bool) -> Result<u8, String> {
    let paths = paths_of(data_dir);
    if let Some(why) = super::node_guard::refusal(&paths).await {
        return Err(why);
    }
    let mut opts = ServeOptions::new(paths);
    opts.port = port_of(port)?;
    opts.shadow = shadow;
    opts.shadow_seed = shadow.then(|| Paths::from_env().home);
    opts.notify.system = true;
    opts.localhost_door_port = Some(HTTP_PORT);
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

// test-tag: critical
#[cfg(test)]
mod tests {
    //! K1: the order of the port flag, the environment and the config.
    use super::pick_port;

    #[test]
    fn the_flag_beats_the_environment_and_neither_leaves_the_port_to_the_config() {
        assert_eq!(pick_port(None, None), None);
        assert_eq!(pick_port(None, Some(47615)), Some(47615));
        assert_eq!(pick_port(Some(47615), Some(9)), Some(47615));
        assert_eq!(pick_port(None, Some(0)), Some(0));
        assert_eq!(pick_port(Some(9), Some(47615)), Some(9));
    }
}
