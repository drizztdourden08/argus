//! Usage: the endpoint reader, its JSON parser, the Desktop log fallback and the poll pass.

pub mod fallback;
pub mod parse;
pub mod poller;

use std::time::Duration;

use reqwest::StatusCode;
use reqwest::blocking::Client;

use crate::ports::UsageSource;
use crate::types::Usage;

const ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";
const BETA: &str = "oauth-2025-04-20";
const TIMEOUT: Duration = Duration::from_secs(20);

/// The prefix of the error a 401 gives, so the poller refreshes the token.
pub const UNAUTHORIZED: &str = "unauthorized";

/// The prefix of the error a 429 gives, so the poller backs the account off.
pub const RATE_LIMITED: &str = "rate limited";

/// The error of a 429, with the seconds the `Retry-After` header asked for when it gave any.
#[must_use]
pub fn rate_limited(retry_after: Option<u64>) -> String {
    match retry_after {
        Some(secs) => format!("{RATE_LIMITED}; retry after {secs}s"),
        None => RATE_LIMITED.to_owned(),
    }
}

/// Whether `error` is a 429; the inner answer is the seconds `Retry-After` asked for, if any.
#[must_use]
pub fn rate_limit_of(error: &str) -> Option<Option<u64>> {
    let rest = error.strip_prefix(RATE_LIMITED)?;
    let secs = rest
        .strip_prefix("; retry after ")
        .and_then(|t| t.strip_suffix('s'))
        .and_then(|t| t.parse().ok());
    Some(secs)
}

/// The real usage endpoint.
pub struct HttpUsage;

impl UsageSource for HttpUsage {
    fn fetch(&self, access_token: &str) -> Result<Usage, String> {
        let client = Client::builder()
            .timeout(TIMEOUT)
            .user_agent("argus")
            .build()
            .map_err(|e| e.without_url().to_string())?;
        let response = client
            .get(ENDPOINT)
            .bearer_auth(access_token)
            .header("anthropic-beta", BETA)
            .send()
            .map_err(|e| e.without_url().to_string())?;
        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(format!(
                "{UNAUTHORIZED}: the usage endpoint refused the token"
            ));
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            let asked = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.trim().parse().ok());
            return Err(rate_limited(asked));
        }
        if !status.is_success() {
            return Err(format!("the usage endpoint answered {status}"));
        }
        let body = response.text().map_err(|e| e.without_url().to_string())?;
        parse::parse_usage(&body)
    }
}
