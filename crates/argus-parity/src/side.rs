//! One server of the comparison: its base URL, its bearer when it wants one, and the two things
//! the comparison asks of it, a probe answered and a stream opened. A client of the server like
//! any other, and read-only: a probe is a GET or one of the two MCP reads.
// test-tag: critical

use std::time::Duration;

use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, RequestBuilder, Response};
use serde_json::Value;

use crate::got::Got;
use crate::probe::Request;

const ANSWER_LIMIT: Duration = Duration::from_secs(20);

/// A dashboard server.
#[derive(Debug, Clone)]
pub struct Side {
    base: String,
    token: Option<String>,
    http: Client,
}

/// The JSON of a body that came as a plain answer or as one server-sent frame.
fn parse_body(text: &str, event_stream: bool) -> Value {
    let payload = if event_stream {
        text.lines()
            .rev()
            .find_map(|l| l.strip_prefix("data:"))
            .map_or(text, str::trim)
    } else {
        text
    };
    serde_json::from_str(payload).unwrap_or_else(|_| Value::String(text.to_owned()))
}

async fn answer(res: Response) -> Got {
    let status = res.status().as_u16();
    let event_stream = res
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|t| t.contains("text/event-stream"));
    match res.text().await {
        Ok(text) => Got::from_answer(status, parse_body(&text, event_stream)),
        Err(e) => Got::none(format!("the body was cut: {e}")),
    }
}

impl Side {
    /// A server at `url` (`http://127.0.0.1:47615`), with the bearer when it needs one.
    #[must_use]
    pub fn new(url: &str, token: Option<String>) -> Side {
        let http = Client::builder()
            .no_proxy()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();
        Side {
            base: url.trim_end_matches('/').to_owned(),
            token,
            http,
        }
    }

    /// The base URL.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.base
    }

    fn authorised(&self, req: RequestBuilder) -> RequestBuilder {
        match &self.token {
            Some(t) => req.header(AUTHORIZATION, format!("Bearer {t}")),
            None => req,
        }
    }

    fn build(&self, request: &Request) -> RequestBuilder {
        match request {
            Request::Get(path) => self.http.get(format!("{}{path}", self.base)),
            Request::Mcp(call) => self
                .http
                .post(format!("{}/mcp", self.base))
                .header(CONTENT_TYPE, "application/json")
                .header(ACCEPT, "application/json, text/event-stream")
                .body(call.body().to_string()),
        }
    }

    /// Puts a probe to the server.
    pub async fn fetch(&self, request: &Request) -> Got {
        let sent = self.authorised(self.build(request)).timeout(ANSWER_LIMIT);
        match sent.send().await {
            Ok(res) => answer(res).await,
            Err(e) => Got::none(e.without_url().to_string()),
        }
    }

    /// Opens an event stream; the caller reads the body piece by piece.
    ///
    /// # Errors
    /// The server cannot be reached or answers anything but 200.
    pub async fn open_stream(&self, path: &str) -> Result<Response, String> {
        let req = self
            .http
            .get(format!("{}{path}", self.base))
            .header(ACCEPT, "text/event-stream");
        let res = self
            .authorised(req)
            .send()
            .await
            .map_err(|e| e.without_url().to_string())?;
        match res.status().as_u16() {
            200 => Ok(res),
            status => Err(format!("HTTP {status}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Side, parse_body};

    #[test]
    fn a_plain_body_is_json_and_a_frame_is_its_last_data_line() {
        assert_eq!(parse_body("{\"a\":1}", false), json!({ "a": 1 }));
        let frame = "event: message\ndata: {\"b\":2}\n\n";
        assert_eq!(parse_body(frame, true), json!({ "b": 2 }));
        assert_eq!(parse_body("<html>", false), json!("<html>"));
    }

    #[test]
    fn the_base_url_loses_its_trailing_slash() {
        assert_eq!(
            Side::new("http://127.0.0.1:1/", None).url(),
            "http://127.0.0.1:1"
        );
    }
}
