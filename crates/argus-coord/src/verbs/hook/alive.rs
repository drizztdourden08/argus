//! Two questions about the machine, answered without unsafe code here: is a process running,
//! and did the owner quit the dashboard from the tray during this boot. On Windows both go
//! through the `argus-native` helpers the tray uses.

use std::path::Path;

/// Whether `pid` is a running process (a process that denies the query counts as running).
#[cfg(windows)]
pub fn pid_alive(pid: u32) -> bool {
    argus_native::process_alive(pid)
}

/// Whether `pid` is a running process.
#[cfg(not(windows))]
pub fn pid_alive(pid: u32) -> bool {
    Path::new("/proc").join(pid.to_string()).exists()
}

/// Whether `<home>/quit.flag` was written since this machine booted (with 5 s of slack).
#[cfg(windows)]
pub fn quit_this_boot(home: &Path) -> bool {
    use argus_native::{Boot, TrayFiles, quit_flag_active};
    quit_flag_active(&TrayFiles::new(home), Boot::current())
}

/// Whether `<home>/quit.flag` was written since this machine booted (with 5 s of slack). With
/// no uptime to read, any flag counts: the dashboard then stays down rather than start twice.
#[cfg(not(windows))]
pub fn quit_this_boot(home: &Path) -> bool {
    use std::time::{Duration, SystemTime};
    let Ok(written) = std::fs::metadata(home.join("quit.flag")).and_then(|m| m.modified()) else {
        return false;
    };
    let uptime = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|t| t.split_whitespace().next()?.parse::<f64>().ok());
    let Some(uptime) = uptime else {
        return true;
    };
    let boot = SystemTime::now() - Duration::from_secs_f64(uptime) - Duration::from_secs(5);
    written >= boot
}
