//! What one server gave for one probe: a JSON answer, or a refusal (an HTTP error, or no answer
//! at all, which is status 0).
// test-tag: critical

use serde_json::Value;

/// One side's answer to a probe.
#[derive(Debug, Clone, PartialEq)]
pub enum Got {
    /// A 2xx answer and its body.
    Doc(Value),
    /// An error answer, or none.
    Refused {
        /// The HTTP status, 0 when the server could not be reached.
        status: u16,
        /// The error code of the body, or the reason there was no answer.
        code: String,
    },
}

impl Got {
    /// The answer of an HTTP exchange: a body for a 2xx status, a refusal for any other.
    #[must_use]
    pub fn from_answer(status: u16, body: Value) -> Got {
        if (200..300).contains(&status) {
            return Got::Doc(body);
        }
        let code = body["error"]["code"].as_str().unwrap_or_default();
        Got::Refused {
            status,
            code: code.to_owned(),
        }
    }

    /// No answer, for this reason.
    #[must_use]
    pub fn none(why: impl Into<String>) -> Got {
        Got::Refused {
            status: 0,
            code: why.into(),
        }
    }

    /// A few words for a report.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Got::Doc(_) => "answered".to_owned(),
            Got::Refused { status: 0, code } => format!("no answer ({code})"),
            Got::Refused { status, code } if code.is_empty() => format!("HTTP {status}"),
            Got::Refused { status, code } => format!("HTTP {status} {code}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Got;

    #[test]
    fn a_2xx_status_is_a_doc_and_the_rest_a_refusal_with_the_error_code() {
        assert_eq!(
            Got::from_answer(200, json!({ "a": 1 })),
            Got::Doc(json!({ "a": 1 }))
        );
        let body = json!({ "error": { "code": "not_found", "message": "x" } });
        let got = Got::from_answer(404, body);
        assert_eq!(got.describe(), "HTTP 404 not_found");
    }
}
