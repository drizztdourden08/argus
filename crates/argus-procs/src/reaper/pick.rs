//! What the reaper takes from one sample: orphaned or long searches, and test or build runs
//! whose tree went idle or ran past the cap. Pure: the sample's own time is the clock.

use super::{ReaperConfig, ReaperState};
use crate::text::{base_name, slashed};
use crate::tree::{chain_of, descendants_of};
use crate::types::{ProcRow, Sample};
use argus_core::clock::age_text;
use fancy_regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const SEARCHES: [&str; 4] = ["find", "grep", "rg", "du"];
const BUILDS: [&str; 5] = ["cargo", "vitest", "tsc", "eslint", "esbuild"];
/// Cargo verbs that run a program the owner may be using, not a build or a test.
const CARGO_KEEPS: [&str; 4] = ["run", "r", "watch", "install"];
const PARENT_SLACK_MS: i64 = 1000;

/// A node process whose script is one of the tools (`.../vitest/vitest.mjs`, `.../bin/tsc`).
static NODE_TOOL: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r#"(?:^|[\s/"'])(vitest|tsc|eslint|esbuild)(?:[.](?:m|c)?js)?(?:["'\s]|$)"#).ok()
});

/// One tree the reaper means to end: its first process, the others, and why.
#[derive(Debug, Clone)]
pub(crate) struct Candidate {
    pub root: ProcRow,
    pub kids: Vec<u32>,
    pub why: String,
}

/// Whether the process still has a live parent: on Windows a parent no younger than the process
/// (pids are reused), elsewhere any parent row that is not init.
pub(super) fn parent_alive(sample: &Sample, parent: Option<&ProcRow>, proc: &ProcRow) -> bool {
    if sample.is_windows() {
        proc.ppid == 0
            || parent.is_some_and(|p| {
                p.start_ms == 0
                    || proc.start_ms == 0
                    || p.start_ms <= proc.start_ms + PARENT_SLACK_MS
            })
    } else {
        proc.ppid != 1 && parent.is_some()
    }
}

struct View<'a> {
    sample: &'a Sample,
    by_pid: HashMap<u32, &'a ProcRow>,
}

impl View<'_> {
    fn parent_alive(&self, proc: &ProcRow) -> bool {
        parent_alive(self.sample, self.by_pid.get(&proc.ppid).copied(), proc)
    }

    fn age(&self, proc: &ProcRow) -> i64 {
        if proc.start_ms == 0 {
            0
        } else {
            (self.sample.at - proc.start_ms).max(0)
        }
    }

    fn kids(&self, pid: u32) -> Vec<u32> {
        let mut kids: Vec<u32> = descendants_of(pid, &self.sample.procs)
            .into_iter()
            .collect();
        kids.sort_unstable();
        kids
    }
}

/// The verb after `cargo` (`build`, `test`, `run`), skipping flags and `+toolchain`.
fn cargo_verb(cmd: &str) -> Option<String> {
    let line = slashed(cmd).to_lowercase();
    let words: Vec<&str> = line.split_whitespace().collect();
    let at = words.iter().position(|w| {
        let last = w.trim_matches('"').rsplit('/').next().unwrap_or("");
        base_name(last) == "cargo"
    })?;
    words[at + 1..]
        .iter()
        .find(|w| !w.starts_with('-') && !w.starts_with('+'))
        .map(|w| (*w).to_string())
}

/// The tool a process runs, when it is one the reaper watches.
fn tool_of(proc: &ProcRow) -> Option<&'static str> {
    let base = base_name(&proc.name);
    if let Some(tool) = BUILDS.iter().copied().find(|b| *b == base.as_str()) {
        let keeps = tool == "cargo"
            && cargo_verb(&proc.cmd).is_some_and(|v| CARGO_KEEPS.contains(&v.as_str()));
        return (!keeps).then_some(tool);
    }
    if base != "node" {
        return None;
    }
    let line = slashed(&proc.cmd).to_lowercase();
    let caps = NODE_TOOL.as_ref()?.captures(&line).ok().flatten()?;
    let word = caps.get(1)?.as_str();
    BUILDS.iter().copied().find(|b| *b == word)
}

fn search_why(view: &View<'_>, proc: &ProcRow, cfg: &ReaperConfig) -> Option<String> {
    if !SEARCHES.contains(&base_name(&proc.name).as_str()) {
        return None;
    }
    if !view.parent_alive(proc) {
        return Some("an orphaned search (whatever started it is gone)".to_string());
    }
    let age = view.age(proc);
    (age > cfg.search_ms).then(|| format!("a search running for {}", age_text(age)))
}

/// The tool of a run's first process: a watched tool none of whose parents is one. An esbuild
/// service under a live parent belongs to that parent (a dev server), never a run of its own.
fn run_root(view: &View<'_>, proc: &ProcRow) -> Option<&'static str> {
    let tool = tool_of(proc)?;
    if tool == "esbuild" && proc.cmd.contains("--service") && view.parent_alive(proc) {
        return None;
    }
    let parents = chain_of(proc.pid, &view.sample.procs);
    let under_tool = parents
        .iter()
        .filter(|pid| **pid != proc.pid)
        .filter_map(|pid| view.by_pid.get(pid))
        .any(|p| tool_of(p).is_some());
    (!under_tool).then_some(tool)
}

fn run_why(
    view: &View<'_>,
    state: &mut ReaperState,
    (proc, tool, kids): (&ProcRow, &str, &[u32]),
    cfg: &ReaperConfig,
) -> Option<String> {
    let cpu = proc.cpu_ms
        + kids
            .iter()
            .filter_map(|k| view.by_pid.get(k))
            .map(|k| k.cpu_ms)
            .sum::<i64>();
    let idle = state.idle_for((proc.pid, proc.start_ms), cpu, view.sample.at);
    let age = view.age(proc);
    if age > cfg.cap_ms {
        return Some(format!(
            "a {tool} run older than the {} cap",
            age_text(cfg.cap_ms)
        ));
    }
    (idle >= cfg.idle_ms).then(|| format!("a {tool} run with no CPU time for {}", age_text(idle)))
}

/// The trees to end in this sample, none inside another; forgets the runs that are gone.
pub(crate) fn pick(sample: &Sample, state: &mut ReaperState, cfg: &ReaperConfig) -> Vec<Candidate> {
    let view = View {
        sample,
        by_pid: sample.procs.iter().map(|p| (p.pid, p)).collect(),
    };
    let mut out: Vec<Candidate> = Vec::new();
    let mut runs = HashSet::new();
    for proc in &sample.procs {
        if let Some(why) = search_why(&view, proc, cfg) {
            let kids = view.kids(proc.pid);
            out.push(Candidate {
                root: proc.clone(),
                kids,
                why,
            });
            continue;
        }
        let Some(tool) = run_root(&view, proc) else {
            continue;
        };
        runs.insert((proc.pid, proc.start_ms));
        let kids = view.kids(proc.pid);
        if let Some(why) = run_why(&view, state, (proc, tool, &kids), cfg) {
            out.push(Candidate {
                root: proc.clone(),
                kids,
                why,
            });
        }
    }
    state.keep_runs(&runs);
    let inside: HashSet<u32> = out.iter().flat_map(|c| c.kids.iter().copied()).collect();
    out.retain(|c| !inside.contains(&c.root.pid));
    out
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, cmd: &str) -> ProcRow {
        ProcRow {
            pid: 1,
            ppid: 0,
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

    #[test]
    fn the_tools_are_named_by_executable_or_by_node_script() {
        let vitest = "node x:/a/node_modules/vitest/vitest.mjs run";
        assert_eq!(tool_of(&row("node.exe", vitest)), Some("vitest"));
        let tsc = "node X:\\a\\node_modules\\typescript\\bin\\tsc -p .";
        assert_eq!(tool_of(&row("node.exe", tsc)), Some("tsc"));
        let eslint = "node x:/a/node_modules/eslint/bin/eslint.js .";
        assert_eq!(tool_of(&row("node.exe", eslint)), Some("eslint"));
        let editor = "node c:/vscode/extensions/eslint/server/out/eslintServer.js";
        assert_eq!(tool_of(&row("node.exe", editor)), None);
        assert_eq!(tool_of(&row("node.exe", "node tsserver.js")), None);
        assert_eq!(tool_of(&row("cargo.exe", "cargo test -p a")), Some("cargo"));
        assert_eq!(tool_of(&row("cargo.exe", "cargo +nightly run")), None);
        assert_eq!(tool_of(&row("rustc.exe", "rustc --crate-name a")), None);
    }
}
