//! `argus tray start`: the tray in the background, and with it the server, as Node's
//! `dashboard tray start` does. A tray that runs already is left alone and the server is still
//! made sure of, so the command always ends with both up.

use std::time::Duration;

use argus_core::Paths;
use argus_native::{Launch, TrayFiles, prepare_launch, process_alive, wait_for_tray};
use tokio::time::sleep;

use argus::detach;
use argus_core::launch;

use crate::lifecycle::live::{Up, up};
use crate::lifecycle::start::{self as server_start, Options};

const WAIT: Duration = Duration::from_secs(10);
const STEP: Duration = Duration::from_millis(150);
/// How long a fresh tray gets to bring the server up before this command starts it itself.
const FRESH_GRACE: Duration = Duration::from_secs(10);
/// The same for a tray that ran already: its server is up, or it was stopped on purpose.
const RUNNING_GRACE: Duration = Duration::from_millis(1500);
const LOOK: Duration = Duration::from_millis(200);

/// Waits for the server for `grace`, then starts it here when it did not come up, and prints the
/// address as `argus start` does.
async fn ensure_server(paths: &Paths, grace: Duration) -> Result<u8, String> {
    let mut waited = Duration::ZERO;
    loop {
        if let Some(live) = up(paths).await.filter(Up::is_argus) {
            println!(
                "dashboard running at {}/ (pid {})",
                live.at.base(),
                live.at.pid_text()
            );
            return Ok(0);
        }
        if waited >= grace {
            break;
        }
        sleep(LOOK).await;
        waited += LOOK;
    }
    let options = Options {
        port: None,
        wait: server_start::START_WAIT,
        quiet: false,
    };
    server_start::start(paths, &options).await
}

/// Starts the tray unless one runs. Starting it is how the owner reopens a dashboard they quit,
/// so `quit.flag` is cleared first.
///
/// # Errors
///
/// The flag could not be cleared, the tray could not be started or did not show up in time.
pub async fn start(paths: &Paths) -> Result<u8, String> {
    let files = TrayFiles::new(&paths.home);
    let ready = prepare_launch(&files, process_alive).map_err(|e| e.to_string())?;
    if let Launch::Already { pid } = ready {
        println!("tray already running (pid {pid})");
        return ensure_server(paths, RUNNING_GRACE).await;
    }
    detach::argus(&launch::tray_run(&paths.home))
        .map_err(|e| format!("the tray could not be started: {e}"))?;
    let pause = || std::thread::sleep(STEP);
    let Some(tray) = wait_for_tray(&files, process_alive, WAIT, pause) else {
        return Err("the tray did not start in time".to_owned());
    };
    println!("tray started (pid {})", tray.pid);
    ensure_server(paths, FRESH_GRACE).await
}
