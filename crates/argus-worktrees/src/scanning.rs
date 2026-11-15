//! Scans and the live check: how the service fills and keeps its picture of the disk.

use crate::attribution::{Entry, listed_here, own_facts};
use crate::clock::{Clock, SystemClock};
use crate::doc::Derived;
use crate::fact::Fact;
use crate::inspect::{ScanCtx, inspect_dir};
use crate::measure::{Leftover, measure_folder};
use crate::paths::{clean_path, path_key};
use crate::probe::{fingerprint, shape_of};
use crate::rule::{Rule, RuleKind};
use crate::scan::{
    DeletingFolder, RuleInfo, ScanEnv, folder_candidates, mark_leftover, scan_rules_with,
};
use crate::service::{Measured, ScanState, Service};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::time::Instant;

/// What the live check did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiveResult {
    pub dropped: usize,
    pub added: Vec<String>,
}

fn elapsed_ms(since: Instant) -> i64 {
    i64::try_from(since.elapsed().as_millis()).unwrap_or(i64::MAX)
}

/// The scan of a project with no rules: nothing found.
fn empty_scan(at_ms: i64, full_ms: i64, shape: String) -> ScanState {
    ScanState {
        at_ms,
        took_ms: 0,
        rules: Vec::new(),
        facts: Vec::new(),
        listed: HashSet::new(),
        error: None,
        full_ms,
        shape,
        prints: HashMap::new(),
    }
}

fn rules_signature(rules: &[Rule]) -> String {
    serde_json::to_string(rules).unwrap_or_default()
}

impl Service {
    /// A leftover is measured at most once per `remeasure_ms` unless its top-level folder changed, and a scan spends at
    /// most `budget_ms` measuring (the rest show as pending and are measured by a later scan).
    fn measure_leftover(&self, path: &str) -> Leftover {
        let key = path_key(path, self.ci());
        let mtime = self.deps.fs.mtime_ms(path);
        let hit = {
            let state = self.st();
            state
                .measured
                .get(&key)
                .map(|m| (m.mtime, m.at, m.value.clone()))
        };
        if let Some((hit_mtime, at, value)) = &hit {
            if *hit_mtime == mtime && self.now() - at < self.opts.remeasure_ms {
                return value.clone();
            }
        }
        if self.st().budget_spent_ms >= self.opts.budget_ms {
            return hit.map_or_else(Leftover::pending, |(_, _, value)| value);
        }
        let started = Instant::now();
        let value = measure_folder(
            self.deps.fs.as_ref(),
            path,
            self.opts.measure_bounds,
            &|| SystemClock.now_ms(),
        );
        let mut state = self.st();
        state.budget_spent_ms += elapsed_ms(started);
        state.measured.insert(
            key,
            Measured {
                at: self.now(),
                mtime,
                value: value.clone(),
            },
        );
        value
    }

    /// Every project's board, in registration order.
    fn all_boards(&self) -> Vec<Entry> {
        let store = &self.deps.store;
        let named = |p: String| store.board(&p).map(|b| (p, b));
        store.projects().into_iter().filter_map(named).collect()
    }

    fn scan_env<'a>(
        &'a self,
        measure: &'a dyn Fn(&str) -> Leftover,
        known_repos: &'a [String],
    ) -> ScanEnv<'a> {
        ScanEnv {
            git: &self.deps.git,
            fs: self.deps.fs.as_ref(),
            case_insensitive: self.ci(),
            measure,
            known_repos,
        }
    }

    /// Scan one project now: read every rule and every worktree with git, mirror the result, tell the threads, forget
    /// what no longer exists. This is the forced scan: it never trusts a fingerprint. Whatever asks for a fresh look
    /// (a rule added, a removal, the request worker, the hygiene, the first scan) comes here.
    pub fn scan_now(&self, project: &str) -> Option<Derived> {
        self.scan_with(project, false)
    }

    /// A scan that keeps the facts of a worktree whose fingerprint did not change (`keep`), or reads all with git.
    pub(crate) fn scan_with(&self, project: &str, keep: bool) -> Option<Derived> {
        let _one = self.one_at_a_time(project, "a scan of the project", None);
        if self.stopped() {
            return None;
        }
        let board = self.deps.store.board(project)?;
        self.st()
            .sigs
            .insert(project.to_string(), rules_signature(&board.rules));
        let started = self.now();
        let timer = Instant::now();
        let shape = shape_of(&board.rules, self.deps.fs.as_ref(), self.ci());
        let kept = if keep { self.kept_of(project) } else { None };
        let full_ms = kept.as_ref().map_or(started, |k| k.full_ms);
        let scan = if board.rules.is_empty() {
            empty_scan(started, full_ms, shape)
        } else {
            self.st().budget_spent_ms = 0;
            let measure = |path: &str| self.measure_leftover(path);
            let repos = self.known_repos();
            let env = self.scan_env(&measure, &repos);
            let mut found = scan_rules_with(&board.rules, &env, kept.as_ref());
            found.facts = own_facts(project, found.facts, &self.all_boards(), &repos, self.ci());
            self.adopt_deleting(project, &found.deleting, &board.rules);
            ScanState {
                at_ms: self.now(),
                took_ms: elapsed_ms(timer),
                rules: found.rules,
                facts: found.facts,
                listed: found.listed,
                error: None,
                full_ms,
                shape,
                prints: found.prints,
            }
        };
        let clean = scan.error.is_none();
        self.log_scan_failures(project, &scan.rules);
        self.st().scans.insert(project.to_string(), scan);
        if self.stopped() {
            return None;
        }
        self.resume_batches(project);
        self.wake_step(project);
        self.hygiene_step(project);
        self.prune_state(project);
        if clean {
            self.prune_made(project);
        }
        self.derive(project)
    }

    /// One log line for each rule whose scan failed, when it did not fail the same way at the last scan.
    fn log_scan_failures(&self, project: &str, rules: &[RuleInfo]) {
        let before: Vec<RuleInfo> = self
            .st()
            .scans
            .get(project)
            .map(|s| s.rules.clone())
            .unwrap_or_default();
        for rule in rules {
            let Some(error) = &rule.error else {
                continue;
            };
            let same = before
                .iter()
                .any(|b| b.id == rule.id && b.error.as_ref() == Some(error));
            if !same {
                self.deps.notifier.problem(&format!(
                    "{project}: the worktree scan of rule {} failed: {error}",
                    rule.id
                ));
            }
        }
    }

    /// Scan every project, one after another.
    pub fn scan_all(&self) {
        for project in self.deps.store.projects() {
            self.scan_now(&project);
        }
    }

    /// Folders a crash left half deleted (`*.argus-deleting-*` under a current rule) are finished once per run, by a job.
    fn adopt_deleting(&self, project: &str, folders: &[DeletingFolder], rules: &[Rule]) {
        if !self.st().adopted.insert(project.to_string()) {
            return;
        }
        for folder in folders {
            if let Some(rule) = rules.iter().find(|r| r.id == folder.rule) {
                self.finish_deleting(project, &folder.path, rule);
            }
        }
    }

    /// Did the project's rules change since the last scan? The server scans again a moment after a change.
    pub fn rules_changed(&self, project: &str) -> bool {
        let Some(board) = self.deps.store.board(project) else {
            return false;
        };
        let signature = rules_signature(&board.rules);
        let mut state = self.st();
        let known = state
            .sigs
            .get(project)
            .cloned()
            .unwrap_or_else(|| "[]".to_string());
        let changed = known != signature;
        if changed {
            state.sigs.insert(project.to_string(), signature);
        }
        changed
    }

    /// Did an agent of the project end (done, stopped or failed) since the last look at its board? The server scans
    /// again a moment after, and the hygiene step that follows the scan sweeps the worktree it left. The first look
    /// only remembers: the first scan at the start sweeps everything.
    pub fn agents_ended(&self, project: &str) -> bool {
        let Some(board) = self.deps.store.board(project) else {
            return false;
        };
        let ended: BTreeSet<String> = board
            .agents
            .iter()
            .filter(|a| !a.live() && !a.status.is_empty())
            .map(|a| a.id.clone())
            .collect();
        let before = self
            .st()
            .ended_agents
            .insert(project.to_string(), ended.clone());
        before.is_some_and(|b| !ended.is_subset(&b))
    }

    /// Has the project been scanned since the service started? Before that its worktrees are not known.
    pub fn scanned(&self, project: &str) -> bool {
        self.st().scans.contains_key(project)
    }

    /// How many leftovers the last scan had no time to measure: the server scans again soon to measure the next ones.
    pub fn pending_count(&self, project: &str) -> usize {
        self.st().scans.get(project).map_or(0, |s| {
            s.facts
                .iter()
                .filter(|f| f.leftover.as_ref().is_some_and(|l| l.pending))
                .count()
        })
    }

    fn vanished(&self, scan_facts: Vec<Fact>) -> (Vec<Fact>, usize) {
        let mut dropped = 0;
        let mut kept = Vec::new();
        for mut fact in scan_facts {
            if fact.exists && !self.deps.fs.exists(&fact.path) {
                dropped += 1;
                if !fact.is_git || !fact.registered || fact.main {
                    continue;
                }
                fact.exists = false;
                fact.prunable
                    .get_or_insert_with(|| "folder is gone".to_string());
            }
            kept.push(fact);
        }
        (kept, dropped)
    }

    fn new_folders(&self, project: &str, facts: &[Fact]) -> Vec<(String, Rule)> {
        let Some(board) = self.deps.store.board(project) else {
            return Vec::new();
        };
        let mut known: HashSet<String> =
            facts.iter().map(|f| path_key(&f.path, self.ci())).collect();
        let mut added = Vec::new();
        for rule in board
            .rules
            .iter()
            .filter(|r| r.kind == RuleKind::Folder && r.problem().is_none())
        {
            for dir in folder_candidates(rule, self.deps.fs.as_ref(), self.ci()) {
                if known.insert(path_key(&dir, self.ci())) {
                    added.push((dir, rule.clone()));
                }
            }
        }
        added
    }

    /// The live check: no git for what vanished. A listed folder that is gone leaves the docs at once (a linked worktree
    /// becomes `gone`, anything else is forgotten). A new folder under a folder rule is read with git and joins them.
    pub fn live_step(&self, project: &str) -> LiveResult {
        let facts = {
            let state = self.st();
            match state.scans.get(project) {
                Some(scan) if !state.stopped => scan.facts.clone(),
                _ => return LiveResult::default(),
            }
        };
        let (kept, dropped) = self.vanished(facts);
        let added = self.new_folders(project, &kept);
        if let Some(scan) = self.st().scans.get_mut(project) {
            scan.facts = kept;
        }
        let result = LiveResult {
            dropped,
            added: added.iter().map(|(dir, _)| dir.clone()).collect(),
        };
        if !added.is_empty() {
            self.read_added(project, &added);
        }
        result
    }

    fn read_added(&self, project: &str, added: &[(String, Rule)]) {
        let _one = self.one_at_a_time(project, "a look at new folders", None);
        let listed = match self.st().scans.get(project) {
            Some(scan) => scan.listed.clone(),
            None => return,
        };
        self.st().budget_spent_ms = 0;
        let measure = |path: &str| self.measure_leftover(path);
        let env = self.scan_env(&measure, &[]);
        let (boards, repos) = (self.all_boards(), self.known_repos());
        let mut ctx = ScanCtx {
            git: env.git,
            fs: env.fs,
            lists: Vec::new(),
            case_insensitive: env.case_insensitive,
        };
        for (dir, rule) in added {
            let print = fingerprint(env.fs, &clean_path(dir));
            let mut fact = inspect_dir(dir, rule, &mut ctx);
            mark_leftover(&mut fact, &listed, &env);
            if !listed_here(project, &fact, &boards, &repos, self.ci()) {
                continue;
            }
            if let Some(scan) = self.st().scans.get_mut(project) {
                scan.prints.insert(fact.path.clone(), print);
                if !scan.facts.iter().any(|f| f.id == fact.id) {
                    scan.facts.push(fact);
                }
            }
        }
    }
}
