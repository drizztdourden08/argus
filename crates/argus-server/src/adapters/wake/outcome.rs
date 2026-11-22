//! How a headless resume ended, read from the CLI's one JSON result on stdout.

use serde_json::Value;

/// The end of one headless resume.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The CLI reported success.
    Ok,
    /// The CLI could not sign in (status 401 or an authentication error).
    SignIn,
    /// The CLI failed for another reason, in a few words.
    Failed(String),
    /// The CLI ran too long and was killed.
    TimedOut,
}

/// The outcome the CLI's stdout describes. Output that is not the result object counts as a failure.
#[must_use]
pub fn parse(stdout: &str) -> Outcome {
    let Ok(result) = serde_json::from_str::<Value>(stdout.trim()) else {
        return Outcome::Failed("no readable result".to_owned());
    };
    if result.get("is_error").and_then(Value::as_bool) != Some(true) {
        return Outcome::Ok;
    }
    let status = result.get("api_error_status").and_then(Value::as_i64);
    let kind = result.get("api_error_type").and_then(Value::as_str);
    if status == Some(401) || kind == Some("authentication_error") {
        return Outcome::SignIn;
    }
    Outcome::Failed(reason(&result, status, kind))
}

fn reason(result: &Value, status: Option<i64>, kind: Option<&str>) -> String {
    let mut parts = Vec::new();
    if let Some(subtype) = result.get("subtype").and_then(Value::as_str) {
        parts.push(subtype.to_owned());
    }
    if let Some(kind) = kind {
        parts.push(kind.to_owned());
    }
    if let Some(status) = status {
        parts.push(format!("status {status}"));
    }
    let text: String = parts.join(", ").chars().take(80).collect();
    if text.is_empty() {
        "error".to_owned()
    } else {
        text
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_result_without_an_error_is_ok() {
        let out = parse(r#"{"type":"result","subtype":"success","is_error":false}"#);
        assert_eq!(out, Outcome::Ok);
    }

    #[test]
    fn a_401_or_an_authentication_error_is_a_sign_in_failure() {
        let by_status = parse(r#"{"is_error":true,"api_error_status":401}"#);
        assert_eq!(by_status, Outcome::SignIn);
        let by_type = parse(r#"{"is_error":true,"api_error_type":"authentication_error"}"#);
        assert_eq!(by_type, Outcome::SignIn);
    }

    #[test]
    fn another_error_keeps_a_short_reason() {
        let out =
            parse(r#"{"is_error":true,"subtype":"error_during_execution","api_error_status":529}"#);
        assert_eq!(
            out,
            Outcome::Failed("error_during_execution, status 529".to_owned())
        );
        assert_eq!(
            parse(r#"{"is_error":true}"#),
            Outcome::Failed("error".to_owned())
        );
    }

    #[test]
    fn output_that_is_not_json_is_a_failure() {
        assert_eq!(
            parse("not json"),
            Outcome::Failed("no readable result".to_owned())
        );
        assert_eq!(parse(""), Outcome::Failed("no readable result".to_owned()));
    }
}
