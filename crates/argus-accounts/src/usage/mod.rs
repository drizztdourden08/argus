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
        if !status.is_success() {
            return Err(format!("the usage endpoint answered {status}"));
        }
        let body = response.text().map_err(|e| e.without_url().to_string())?;
        parse::parse_usage(&body)
    }
}
