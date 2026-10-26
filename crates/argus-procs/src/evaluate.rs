//! One sample against the rules: the listed processes with their flags, and the per-rule totals.

use crate::clock::iso;
use crate::guard::GuardCtx;
use crate::hint::hint_of;
use crate::matcher::{CompiledRule, match_rule};
use crate::num::{mem_mb, to_f64};
use crate::project::project_of;
use crate::rule::Rule;
use crate::text::one_line;
use crate::tracker::{Tracked, Tracker};
use crate::tree::descendants_of;
use crate::types::{Flag, ProcDoc, ProcRow, Sample, Status};
use std::collections::{HashMap, HashSet};

const CMD_CLIP: usize = 200;
const MINUTE_MS: i64 = 60_000;
const PARENT_SLACK_MS: i64 = 1000;

/// Everything one evaluation reads.
pub struct EvalInput<'a> {
    pub sample: &'a Sample,
    pub rules: &'a [Rule],
    pub compiled: &'a [CompiledRule],
    pub roots: &'a [(String, String)],
    pub guard: &'a GuardCtx,
}

/// The processes of one rule that the sample matched, listed or not.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Group {
    pub count: usize,
    pub mem_mb: i64,
}

/// The listing of one sample.
#[derive(Debug, Default)]
pub struct Evaluation {
    pub docs: Vec<ProcDoc>,
    pub groups: HashMap<String, Group>,
    pub over: Vec<String>,
}

struct Facts {
    mem_mb: i64,
    age_min: i64,
    idle_min: i64,
    idle: bool,
    parent_alive: bool,
}

struct Env<'a> {
    input: &'a EvalInput<'a>,
    by_pid: HashMap<u32, &'a ProcRow>,
}

fn exceeds(limit: Option<f64>, value: i64) -> bool {
    limit.is_some_and(|max| to_f64(value) > max)
}

fn flags_of(rule: &Rule, proc: &ProcRow, facts: &Facts) -> Vec<Flag> {
    let stale = facts.idle
        && rule
            .idle_min
            .is_some_and(|min| to_f64(facts.idle_min) >= min);
    [
        (
            Flag::Hung,
            rule.flags_hung() && proc.responding == Some(false),
        ),
        (Flag::Hog, exceeds(rule.max_mem_mb, facts.mem_mb)),
        (Flag::Orphan, rule.flags_orphans() && !facts.parent_alive),
        (Flag::Stale, stale),
        (Flag::Old, exceeds(rule.max_age_min, facts.age_min)),
    ]
    .into_iter()
    .filter_map(|(flag, on)| on.then_some(flag))
    .collect()
}

fn is_listed(rule: &Rule, mem_mb: i64, flags: &[Flag]) -> bool {
    let above_floor = rule.min_mem_mb.is_none_or(|min| to_f64(mem_mb) >= min);
    above_floor && !(rule.lists_only_flagged() && flags.is_empty())
}

fn minutes_since(now: i64, then: i64) -> i64 {
    (now - then).div_euclid(MINUTE_MS).max(0)
}

impl<'a> Env<'a> {
    fn new(input: &'a EvalInput<'a>) -> Self {
        let by_pid = input.sample.procs.iter().map(|p| (p.pid, p)).collect();
        Self { input, by_pid }
    }

    fn hidden(&self) -> HashSet<u32> {
        let Some(pid) = self.input.sample.sampler_pid.filter(|p| *p != 0) else {
            return HashSet::new();
        };
        let mut out = descendants_of(pid, &self.input.sample.procs);
        out.insert(pid);
        out
    }

    fn parent_alive(&self, proc: &ProcRow, start_ms: i64) -> bool {
        let parent = self.by_pid.get(&proc.ppid);
        if self.input.sample.is_windows() {
            proc.ppid == 0
                || parent.is_some_and(|p| {
                    p.start_ms == 0 || start_ms == 0 || p.start_ms <= start_ms + PARENT_SLACK_MS
                })
        } else {
            proc.ppid != 1 && parent.is_some()
        }
    }

    fn facts(&self, proc: &ProcRow, reading: &Tracked) -> Facts {
        let now = self.input.sample.at;
        Facts {
            mem_mb: mem_mb(proc.mem_bytes),
            age_min: if reading.start_ms == 0 {
                0
            } else {
                minutes_since(now, reading.start_ms)
            },
            idle_min: reading
                .idle_since
                .map_or(0, |since| minutes_since(now, since)),
            idle: reading.idle_since.is_some(),
            parent_alive: self.parent_alive(proc, reading.start_ms),
        }
    }

    fn assess(
        &self,
        tracker: &mut Tracker,
        proc: &ProcRow,
        rule: &Rule,
        reading: &Tracked,
    ) -> Option<ProcDoc> {
        let now = self.input.sample.at;
        let facts = self.facts(proc, reading);
        let flags = flags_of(rule, proc, &facts);
        if !is_listed(rule, facts.mem_mb, &flags) {
            return None;
        }
        let status = Status::from_flags(&flags);
        let since = tracker.settle(reading.key, status, now);
        let guard = self
            .input
            .guard
            .guard_of(proc, self.input.sample.is_windows());
        Some(ProcDoc {
            id: Tracker::id_of(reading.key),
            pid: proc.pid,
            ppid: proc.ppid,
            name: proc.name.clone(),
            cmd: one_line(&proc.cmd, CMD_CLIP),
            hint: hint_of(&proc.cmd),
            rule: rule.name.clone(),
            project: project_of(proc, self.input.roots),
            mem_mb: facts.mem_mb,
            cpu_pct: reading.cpu_pct,
            started_at: (reading.start_ms != 0).then(|| iso(reading.start_ms)),
            age_min: facts.age_min,
            idle_min: facts.idle_min,
            responding: proc.responding,
            parent_alive: facts.parent_alive,
            status,
            flags,
            protected: guard.is_some(),
            protected_why: guard,
            killable: guard.is_none() && rule.is_killable(),
            since: iso(since),
        })
    }
}

fn mark_groups(ev: &mut Evaluation, rules: &[Rule]) {
    ev.over = rules
        .iter()
        .filter(|r| {
            let total = ev.groups.get(&r.name).map_or(0, |g| g.mem_mb);
            exceeds(r.max_group_mem_mb, total)
        })
        .map(|r| r.name.clone())
        .collect();
    for doc in &mut ev.docs {
        if ev.over.contains(&doc.rule) {
            doc.flags.push(Flag::Group);
        }
    }
}

/// Sample -> docs and per-rule totals. The tracker learns CPU, idle time and status as a side effect.
pub fn evaluate(tracker: &mut Tracker, input: &EvalInput<'_>) -> Evaluation {
    let env = Env::new(input);
    let hidden = env.hidden();
    let mut ev = Evaluation::default();
    let mut seen = HashSet::new();
    for proc in input
        .sample
        .procs
        .iter()
        .filter(|p| !hidden.contains(&p.pid))
    {
        let Some(rule) = match_rule(input.compiled, proc) else {
            continue;
        };
        let reading = tracker.track(proc, input.sample.at);
        seen.insert(reading.key);
        let group = ev.groups.entry(rule.name.clone()).or_default();
        group.count += 1;
        group.mem_mb += mem_mb(proc.mem_bytes);
        if let Some(doc) = env.assess(tracker, proc, rule, &reading) {
            ev.docs.push(doc);
        }
    }
    tracker.retain(&seen);
    mark_groups(&mut ev, input.rules);
    ev
}
