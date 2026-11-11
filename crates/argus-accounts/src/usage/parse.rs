//! Reads the usage endpoint's JSON into [`Usage`]: `limits[]` first, with unknown kinds kept,
//! and the legacy `five_hour` / `seven_day` windows when `limits[]` is absent. Nulls are fine
//! everywhere.

use serde_json::Value;

use crate::types::{Severity, Usage, UsageLimit};

/// The severity a bare percent stands for, when the endpoint gave none.
#[must_use]
pub fn severity_for(percent: f64) -> Severity {
    if percent >= 100.0 {
        Severity::Critical
    } else if percent >= 80.0 {
        Severity::Warning
    } else {
        Severity::Normal
    }
}

/// The usage in the endpoint's answer. `read_at` and `error` stay empty for the caller.
///
/// # Errors
/// When the text is not a JSON object.
pub fn parse_usage(text: &str) -> Result<Usage, String> {
    let value: Value =
        serde_json::from_str(text).map_err(|e| format!("usage answer is not JSON: {e}"))?;
    if !value.is_object() {
        return Err("usage answer is not an object".to_owned());
    }
    Ok(Usage {
        limits: limits_of(&value),
        read_at: None,
        error: None,
    })
}

fn limits_of(value: &Value) -> Vec<UsageLimit> {
    match value.get("limits").and_then(Value::as_array) {
        Some(list) if !list.is_empty() => list.iter().filter_map(limit_of).collect(),
        _ => legacy_limits(value),
    }
}

fn limit_of(item: &Value) -> Option<UsageLimit> {
    let percent = number(item, "percent").or_else(|| number(item, "utilization"))?;
    Some(UsageLimit {
        kind: text(item, "kind").unwrap_or_else(|| "unknown".to_owned()),
        group: text(item, "group"),
        percent,
        severity: severity_of(item).unwrap_or_else(|| severity_for(percent)),
        resets_at: text(item, "resets_at"),
        model: item
            .pointer("/scope/model/display_name")
            .and_then(Value::as_str)
            .map(str::to_owned),
        is_active: item
            .get("is_active")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn legacy_limits(value: &Value) -> Vec<UsageLimit> {
    [("five_hour", "five_hour"), ("seven_day", "weekly_all")]
        .into_iter()
        .filter_map(|(field, kind)| {
            let window = value.get(field)?;
            let percent = number(window, "utilization")?;
            Some(UsageLimit {
                kind: kind.to_owned(),
                group: None,
                percent,
                severity: severity_for(percent),
                resets_at: text(window, "resets_at"),
                model: None,
                is_active: false,
            })
        })
        .collect()
}

fn severity_of(item: &Value) -> Option<Severity> {
    serde_json::from_value(item.get("severity")?.clone()).ok()
}

fn number(item: &Value, key: &str) -> Option<f64> {
    item.get(key).and_then(Value::as_f64)
}

fn text(item: &Value, key: &str) -> Option<String> {
    item.get(key).and_then(Value::as_str).map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str = r#"{
        "five_hour": {"utilization": 12.0, "resets_at": "2026-11-11T20:00:00Z"},
        "seven_day": {"utilization": 43.0, "resets_at": "2026-11-14T10:00:00Z"},
        "limits": [
            {"kind":"five_hour","group":"session","percent":12,"severity":"normal",
             "resets_at":"2026-11-11T20:00:00Z","scope":null,"is_active":false},
            {"kind":"weekly_all","group":"weekly","percent":43,"severity":"warning",
             "resets_at":"2026-11-14T10:00:00Z","scope":null,"is_active":true},
            {"kind":"weekly_scoped","group":"weekly","percent":9,"severity":"normal",
             "resets_at":null,"scope":{"model":{"display_name":"Opus"}},"is_active":false},
            {"kind":"brand_new_kind","percent":1}
        ]
    }"#;

    #[test]
    fn limits_come_first_and_keep_unknown_kinds() {
        let usage = parse_usage(FULL).unwrap();
        let kinds: Vec<_> = usage.limits.iter().map(|l| l.kind.as_str()).collect();
        assert_eq!(
            kinds,
            ["five_hour", "weekly_all", "weekly_scoped", "brand_new_kind"]
        );
        assert_eq!(usage.limits[1].severity, Severity::Warning);
        assert!(usage.limits[1].is_active);
        assert_eq!(usage.limits[2].model.as_deref(), Some("Opus"));
        assert_eq!(usage.limits[2].resets_at, None);
        assert_eq!(usage.limits[3].severity, Severity::Normal);
    }

    #[test]
    fn a_critical_limit_at_100_is_kept() {
        let text = r#"{"limits":[{"kind":"weekly_all","group":"weekly","percent":100,
            "severity":"critical","resets_at":"2026-11-14T10:00:00Z","is_active":true}]}"#;
        let usage = parse_usage(text).unwrap();
        assert_eq!(usage.limits.len(), 1);
        assert!((usage.limits[0].percent - 100.0).abs() < f64::EPSILON);
        assert_eq!(usage.limits[0].severity, Severity::Critical);
    }

    #[test]
    fn null_windows_give_no_limits() {
        let usage = parse_usage(r#"{"five_hour":null,"seven_day":null}"#).unwrap();
        assert!(usage.limits.is_empty());
        let usage = parse_usage(r#"{"five_hour":{"utilization":null,"resets_at":null}}"#).unwrap();
        assert!(usage.limits.is_empty());
    }

    #[test]
    fn legacy_windows_are_the_fallback() {
        let text = r#"{"five_hour":{"utilization":85.5,"resets_at":"2026-11-11T20:00:00Z"},
                       "seven_day":null,"limits":null}"#;
        let usage = parse_usage(text).unwrap();
        assert_eq!(usage.limits.len(), 1);
        assert_eq!(usage.limits[0].kind, "five_hour");
        assert_eq!(usage.limits[0].severity, Severity::Warning);
        assert_eq!(
            usage.limits[0].resets_at.as_deref(),
            Some("2026-11-11T20:00:00Z")
        );
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(parse_usage("nope").is_err());
        assert!(parse_usage("[1]").is_err());
    }
}
