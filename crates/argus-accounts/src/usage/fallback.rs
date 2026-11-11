//! The Desktop app's own usage log, `plan-usage-history.json`, as the answer when the live read
//! fails: the newest sample of the account's organization.

use std::fs;
use std::path::Path;

use argus_core::clock::iso_from_ms;
use serde_json::Value;

use super::parse::severity_for;
use crate::types::{Usage, UsageLimit};

/// The newest sample of `org` in `<app>/plan-usage-history.json`, as a usage.
#[must_use]
pub fn latest_sample(app: &Path, org: &str) -> Option<Usage> {
    let text = fs::read_to_string(app.join("plan-usage-history.json")).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    let newest = value
        .get("samples")?
        .as_array()?
        .iter()
        .filter(|s| s.get("org").and_then(Value::as_str) == Some(org))
        .max_by_key(|s| s.get("t").and_then(Value::as_i64).unwrap_or(0))?;
    let limits = [("fh", "five_hour"), ("sd", "weekly_all")]
        .into_iter()
        .filter_map(|(field, kind)| {
            let percent = newest.pointer(&format!("/u/{field}"))?.as_f64()?;
            Some(UsageLimit {
                kind: kind.to_owned(),
                group: None,
                percent,
                severity: severity_for(percent),
                resets_at: None,
                model: None,
                is_active: false,
            })
        })
        .collect();
    Some(Usage {
        limits,
        read_at: newest.get("t").and_then(Value::as_i64).map(iso_from_ms),
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_the_newest_sample_of_the_org() {
        let dir = tempfile::tempdir().unwrap();
        let log = r#"{"version":2,"samples":[
            {"t":100,"org":"o1","u":{"fh":10,"sd":20}},
            {"t":300,"org":"o2","u":{"fh":99,"sd":99}},
            {"t":200,"org":"o1","u":{"sd":30}}]}"#;
        fs::write(dir.path().join("plan-usage-history.json"), log).unwrap();
        let usage = latest_sample(dir.path(), "o1").unwrap();
        assert_eq!(usage.limits.len(), 1);
        assert_eq!(usage.limits[0].kind, "weekly_all");
        assert!((usage.limits[0].percent - 30.0).abs() < f64::EPSILON);
        assert!(latest_sample(dir.path(), "none").is_none());
    }
}
