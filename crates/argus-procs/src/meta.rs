//! The summary of a sample: doc `meta/procs`.

use crate::clock::iso;
use crate::evaluate::Evaluation;
use crate::rule::Rule;
use crate::types::{Meta, RuleSummary};

/// What the summary needs besides the evaluation.
pub struct MetaParts<'a> {
    pub rules: &'a [Rule],
    pub rules_file: &'a str,
    pub rules_error: Option<&'a str>,
    pub sample_ms: u64,
    pub sampled_at: i64,
    pub platform: &'a str,
}

fn by_rule(ev: &Evaluation, rules: &[Rule]) -> Vec<RuleSummary> {
    rules
        .iter()
        .filter_map(|rule| {
            let group = ev.groups.get(&rule.name)?;
            Some(RuleSummary {
                rule: rule.name.clone(),
                count: ev.docs.iter().filter(|d| d.rule == rule.name).count(),
                matched: group.count,
                mem_mb: group.mem_mb,
                max_group_mem_mb: rule.max_group_mem_mb,
                over: ev.over.contains(&rule.name),
            })
        })
        .collect()
}

/// The meta doc of a successful sample, without `tookMs`.
#[must_use]
pub fn build_meta(ev: &Evaluation, parts: &MetaParts<'_>) -> Meta {
    Meta {
        id: "procs".to_string(),
        sampled_at: Some(iso(parts.sampled_at)),
        sample_ms: parts.sample_ms,
        count: ev.docs.len(),
        total_mem_mb: ev.docs.iter().map(|d| d.mem_mb).sum(),
        by_rule: by_rule(ev, parts.rules),
        rules: parts.rules.to_vec(),
        rules_file: parts.rules_file.to_string(),
        rules_error: parts.rules_error.map(ToString::to_string),
        platform: parts.platform.to_string(),
        error: None,
        took_ms: None,
    }
}

/// The meta doc after a failed sample: the previous one (or an empty one) with the new error.
#[must_use]
pub fn failed_meta(previous: Option<&Meta>, parts: &MetaParts<'_>, error: &str) -> Meta {
    let base = previous.cloned().unwrap_or_else(|| Meta {
        id: "procs".to_string(),
        sampled_at: None,
        sample_ms: parts.sample_ms,
        count: 0,
        total_mem_mb: 0,
        by_rule: Vec::new(),
        rules: Vec::new(),
        rules_file: String::new(),
        rules_error: None,
        platform: parts.platform.to_string(),
        error: None,
        took_ms: None,
    });
    Meta {
        rules: parts.rules.to_vec(),
        rules_file: parts.rules_file.to_string(),
        rules_error: parts.rules_error.map(ToString::to_string),
        error: Some(error.chars().take(300).collect()),
        ..base
    }
}
