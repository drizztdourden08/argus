//! `argus stop`: asks the server to stop through its own route and waits for it and its process
//! to be gone. A server that does not stop in time is ended by the pid it recorded, which was
//! confirmed as the dashboard a moment before. Nothing is ever ended by image name.

use std::fs;
use std::time::Duration;

use argus_core::Paths;
use argus_server::instance::pid_alive;
use serde_json::json;
use tokio::time::sleep;

use super::live::{api, up};
use super::spawn::kill_tree;

const STOP_WAIT: Duration = Duration::from_secs(8);
const STEP: Duration = Duration::from_millis(150);
const AFTER_KILL: Duration = Duration::from_millis(500);

/// Whether the server still answers or its process still runs.
async fn lingering(paths: &Paths, pid: Option<u32>) -> bool {
    up(paths).await.is_some() || pid.is_some_and(pid_alive)
}

async fn wait_gone(paths: &Paths, pid: Option<u32>) -> bool {
    let mut waited = Duration::ZERO;
    while lingering(paths, pid).await {
        if waited >= STOP_WAIT {
            return false;
        }
        sleep(STEP).await;
        waited += STEP;
    }
    true
}

/// A pid file of a process that is gone is removed.
fn tidy(paths: &Paths) {
    let record = argus_server::instance::read_pid_file(&paths.pid_file);
    if record.is_some_and(|r| !pid_alive(r.pid)) {
        let _removed = fs::remove_file(&paths.pid_file);
    }
}

/// Stops the server of this home.
///
/// # Errors
///
/// None: a server that cannot be stopped is exit 1.
pub async fn stop(paths: &Paths) -> Result<u8, String> {
    let Some(running) = up(paths).await else {
        tidy(paths);
        println!("dashboard is not running");
        return Ok(0);
    };
    let pid = running.at.record.as_ref().map(|r| r.pid);
    if let Err(why) = api(&running.at, "POST", "/api/admin/shutdown", &json!({})).await {
        eprintln!("dashboard: shutdown request failed ({why}); stopping the process");
    }
    if wait_gone(paths, pid).await {
        println!("dashboard stopped");
        return Ok(0);
    }
    if let Some(pid) = pid {
        kill_tree(pid);
    }
    sleep(AFTER_KILL).await;
    if up(paths).await.is_some() {
        eprintln!("dashboard: the server did not stop");
        return Ok(1);
    }
    println!("dashboard stopped (killed)");
    Ok(0)
}
