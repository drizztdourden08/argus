//! The Argus process monitor (DESIGN.md 13.10): samples the machine's processes, lists the ones the
//! Overlord's rules take, wakes the Overlord about the bad ones and kills on request.
//!
//! The HTTP routes are not here. The server mounts a [`ProcsService`] built from [`Ports`]: a
//! [`ProcsStore`] keeps the docs, a [`ProcsHost`] reaches the wake engine, activity log, projects and tray.

mod alert_text;
mod alerts;
mod clock;
mod config;
mod defaults;
mod error;
mod evaluate;
mod guard;
mod hint;
mod killing;
mod matcher;
mod meta;
mod normalize;
mod num;
mod ports;
mod project;
mod publish;
mod reaper;
mod responder;
mod rule;
mod rulebook;
mod rules_file;
mod sampling;
mod scope;
mod self_kill;
mod service;
mod shell;
mod system_killer;
mod system_sampler;
mod text;
mod timer;
mod tracker;
mod tree;
mod types;

pub use alert_text::{ALERT_TAIL, alert_text};
pub use alerts::{AlertBook, Due, Pace};
pub use argus_core::clock::age_text;
pub use config::ProcsConfig;
pub use defaults::{ARGUS_CMD, PROTECTED_CMD, PROTECTED_NAMES, default_rules};
pub use error::{ErrorCode, ProcsError};
pub use evaluate::{EvalInput, Evaluation, Group, evaluate};
pub use guard::GuardCtx;
pub use hint::hint_of;
pub use killing::{KillOutcome, KillRequest, KillRuleOutcome, KillRuleRequest, Skipped, Source};
pub use matcher::{CompiledRule, compile_rules, match_rule};
pub use normalize::normalize_rule;
pub use ports::{
    Activity, KillResult, Killer, Level, ProcsHost, ProcsStore, Responder, Sampler, WakeRequest,
};
pub use project::{ProjectFolders, ProjectRoot, normalize_roots};
pub use publish::HEARTBEAT_MS;
pub use reaper::{REAP_KIND, REAP_REASON, ReaperConfig};
pub use responder::{NoResponder, PowerShellResponder};
pub use rule::{Match, Rule};
pub use rulebook::{LoadWarning, RuleBook, RuleChange};
pub use rules_file::{RULES_FILE, parse_rules_file, rules_file_text};
pub use scope::{Scope, refusal};
pub use self_kill::{SELF_KILL_KIND, SelfKillOutcome, SelfKillRequest};
pub use service::{Options, Ports, ProcsService, RulesView};
pub use system_killer::SystemKiller;
pub use system_sampler::{SystemSampler, platform_name};
pub use text::{base_name, mem_text, slashed};
pub use tracker::{IDLE_CPU_PCT, Tracked, Tracker};
pub use tree::{chain_of, descendants_of};
pub use types::{Flag, Guard, Meta, ProcDoc, ProcRow, RuleSummary, Sample, Status};
