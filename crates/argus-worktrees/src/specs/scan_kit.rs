//! Helpers shared by the scan specs: the facts by folder name, the scan environment, the rules as lines.

use crate::fact::Fact;
use crate::fs::RealFs;
use crate::git::Git;
use crate::paths::base_name;
use crate::scan::{RuleInfo, ScanEnv};
use std::collections::HashMap;

pub fn by_name(facts: &[Fact]) -> HashMap<String, &Fact> {
    facts.iter().map(|f| (base_name(&f.path), f)).collect()
}

pub fn env<'a>(
    git: &'a Git,
    ci: bool,
    measure: &'a dyn Fn(&str) -> crate::measure::Leftover,
) -> ScanEnv<'a> {
    ScanEnv {
        git,
        fs: &RealFs,
        case_insensitive: ci,
        measure,
        known_repos: &[],
    }
}

pub fn rule_lines(rules: &[RuleInfo]) -> Vec<String> {
    let line = |r: &RuleInfo| {
        format!(
            "{} {} {} {} {:?}",
            r.id,
            r.exists,
            r.matched,
            r.candidates,
            r.error.as_deref()
        )
    };
    rules.iter().map(line).collect()
}
