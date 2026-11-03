//! The blocking HTTP client of the CLI (`ureq`, no TLS, no async runtime to start): JSON in, JSON
//! out, the bearer token only when `config.json` asks for it, and the error the Node client
//! throws, with the `dashboard_down` case apart so a verb can pick its exit code.

use std::fmt;
use std::io::Read;
use std::time::Duration;

use serde_json::Value;
use ureq::http::Request;

use super::config::DashboardConfig;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// The sentence of a server that is not running.
pub const DOWN_MESSAGE: &str = "the dashboard is not running (dashboard start)";

/// Why a call to the server failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    /// The server did not answer; the detail names the cause (`ECONNREFUSED` ...). The Node
    /// client gives this error the exit code 3.
    Down(Option<String>),
    /// The server accepted the call but did not answer within the time limit (seconds): it is
    /// running, and the work may still be going on there. Never "not running".
    Timeout(u64),
    /// The server answered with an error status.
    Http {
        /// The status code.
        status: u16,
        /// The `error.code` of the body, else `http_error`.
        code: String,
        /// The `error.message` of the body, else `<status> <reason>`.
        message: String,
    },
}

impl ClientError {
    /// The error of a dashboard that is down, with an optional cause.
    #[must_use]
    pub fn down(detail: Option<&str>) -> Self {
        ClientError::Down(detail.map(str::to_owned))
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Down(None) => f.write_str(DOWN_MESSAGE),
            ClientError::Down(Some(detail)) => write!(f, "{DOWN_MESSAGE}: {detail}"),
            ClientError::Timeout(secs) => write!(
                f,
                "the dashboard did not answer in time ({secs} s): the call may still be running there"
            ),
            ClientError::Http { message, .. } => f.write_str(message),
        }
    }
}

impl std::error::Error for ClientError {}

/// A client of the running dashboard.
#[derive(Debug, Clone)]
pub struct Client {
    /// `http://127.0.0.1:<port>`.
    pub base: String,
    token: Option<String>,
}

fn cause(e: &ureq::Error) -> String {
    match e {
        ureq::Error::ConnectionFailed => "ECONNREFUSED".to_owned(),
        ureq::Error::Io(io) if io.kind() == std::io::ErrorKind::ConnectionRefused => {
            "ECONNREFUSED".to_owned()
        }
        other => other.to_string(),
    }
}

/// A call that ran past its limit is a timeout; every other failure to reach the server is down.
fn failed(e: &ureq::Error, timeout: Option<Duration>) -> ClientError {
    match e {
        ureq::Error::Timeout(_) => ClientError::Timeout(timeout.map_or(0, |t| t.as_secs())),
        other => ClientError::down(Some(&cause(other))),
    }
}

fn agent(timeout: Option<Duration>) -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .proxy(None)
        .timeout_global(timeout)
        .build()
        .into()
}

fn read_text(response: ureq::http::Response<ureq::Body>) -> String {
    let mut text = String::new();
    response
        .into_body()
        .into_reader()
        .read_to_string(&mut text)
        .ok();
    text
}

fn http_error(status: u16, reason: &str, text: &str) -> ClientError {
    let body: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    let error = &body["error"];
    ClientError::Http {
        status,
        code: error["code"].as_str().unwrap_or("http_error").to_owned(),
        message: error["message"]
            .as_str()
            .map_or_else(|| format!("{status} {reason}"), str::to_owned),
    }
}

impl Client {
    /// A client of the server `config` describes.
    #[must_use]
    pub fn new(config: &DashboardConfig) -> Self {
        Client {
            base: format!("http://127.0.0.1:{}", config.port),
            token: config.token.clone(),
        }
    }

    fn builder(&self, method: &str, path: &str, json: bool) -> ureq::http::request::Builder {
        let mut builder = Request::builder()
            .method(method)
            .uri(format!("{}{path}", self.base));
        if let Some(token) = &self.token {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }
        if json {
            builder = builder.header("content-type", "application/json");
        }
        builder
    }

    fn send(
        &self,
        timeout: Option<Duration>,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<ureq::http::Response<ureq::Body>, ClientError> {
        let json = body.is_some() || method != "GET" && method != "HEAD";
        let builder = self.builder(method, path, json);
        let agent = agent(timeout);
        let sent = match body {
            Some(value) => agent.run(
                builder
                    .body(value.to_string())
                    .map_err(|e| ClientError::down(Some(&e.to_string())))?,
            ),
            None => agent.run(
                builder
                    .body(())
                    .map_err(|e| ClientError::down(Some(&e.to_string())))?,
            ),
        };
        sent.map_err(|e| failed(&e, timeout))
    }

    /// One call: the JSON of the answer, `Null` for an empty or unparseable body. A plain
    /// object body is sent as JSON; a body-less write still says it is JSON.
    ///
    /// # Errors
    /// [`ClientError::Down`] when the server does not answer, [`ClientError::Http`] for an
    /// error status.
    pub fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value, ClientError> {
        self.request_within(REQUEST_TIMEOUT, method, path, body)
    }

    /// [`Client::request`] with its own time limit.
    ///
    /// # Errors
    /// As [`Client::request`].
    pub fn request_within(
        &self,
        timeout: Duration,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value, ClientError> {
        let response = self.send(Some(timeout), method, path, body)?;
        let status = response.status();
        let reason = status.canonical_reason().unwrap_or_default().to_owned();
        let text = read_text(response);
        if !status.is_success() {
            return Err(http_error(status.as_u16(), &reason, &text));
        }
        Ok(serde_json::from_str(&text).unwrap_or(Value::Null))
    }

    /// `GET path`.
    ///
    /// # Errors
    /// As [`Client::request`].
    pub fn get(&self, path: &str) -> Result<Value, ClientError> {
        self.request("GET", path, None)
    }

    /// `POST path` with a JSON body.
    ///
    /// # Errors
    /// As [`Client::request`].
    pub fn post(&self, path: &str, body: &Value) -> Result<Value, ClientError> {
        self.request("POST", path, Some(body))
    }

    /// Opens a server-sent event stream and returns its body to read, until `timeout`.
    ///
    /// # Errors
    /// [`ClientError::Down`] when the server does not answer, [`ClientError::Http`] for an
    /// error status.
    pub fn stream(
        &self,
        path: &str,
        timeout: Duration,
    ) -> Result<Box<dyn Read + Send>, ClientError> {
        let builder = self
            .builder("GET", path, false)
            .header("accept", "text/event-stream");
        let request = builder
            .body(())
            .map_err(|e| ClientError::down(Some(&e.to_string())))?;
        let response = agent(Some(timeout))
            .run(request)
            .map_err(|e| failed(&e, Some(timeout)))?;
        let status = response.status();
        if !status.is_success() {
            let reason = status.canonical_reason().unwrap_or_default().to_owned();
            return Err(http_error(status.as_u16(), &reason, &read_text(response)));
        }
        Ok(Box::new(response.into_body().into_reader()))
    }
}
