//! Kills: only a listed, unprotected process of a killable rule, checked against a fresh sample.

use crate::clock::{iso, parse_iso};
use crate::error::ProcsError;
use crate::ports::{Activity, Level};
use crate::service::{Engine, Inner, locked};
use crate::text::{age_text, base_name, mem_text};
use crate::tree::descendants_of;
use crate::types::{ProcDoc, Sample};
use serde::Serialize;
use std::thread;
use std::time::{Duration, Instant};

const WAIT_GONE: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(50);
const DEFAULT_IDLE_LIMIT: f64 = 30.0;
const MINUTE_MS: i64 = 60_000;

/// Who asked for the kill.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Source {
    #[default]
    Page,
    Cli {
        actor: Option<String>,
    },
}

impl Source {
    fn text(&self) -> String {
        match self {
            Self::Page => "from the page".to_string(),
            Self::Cli { actor: Some(actor) } => format!("from the CLI ({actor})"),
            Self::Cli { actor: None } => "from the CLI".to_string(),
        }
    }
}

/// Kill one process: its pid and the creation time the listing showed.
#[derive(Debug, Clone, Default)]
pub struct KillRequest {
    pub pid: u32,
    pub started_at: Option<String>,
    pub tree: bool,
    pub reason: Option<String>,
    pub source: Source,
}

/// Kill every killable process of a rule, optionally only the old or the idle ones.
#[derive(Debug, Clone, Default)]
pub struct KillRuleRequest {
    pub rule: String,
    pub older_than_min: Option<f64>,
    pub idle_only: bool,
    pub reason: Option<String>,
    pub source: Source,
}

/// The answer to one kill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KillOutcome {
    pub killed: bool,
    pub pid: u32,
    pub name: String,
    #[serde(rename = "memMB")]
    pub mem_mb: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// A process a kill-rule left alone, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Skipped {
    pub pid: u32,
    pub name: String,
    pub why: String,
}

/// The answer to a kill-rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KillRuleOutcome {
    pub rule: String,
    pub killed: Vec<KillOutcome>,
    pub failed: Vec<KillOutcome>,
    pub skipped: Vec<Skipped>,
}

fn proc_text(doc: &ProcDoc) -> String {
    let idle = (doc.idle_min > 0).then(|| format!("idle {}", age_text(doc.idle_min * MINUTE_MS)));
    let parts = [
        Some(doc.hint.clone().unwrap_or_else(|| doc.rule.clone())),
        Some(mem_text(doc.mem_mb)),
        idle,
    ];
    let inside: Vec<String> = parts.into_iter().flatten().collect();
    format!(
        "{} {} ({})",
        base_name(&doc.name),
        doc.pid,
        inside.join(", ")
    )
}

fn because(reason: Option<&str>) -> String {
    reason.map_or(String::new(), |r| format!(": {r}"))
}

impl Inner {
    fn note(&self, level: Level, text: String) {
        self.ports.host.activity(&Activity {
            kind: "process-kill",
            level,
            text,
        });
    }

    fn refuse(&self, error: ProcsError, subject: &str, request: &KillRequest) -> ProcsError {
        let reason = request
            .reason
            .as_deref()
            .map_or(String::new(), |r| format!(" (reason given: {r})"));
        self.note(
            Level::Warn,
            format!(
                "refused to kill {subject} {}: {}{reason}",
                request.source.text(),
                error.message()
            ),
        );
        error
    }

    fn tree_refusal(&self, doc: &ProcDoc, raw: &Sample) -> Option<ProcsError> {
        let guard = self.guard_for(raw);
        let windows = raw.is_windows();
        descendants_of(doc.pid, &raw.procs).into_iter().find_map(|pid| {
            let proc = raw.procs.iter().find(|p| p.pid == pid)?;
            let why = guard.guard_of(proc, windows)?;
            Some(ProcsError::forbidden(format!(
                "the tree of {} {} holds a protected process ({} {}, {}): kill it without --tree",
                base_name(&doc.name),
                doc.pid,
                base_name(&proc.name),
                proc.pid,
                why.as_str()
            )))
        })
    }

    fn refusal_of(&self, doc: &ProcDoc, tree: bool) -> Option<ProcsError> {
        let name = base_name(&doc.name);
        if let Some(why) = doc.protected_why {
            return Some(ProcsError::forbidden(format!(
                "{name} {} is protected ({}): it is never killed from here",
                doc.pid,
                why.as_str()
            )));
        }
        if !doc.killable {
            return Some(ProcsError::forbidden(format!(
                "{name} {} belongs to rule {}, which is not killable",
                doc.pid, doc.rule
            )));
        }
        let raw = locked(&self.view).raw.clone();
        raw.filter(|_| tree)
            .and_then(|raw| self.tree_refusal(doc, &raw))
    }

    fn wait_gone(&self, pid: u32) -> bool {
        let end = Instant::now() + WAIT_GONE;
        while self.ports.killer.is_alive(pid) && Instant::now() < end {
            thread::sleep(POLL);
        }
        !self.ports.killer.is_alive(pid)
    }

    fn kill_doc(
        &self,
        doc: &ProcDoc,
        tree: bool,
        reason: Option<&str>,
        source: &Source,
    ) -> KillOutcome {
        let raw = locked(&self.view).raw.clone();
        let descendants: Vec<u32> = raw
            .filter(|_| tree)
            .map(|r| descendants_of(doc.pid, &r.procs).into_iter().collect())
            .unwrap_or_default();
        let result = self.ports.killer.kill(doc.pid, tree, &descendants);
        let gone = result.ok && self.wait_gone(doc.pid);
        let why = because(reason);
        let who = source.text();
        if result.ok {
            let tree_text = if tree { " and its tree" } else { "" };
            let closing = if gone {
                ""
            } else {
                ", it may still be closing"
            };
            self.note(
                Level::Info,
                format!("killed {}{tree_text} {who}{closing}{why}", proc_text(doc)),
            );
        } else {
            let detail = result.detail.as_deref().unwrap_or("refused");
            self.note(
                Level::Error,
                format!("could not kill {} {who}: {detail}{why}", proc_text(doc)),
            );
        }
        KillOutcome {
            killed: result.ok,
            pid: doc.pid,
            name: doc.name.clone(),
            mem_mb: doc.mem_mb,
            error: (!result.ok).then(|| result.detail.unwrap_or_else(|| "refused".to_string())),
        }
    }

    fn listed(&self, pid: u32) -> Option<ProcDoc> {
        locked(&self.view)
            .listing
            .iter()
            .find(|d| d.pid == pid)
            .cloned()
    }

    fn check_target(&self, request: &KillRequest, want: i64) -> Result<ProcDoc, ProcsError> {
        let pid = request.pid;
        let refuse = |error| self.refuse(error, &pid.to_string(), request);
        let doc = self.listed(pid).ok_or_else(|| {
            refuse(ProcsError::not_found(format!(
                "process {pid} is not in the listing (gone, or no rule takes it)"
            )))
        })?;
        let listed_at = doc.started_at.as_deref().unwrap_or("");
        if parse_iso(listed_at) != Some(want) {
            return Err(refuse(ProcsError::conflict(format!(
                "process {pid} was created at {listed_at}, not {}: the pid was reused, nothing was killed",
                iso(want)
            ))));
        }
        match self.refusal_of(&doc, request.tree) {
            Some(error) => Err(refuse(error)),
            None => Ok(doc),
        }
    }

    /// `{killed, pid, name}`; `not_found`, `forbidden` or `conflict` otherwise.
    pub(crate) fn kill_one(&self, request: &KillRequest) -> Result<KillOutcome, ProcsError> {
        if request.pid == 0 {
            return Err(ProcsError::bad_request("pid must be a positive integer"));
        }
        let want = request
            .started_at
            .as_deref()
            .and_then(parse_iso)
            .ok_or_else(|| {
                ProcsError::bad_request("startedAt (the listed creation time) is required")
            })?;
        let mut engine = locked(&self.engine);
        self.sample_once(&mut engine);
        let doc = self.check_target(request, want)?;
        let outcome = self.kill_doc(
            &doc,
            request.tree,
            request.reason.as_deref(),
            &request.source,
        );
        self.sample_once(&mut engine);
        Ok(outcome)
    }

    fn skip_reason(
        &self,
        doc: &ProcDoc,
        request: &KillRuleRequest,
        idle_limit: f64,
    ) -> Option<String> {
        if let Some(error) = self.refusal_of(doc, false) {
            return Some(error.message().to_string());
        }
        let young = request
            .older_than_min
            .filter(|min| crate::num::to_f64(doc.age_min) < *min);
        if let Some(min) = young {
            return Some(format!("younger than {min} min"));
        }
        let quiet = crate::num::to_f64(doc.idle_min) >= idle_limit;
        (request.idle_only && !quiet).then(|| format!("not idle for {idle_limit} min"))
    }

    /// Every killable, unprotected process of a rule; `not_found` for an unknown rule.
    pub(crate) fn kill_rule(
        &self,
        request: &KillRuleRequest,
    ) -> Result<KillRuleOutcome, ProcsError> {
        let mut engine = locked(&self.engine);
        self.kill_rule_locked(&mut engine, request)
    }

    fn kill_rule_locked(
        &self,
        engine: &mut Engine,
        request: &KillRuleRequest,
    ) -> Result<KillRuleOutcome, ProcsError> {
        let rules = self.rules_snapshot();
        let rule = rules
            .rules
            .iter()
            .find(|r| r.name == request.rule)
            .ok_or_else(|| ProcsError::not_found(format!("no rule \"{}\"", request.rule)))?;
        self.sample_once(engine);
        let idle_limit = rule.idle_min.unwrap_or(DEFAULT_IDLE_LIMIT);
        let docs: Vec<ProcDoc> = locked(&self.view)
            .listing
            .iter()
            .filter(|d| d.rule == request.rule)
            .cloned()
            .collect();
        let mut out = KillRuleOutcome {
            rule: request.rule.clone(),
            killed: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
        };
        for doc in &docs {
            if let Some(why) = self.skip_reason(doc, request, idle_limit) {
                out.skipped.push(Skipped {
                    pid: doc.pid,
                    name: doc.name.clone(),
                    why,
                });
                continue;
            }
            let outcome = self.kill_doc(doc, false, request.reason.as_deref(), &request.source);
            if outcome.killed {
                out.killed.push(outcome);
            } else {
                out.failed.push(outcome);
            }
        }
        self.finish_rule(engine, request, &out);
        Ok(out)
    }

    fn finish_rule(&self, engine: &mut Engine, request: &KillRuleRequest, out: &KillRuleOutcome) {
        if out.killed.is_empty() && out.failed.is_empty() {
            self.note(
                Level::Info,
                format!(
                    "kill-rule {} {}: nothing to kill ({} skipped)",
                    request.rule,
                    request.source.text(),
                    out.skipped.len()
                ),
            );
        } else {
            self.sample_once(engine);
        }
    }
}
