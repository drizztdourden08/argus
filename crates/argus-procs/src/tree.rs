//! The process tree as far as one sample knows it.

use crate::types::ProcRow;
use std::collections::{HashMap, HashSet};

const CHAIN_LIMIT: usize = 30;
const CLOCK_SLACK_MS: i64 = 1000;

/// The processes this one started, from the sample (a child older than its parent is a recycled pid).
#[must_use]
pub fn descendants_of(pid: u32, procs: &[ProcRow]) -> HashSet<u32> {
    let mut kids: HashMap<u32, Vec<&ProcRow>> = HashMap::new();
    for p in procs {
        kids.entry(p.ppid).or_default().push(p);
    }
    let by_pid: HashMap<u32, &ProcRow> = procs.iter().map(|p| (p.pid, p)).collect();
    let mut out = HashSet::new();
    let mut stack = vec![pid];
    while let Some(id) = stack.pop() {
        let parent_start = by_pid.get(&id).map_or(0, |p| p.start_ms);
        for kid in kids.get(&id).into_iter().flatten() {
            let recycled = parent_start != 0
                && kid.start_ms != 0
                && kid.start_ms + CLOCK_SLACK_MS < parent_start;
            if kid.pid != id && !recycled && out.insert(kid.pid) {
                stack.push(kid.pid);
            }
        }
    }
    out
}

/// This process and its parents, as far as the sample knows them.
#[must_use]
pub fn chain_of(pid: u32, procs: &[ProcRow]) -> HashSet<u32> {
    let by_pid: HashMap<u32, &ProcRow> = procs.iter().map(|p| (p.pid, p)).collect();
    let mut out = HashSet::from([pid]);
    let mut current = by_pid.get(&pid);
    for _ in 0..CHAIN_LIMIT {
        let Some(cur) = current else { break };
        let Some(parent) = by_pid.get(&cur.ppid) else {
            break;
        };
        let younger = parent.start_ms != 0
            && cur.start_ms != 0
            && parent.start_ms > cur.start_ms + CLOCK_SLACK_MS;
        if younger || !out.insert(parent.pid) {
            break;
        }
        current = Some(parent);
    }
    out
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: u32, parent: u32, start_ms: i64) -> ProcRow {
        ProcRow {
            pid: id,
            ppid: parent,
            name: String::new(),
            cmd: String::new(),
            path: None,
            mem_bytes: 0,
            start_ms,
            cpu_ms: 0,
            responding: None,
            cwd: None,
        }
    }

    #[test]
    fn descendants_skip_recycled_parents() {
        let procs = vec![
            row(1, 0, 5000),
            row(2, 1, 6000),
            row(3, 2, 7000),
            row(4, 1, 100),
        ];
        let kids = descendants_of(1, &procs);
        assert_eq!(kids, HashSet::from([2, 3]));
    }

    #[test]
    fn the_chain_climbs_to_the_oldest_known_parent() {
        let procs = vec![row(1, 0, 100), row(2, 1, 200), row(3, 2, 300)];
        assert_eq!(chain_of(3, &procs), HashSet::from([1, 2, 3]));
        assert_eq!(chain_of(9, &procs), HashSet::from([9]));
    }
}
