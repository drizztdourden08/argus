//! What a reap line says about the process beyond its name: the command it ran and who started it.

use super::pick::parent_alive;
use crate::types::{ProcRow, Sample};

const CMD_MAX: usize = 160;

/// The command line, trimmed to `CMD_MAX` characters with an ellipsis, in backticks.
fn cmd_text(cmd: &str) -> String {
    let cmd = cmd.trim().replace('`', "'");
    if cmd.chars().count() <= CMD_MAX {
        return cmd;
    }
    let head: String = cmd.chars().take(CMD_MAX).collect();
    format!("{head}...")
}

/// `parent <name> <ppid>` for a live parent in the sample, else `parent <ppid> gone`.
fn parent_text(root: &ProcRow, procs: &[ProcRow], sample: &Sample) -> String {
    let parent = procs.iter().find(|p| p.pid == root.ppid);
    match parent.filter(|p| parent_alive(sample, Some(p), root)) {
        Some(p) => format!("parent {} {}", p.name, p.pid),
        None => format!("parent {} gone", root.ppid),
    }
}

/// The text appended to a reap line: ` cmd `...`` (when there is one) and the parent.
pub(super) fn of(root: &ProcRow, procs: &[ProcRow], sample: &Sample) -> String {
    let parent = parent_text(root, procs, sample);
    if root.cmd.trim().is_empty() {
        return format!(", {parent}");
    }
    format!(", cmd `{}`, {parent}", cmd_text(&root.cmd))
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn row(pid: u32, ppid: u32, name: &str, cmd: &str) -> ProcRow {
        ProcRow {
            pid,
            ppid,
            name: name.into(),
            cmd: cmd.into(),
            path: None,
            mem_bytes: 0,
            start_ms: 0,
            cpu_ms: 0,
            responding: None,
            cwd: None,
        }
    }

    fn sample(procs: Vec<ProcRow>) -> Sample {
        Sample {
            procs,
            ..Sample::default()
        }
    }

    #[test]
    fn a_live_parent_is_named_with_its_pid() {
        let procs = vec![row(5, 9, "bash.exe", ""), row(7, 5, "grep.exe", "grep -n foo bar.txt")];
        let s = sample(procs.clone());
        assert_eq!(
            of(&procs[1], &procs, &s),
            ", cmd `grep -n foo bar.txt`, parent bash.exe 5"
        );
    }

    #[test]
    fn a_missing_parent_is_gone_and_an_empty_command_is_skipped() {
        let procs = vec![row(7, 5, "grep.exe", "")];
        let s = sample(procs.clone());
        assert_eq!(of(&procs[0], &procs, &s), ", parent 5 gone");
    }

    #[test]
    fn a_long_command_is_trimmed_with_an_ellipsis() {
        let cmd = "x".repeat(300);
        let text = cmd_text(&cmd);
        assert_eq!(text.chars().count(), CMD_MAX + 3);
        assert!(text.ends_with("..."));
        assert_eq!(cmd_text("a`b"), "a'b");
    }
}
