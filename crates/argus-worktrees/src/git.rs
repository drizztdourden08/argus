//! The door into git. Reads carry the lock-free prefix and wait for a free slot; writes are the removal's own commands.

use crate::exec::{GitOutput, Spec, run_command};
use crate::guard::{assert_read_only, assert_write_ok};
use crate::sync::{locked, waited};
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Every read carries this: no index refresh, no daemon.
pub const GIT_PREFIX: [&str; 3] = ["--no-optional-locks", "-c", "core.fsmonitor=false"];
/// Git processes a scan may run at a time.
pub const MAX_PROCS: usize = 2;
/// How long a read may take.
pub const READ_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a write (a removal, a bundle) may take.
pub const WRITE_TIMEOUT: Duration = Duration::from_secs(120);
/// How long `git worktree remove` alone may take. A clean removal of a big pnpm `node_modules` is slow, not stuck.
pub const REMOVE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Runs `git` with already checked arguments. The seam tests replace.
pub trait GitExec: Send + Sync {
    fn exec(&self, cwd: &str, args: &[String], timeout: Duration) -> GitOutput;
}

/// The real git, found on the path.
pub struct SystemExec {
    program: String,
}

impl SystemExec {
    pub fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
        }
    }
}

impl Default for SystemExec {
    fn default() -> Self {
        Self::new("git")
    }
}

impl GitExec for SystemExec {
    fn exec(&self, cwd: &str, args: &[String], timeout: Duration) -> GitOutput {
        if !Path::new(cwd).is_dir() {
            let said = format!(
                "fatal: cannot change to '{cwd}': No such file or directory
"
            );
            return GitOutput::failure(128, &said);
        }
        let env = [
            ("GIT_OPTIONAL_LOCKS", "0"),
            ("GIT_TERMINAL_PROMPT", "0"),
            ("GIT_PAGER", "cat"),
        ];
        run_command(&Spec {
            program: &self.program,
            args,
            cwd,
            env: &env,
            timeout,
        })
    }
}

/// What the door counted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GitStats {
    pub running: usize,
    pub max_running: usize,
    pub started: usize,
    pub timed_out: usize,
}

struct Gate {
    limit: usize,
    stats: Mutex<GitStats>,
    freed: Condvar,
}

struct Slot<'a>(&'a Gate);

impl Gate {
    fn acquire(&self) -> Slot<'_> {
        let mut stats = locked(&self.stats);
        while stats.running >= self.limit {
            stats = waited(&self.freed, stats);
        }
        stats.running += 1;
        stats.started += 1;
        stats.max_running = stats.max_running.max(stats.running);
        Slot(self)
    }
}

impl Drop for Slot<'_> {
    fn drop(&mut self) {
        locked(&self.0.stats).running -= 1;
        self.0.freed.notify_one();
    }
}

/// The only way the crate talks to git.
pub struct Git {
    exec: Arc<dyn GitExec>,
    gate: Gate,
    read_timeout: Duration,
    write_timeout: Duration,
    remove_timeout: Duration,
}

impl Git {
    pub fn new(exec: Arc<dyn GitExec>) -> Self {
        Self::with_limits(exec, MAX_PROCS, READ_TIMEOUT, WRITE_TIMEOUT)
    }

    /// The real git with the standard limits.
    pub fn system() -> Self {
        Self::new(Arc::new(SystemExec::default()))
    }

    pub fn with_limits(
        exec: Arc<dyn GitExec>,
        max_procs: usize,
        read_timeout: Duration,
        write_timeout: Duration,
    ) -> Self {
        let gate = Gate {
            limit: max_procs.max(1),
            stats: Mutex::new(GitStats::default()),
            freed: Condvar::new(),
        };
        Self {
            exec,
            gate,
            read_timeout,
            write_timeout,
            remove_timeout: REMOVE_TIMEOUT,
        }
    }

    /// The same door with another limit for `git worktree remove` (`worktrees.removeTimeoutMs`).
    #[must_use]
    pub fn with_remove_timeout(mut self, remove_timeout: Duration) -> Self {
        self.remove_timeout = remove_timeout;
        self
    }

    /// A reading command. A command that could write is refused with no process started; nothing here panics.
    pub fn read(&self, cwd: &str, args: &[&str]) -> GitOutput {
        if let Err(reason) = assert_read_only(args) {
            return GitOutput::refused(&reason);
        }
        let full: Vec<String> = GIT_PREFIX
            .iter()
            .chain(args)
            .map(ToString::to_string)
            .collect();
        let slot = self.gate.acquire();
        let out = self.exec.exec(cwd, &full, self.read_timeout);
        if out.timed_out {
            locked(&self.gate.stats).timed_out += 1;
        }
        drop(slot);
        out
    }

    /// One of the removal's own commands.
    pub fn write(&self, cwd: &str, args: &[&str]) -> GitOutput {
        if let Err(reason) = assert_write_ok(args) {
            return GitOutput::refused(&reason);
        }
        let full: Vec<String> = args.iter().map(ToString::to_string).collect();
        let timeout = if args.starts_with(&["worktree", "remove"]) {
            self.remove_timeout
        } else {
            self.write_timeout
        };
        self.exec.exec(cwd, &full, timeout)
    }

    pub fn stats(&self) -> GitStats {
        *locked(&self.gate.stats)
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Barrier;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    struct Recording(Mutex<Vec<Vec<String>>>);

    impl GitExec for Recording {
        fn exec(&self, _cwd: &str, args: &[String], _timeout: Duration) -> GitOutput {
            self.0.lock().unwrap().push(args.to_vec());
            GitOutput::success("")
        }
    }

    #[test]
    fn every_read_carries_the_lock_free_prefix_and_a_write_is_refused_before_a_process() {
        let exec = Arc::new(Recording(Mutex::new(Vec::new())));
        let git = Git::new(exec.clone());
        assert!(git.read("/x", &["status", "--porcelain=v1", "-b"]).ok());
        let seen = exec.0.lock().unwrap().clone();
        assert_eq!(
            seen[0],
            [
                "--no-optional-locks",
                "-c",
                "core.fsmonitor=false",
                "status",
                "--porcelain=v1",
                "-b"
            ]
        );
        let refused = git.read("/x", &["fetch"]);
        assert!(refused.error.unwrap().contains("not a read-only command"));
        let refused = git.write("/x", &["push"]);
        assert!(
            refused
                .error
                .unwrap()
                .contains("not a command the worktree removal may run")
        );
        assert_eq!(
            exec.0.lock().unwrap().len(),
            1,
            "the refused calls started nothing"
        );
        assert!(git.write("/x", &["worktree", "prune"]).ok());
        assert_eq!(
            exec.0.lock().unwrap()[1],
            ["worktree", "prune"],
            "a write has no prefix"
        );
    }

    /// A git that answers after `takes`, unless the limit it is given is shorter: then it is cut like a real one.
    struct Takes {
        takes: Duration,
        seen: Mutex<Vec<Duration>>,
    }

    impl GitExec for Takes {
        fn exec(&self, _cwd: &str, _args: &[String], timeout: Duration) -> GitOutput {
            self.seen.lock().unwrap().push(timeout);
            if self.takes > timeout {
                return GitOutput {
                    timed_out: true,
                    error: Some(format!("git timed out after {} s", timeout.as_secs())),
                    ..GitOutput::default()
                };
            }
            GitOutput::success("ok")
        }
    }

    fn takes(ms: u64) -> Arc<Takes> {
        Arc::new(Takes {
            takes: Duration::from_millis(ms),
            seen: Mutex::new(Vec::new()),
        })
    }

    #[test]
    fn worktree_remove_alone_gets_the_long_timeout_every_other_write_the_short_one() {
        assert_eq!(REMOVE_TIMEOUT, Duration::from_secs(15 * 60));
        assert!(REMOVE_TIMEOUT > WRITE_TIMEOUT);
        let exec = takes(0);
        let git = Git::new(exec.clone());
        git.write("/x", &["worktree", "remove", "X:/a/b"]);
        git.write("/x", &["worktree", "remove", "--force", "X:/a/b"]);
        git.write("/x", &["worktree", "prune"]);
        assert_eq!(
            *exec.seen.lock().unwrap(),
            [REMOVE_TIMEOUT, REMOVE_TIMEOUT, WRITE_TIMEOUT]
        );
    }

    #[test]
    fn a_removal_slower_than_the_short_timeout_finishes_and_a_slow_prune_is_cut() {
        let git = Git::with_limits(takes(1500), 2, READ_TIMEOUT, Duration::from_secs(1))
            .with_remove_timeout(Duration::from_secs(5));
        let slow_remove = git.write("/x", &["worktree", "remove", "X:/a/b"]);
        assert_eq!(
            (
                slow_remove.code,
                slow_remove.stdout.as_str(),
                slow_remove.timed_out
            ),
            (Some(0), "ok", false)
        );
        let slow_prune = git.write("/x", &["worktree", "prune"]);
        assert_eq!(
            (
                slow_prune.code,
                slow_prune.timed_out,
                slow_prune.error.as_deref()
            ),
            (None, true, Some("git timed out after 1 s"))
        );
        let cut = Git::with_limits(takes(5000), 2, READ_TIMEOUT, Duration::from_secs(5))
            .with_remove_timeout(Duration::from_secs(1))
            .write("/x", &["worktree", "remove", "--force", "X:/a/b"]);
        assert_eq!(
            (cut.code, cut.timed_out, cut.error.as_deref()),
            (None, true, Some("git timed out after 1 s")),
            "the removal timeout is its own"
        );
    }

    /// Every run waits for a second one to be running with it, so two at once is proven and a third would show.
    struct Slow {
        running: AtomicUsize,
        peak: AtomicUsize,
        pair: Barrier,
    }

    impl GitExec for Slow {
        fn exec(&self, _cwd: &str, _args: &[String], _timeout: Duration) -> GitOutput {
            let now = self.running.fetch_add(1, Ordering::SeqCst) + 1;
            self.peak.fetch_max(now, Ordering::SeqCst);
            self.pair.wait();
            self.running.fetch_sub(1, Ordering::SeqCst);
            GitOutput::success("ok")
        }
    }

    #[test]
    fn at_most_two_git_processes_at_a_time_however_many_are_asked_for() {
        let exec = Arc::new(Slow {
            running: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            pair: Barrier::new(2),
        });
        let git = Arc::new(Git::new(exec.clone()));
        let workers: Vec<_> = (0..12)
            .map(|_| {
                let git = git.clone();
                thread::spawn(move || git.read("/x", &["log", "-1"]))
            })
            .collect();
        for worker in workers {
            assert_eq!(worker.join().unwrap().stdout, "ok");
        }
        assert_eq!(exec.peak.load(Ordering::SeqCst), 2);
        let stats = git.stats();
        assert_eq!(
            (stats.max_running, stats.running, stats.started),
            (2, 0, 12)
        );
    }

    struct Hangs;

    impl GitExec for Hangs {
        fn exec(&self, _cwd: &str, args: &[String], _timeout: Duration) -> GitOutput {
            if args.iter().any(|a| a == "hang") {
                return GitOutput {
                    timed_out: true,
                    error: Some("git timed out after 10 s".into()),
                    ..GitOutput::default()
                };
            }
            GitOutput::success("fine")
        }
    }

    #[test]
    fn a_git_that_hangs_is_counted_and_does_not_hold_its_slot() {
        let git = Git::with_limits(Arc::new(Hangs), 1, READ_TIMEOUT, WRITE_TIMEOUT);
        let slow = git.read("/x", &["log", "hang"]);
        let fast = git.read("/x", &["log", "-1"]);
        assert!(slow.timed_out);
        assert_eq!(fast.stdout, "fine");
        assert_eq!(git.stats().timed_out, 1);
    }

    #[test]
    fn a_missing_git_is_a_result_not_a_panic() {
        let git = Git::new(Arc::new(SystemExec::new("argus-no-such-git")));
        let out = git.read(&std::env::temp_dir().to_string_lossy(), &["log"]);
        assert_eq!(out.code, None);
        assert!(out.error.is_some());
    }

    #[test]
    fn a_missing_folder_starts_no_process_and_reads_as_not_a_repository() {
        let git = Git::system();
        let out = git.read("/definitely/not/a/folder", &["log"]);
        assert_eq!(out.code, Some(128));
        assert!(out.stderr.contains("cannot change to"));
    }
}
