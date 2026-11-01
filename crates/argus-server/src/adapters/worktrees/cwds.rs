//! The folder each process runs in, for the check that no process uses a worktree being removed.
//!
//! On Linux each folder is a link in /proc. Elsewhere one process-table refresh reads the folders of
//! every wanted pid at once (on Windows from the process's own parameters). A process that cannot
//! be read (elevated, a system process, gone since the sample) is left out, without a word.

use std::collections::HashMap;

#[cfg(target_os = "linux")]
pub(super) fn cwds_of(pids: &[u32]) -> HashMap<u32, String> {
    pids.iter()
        .filter_map(|pid| {
            let link = std::fs::read_link(format!("/proc/{pid}/cwd")).ok()?;
            Some((*pid, link.to_string_lossy().into_owned()))
        })
        .collect()
}

#[cfg(not(target_os = "linux"))]
pub(super) fn cwds_of(pids: &[u32]) -> HashMap<u32, String> {
    use argus_procs::{Sampler, SystemSampler};
    SystemSampler.cwds(pids)
}

// test-tag: e2e
#[cfg(test)]
mod tests {
    use super::cwds_of;
    use std::process::{Child, Command, Stdio};

    /// A child that stays alive until its input closes or it is killed.
    fn idle_child(dir: &std::path::Path) -> Child {
        #[cfg(windows)]
        let mut command = {
            let mut c = Command::new("cmd");
            c.args(["/c", "pause"]);
            c
        };
        #[cfg(not(windows))]
        let mut command = {
            let mut c = Command::new("sleep");
            c.arg("60");
            c
        };
        command
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the child starts")
    }

    #[test]
    #[ignore = "e2e"]
    fn a_real_child_is_found_in_the_folder_it_was_started_in_and_a_missing_pid_is_left_out() {
        let dir = tempfile::tempdir().unwrap();
        let name = dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase();
        let mut child = idle_child(dir.path());
        let pid = child.id();
        std::thread::sleep(std::time::Duration::from_millis(500));
        let found = cwds_of(&[pid, u32::MAX - 1]);
        child.kill().unwrap();
        child.wait().unwrap();
        let cwd = found.get(&pid).expect("the child's folder is read");
        assert!(cwd.to_lowercase().contains(&name), "{cwd}");
        assert!(!found.contains_key(&(u32::MAX - 1)));
    }
}
