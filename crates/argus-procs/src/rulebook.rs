//! The rules in memory and on disk: loaded when the file changes, seeded when missing, edited by name.

use crate::defaults::default_rules;
use crate::error::ProcsError;
use crate::matcher::{CompiledRule, compile_rules};
use crate::normalize::normalize_rule;
use crate::rule::Rule;
use crate::rules_file::{parse_rules_file, rules_file_text};
use serde_json::{Map, Value};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// A load that kept the previous rules; the caller logs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadWarning {
    pub message: &'static str,
    pub detail: String,
}

/// The outcome of `set_rule`.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleChange {
    pub rule: Rule,
    pub created: bool,
}

/// The rules the monitor runs, with the file they live in.
#[derive(Debug)]
pub struct RuleBook {
    file: PathBuf,
    rules: Vec<Rule>,
    compiled: Vec<CompiledRule>,
    error: Option<String>,
    stamp: Option<String>,
}

fn stamp_of(file: &Path) -> Option<String> {
    let meta = fs::metadata(file).ok()?;
    let ms = meta
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis();
    Some(format!("{ms}:{}", meta.len()))
}

fn unset(value: &Value) -> bool {
    value.is_null() || value.as_str() == Some("")
}

fn patch(target: &mut Map<String, Value>, changes: &Map<String, Value>, skip: Option<&str>) {
    for (key, value) in changes.iter().filter(|(k, _)| Some(k.as_str()) != skip) {
        if unset(value) {
            target.remove(key);
        } else {
            target.insert(key.clone(), value.clone());
        }
    }
}

fn merged(old: &Rule, input: &Value) -> Value {
    let mut base = match serde_json::to_value(old) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    };
    let Some(changes) = input.as_object() else {
        return Value::Object(base);
    };
    patch(&mut base, changes, Some("match"));
    match changes.get("match") {
        Some(Value::Object(m)) => {
            let mut matcher = base
                .get("match")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            patch(&mut matcher, m, None);
            base.insert("match".into(), Value::Object(matcher));
        }
        Some(other) if !unset(other) => {
            base.insert("match".into(), other.clone());
        }
        _ => {}
    }
    Value::Object(base)
}

impl RuleBook {
    /// A rule book over this file; nothing is read until `load`.
    #[must_use]
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            rules: Vec::new(),
            compiled: Vec::new(),
            error: None,
            stamp: None,
        }
    }

    /// The rules file.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// The rules in force.
    #[must_use]
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// The rules in force, compiled.
    #[must_use]
    pub fn compiled(&self) -> &[CompiledRule] {
        &self.compiled
    }

    /// Why the file could not be used at the last load, if it could not.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    fn adopt(&mut self, list: Vec<Rule>) {
        self.compiled = compile_rules(&list);
        self.rules = list;
    }

    fn ensure_rules(&mut self) {
        if self.rules.is_empty() {
            self.adopt(default_rules());
        }
    }

    fn write(&mut self, list: Vec<Rule>) -> io::Result<()> {
        if let Some(dir) = self.file.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut tmp = self.file.clone().into_os_string();
        tmp.push(format!(".{}.tmp", std::process::id()));
        let tmp = PathBuf::from(tmp);
        fs::write(&tmp, rules_file_text(&list))?;
        fs::rename(&tmp, &self.file)?;
        self.stamp = stamp_of(&self.file);
        self.error = None;
        self.adopt(list);
        Ok(())
    }

    fn read_changed(&mut self) -> io::Result<Option<LoadWarning>> {
        if !self.file.exists() {
            self.write(default_rules())?;
            return Ok(None);
        }
        let stamp = stamp_of(&self.file);
        if stamp == self.stamp && !self.rules.is_empty() {
            return Ok(None);
        }
        let text = fs::read_to_string(&self.file)?;
        self.stamp = stamp;
        match parse_rules_file(&text) {
            Ok(list) => {
                self.error = None;
                self.adopt(list);
                Ok(None)
            }
            Err(detail) => {
                self.error = Some(detail.clone());
                self.ensure_rules();
                Ok(Some(LoadWarning {
                    message: "process rules not loaded, the previous ones stay",
                    detail,
                }))
            }
        }
    }

    /// Reads the file again when it changed (or the first time); seeds the defaults when there is none.
    pub fn load(&mut self) -> Option<LoadWarning> {
        self.read_changed().unwrap_or_else(|err| {
            self.error = Some(err.to_string());
            self.ensure_rules();
            Some(LoadWarning {
                message: "process rules not loaded",
                detail: err.to_string(),
            })
        })
    }

    /// Creates or changes one rule; `null` (or `""`) for a field removes it. A new rule goes before the
    /// first rule with an empty match, or last.
    ///
    /// # Errors
    /// A `bad_request` when the merged rule is not valid, `internal` when the file cannot be written.
    pub fn set_rule(&mut self, input: &Value) -> Result<RuleChange, ProcsError> {
        let name = input.get("name").and_then(Value::as_str).unwrap_or("");
        let at = self.rules.iter().position(|r| r.name == name);
        let old = at.map_or_else(
            || Rule {
                name: name.to_string(),
                ..Rule::default()
            },
            |i| self.rules[i].clone(),
        );
        let rule = normalize_rule(&merged(&old, input))?;
        let mut next = self.rules.clone();
        if let Some(i) = at {
            next[i] = rule.clone();
        } else {
            let before = next.iter().position(|r| r.matcher.is_empty());
            next.insert(before.unwrap_or(next.len()), rule.clone());
        }
        self.write(next)
            .map_err(|e| ProcsError::internal(e.to_string()))?;
        Ok(RuleChange {
            rule,
            created: at.is_none(),
        })
    }

    /// Deletes a rule by name; `false` when there is none of that name.
    ///
    /// # Errors
    /// `internal` when the file cannot be written.
    pub fn remove_rule(&mut self, name: &str) -> Result<bool, ProcsError> {
        let next: Vec<Rule> = self
            .rules
            .iter()
            .filter(|r| r.name != name)
            .cloned()
            .collect();
        if next.len() == self.rules.len() {
            return Ok(false);
        }
        self.write(next)
            .map_err(|e| ProcsError::internal(e.to_string()))?;
        Ok(true)
    }
}
