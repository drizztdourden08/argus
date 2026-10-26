//! Rules with their regexes compiled, and the first-match lookup.

use crate::rule::Rule;
use crate::text::slashed;
use crate::types::ProcRow;
use fancy_regex::Regex;

/// A rule and its compiled regexes.
#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub rule: Rule,
    name: Option<Regex>,
    cmd: Option<Regex>,
    path: Option<Regex>,
    ignore: Vec<Regex>,
}

fn build(src: &str) -> Result<Regex, fancy_regex::Error> {
    Regex::new(&format!("(?i){src}"))
}

fn optional(src: Option<&String>) -> Result<Option<Regex>, fancy_regex::Error> {
    src.map(|text| build(text)).transpose()
}

fn compile_one(rule: &Rule) -> Result<CompiledRule, fancy_regex::Error> {
    let ignore = rule
        .ignore
        .iter()
        .flatten()
        .map(|src| build(src))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CompiledRule {
        rule: rule.clone(),
        name: optional(rule.matcher.name.as_ref())?,
        cmd: optional(rule.matcher.cmd.as_ref())?,
        path: optional(rule.matcher.path.as_ref())?,
        ignore,
    })
}

/// The rules with their regexes compiled, in order; a rule whose regex does not compile is left out.
#[must_use]
pub fn compile_rules(rules: &[Rule]) -> Vec<CompiledRule> {
    rules.iter().filter_map(|r| compile_one(r).ok()).collect()
}

fn hits(regex: Option<&Regex>, text: &str) -> bool {
    regex.is_none_or(|re| re.is_match(text).unwrap_or(false))
}

impl CompiledRule {
    fn takes(&self, proc: &ProcRow, cmd: &str, path: &str) -> bool {
        hits(self.name.as_ref(), &proc.name)
            && hits(self.cmd.as_ref(), cmd)
            && hits(self.path.as_ref(), path)
            && !self
                .ignore
                .iter()
                .any(|re| re.is_match(cmd).unwrap_or(false))
    }
}

/// The first rule that takes the process, or `None`.
#[must_use]
pub fn match_rule<'a>(compiled: &'a [CompiledRule], proc: &ProcRow) -> Option<&'a Rule> {
    let cmd = slashed(&proc.cmd);
    let path = slashed(proc.path.as_deref().unwrap_or(""));
    compiled
        .iter()
        .find(|c| c.takes(proc, &cmd, &path))
        .map(|c| &c.rule)
}
