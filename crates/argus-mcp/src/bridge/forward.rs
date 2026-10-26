//! Forwarding a message to the running server's `/mcp`, and the answer when it is down.

use serde_json::{Value, json};

use super::env::BridgeEnv;
use super::link::Link;
use crate::rpc::{code, reply, rpc_error};

/// What a call says when the owner chose Quit in the tray.
pub const CLOSED_FROM_TRAY: &str =
    "the dashboard was closed from the tray; run `dashboard tray start` to reopen it";

impl<E: BridgeEnv> Link<E> {
    fn down(&self, msg: &Value) -> Value {
        let closed = self.closed_by_owner();
        let why = self.why();
        let text = if closed {
            CLOSED_FROM_TRAY.to_owned()
        } else {
            let why = if why.is_empty() {
                "could not reach it"
            } else {
                &why
            };
            format!("dashboard not running: {why}. Run `dashboard start` and try again.")
        };
        if msg["method"] != "tools/call" {
            return rpc_error(msg.get("id"), code::INTERNAL, &text);
        }
        let code = if closed {
            "closed_from_tray"
        } else {
            "unavailable"
        };
        let error = json!({ "code": code, "message": text });
        let body = json!({ "error": error });
        reply(
            &msg["id"],
            json!({
                "content": [{ "type": "text", "text": body.to_string() }],
                "structuredContent": body,
                "isError": true,
            }),
        )
    }

    async fn read_reply(&self, res: reqwest::Response, msg: &Value) -> Option<Value> {
        let status = res.status();
        if status.as_u16() == 202 {
            return None;
        }
        let body: Option<Value> = res
            .text()
            .await
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok());
        if let Some(rpc) = body.clone().filter(|b| b["jsonrpc"] == "2.0") {
            return Some(rpc);
        }
        let detail = body
            .as_ref()
            .and_then(|b| b["error"]["message"].as_str())
            .filter(|m| !m.is_empty())
            .map_or_else(String::new, |m| format!(": {m}"));
        self.set_why(format!("the server answered {}{detail}", status.as_u16()));
        Some(self.down(msg))
    }

    /// Send `msg` to `/mcp` with the client name and the default project; none for a 202.
    pub(super) async fn forward(&self, msg: &Value) -> Option<Value> {
        if !self.ensure_up().await {
            return Some(self.down(msg));
        }
        let Some(endpoint) = self.env.endpoint() else {
            return Some(self.down(msg));
        };
        let (client, protocol) = self.session();
        let mut request = self
            .http
            .post(format!("{}/mcp", endpoint.base))
            .timeout(self.opts.call_timeout)
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .header("mcp-protocol-version", protocol)
            .header("x-coord-client", client)
            .body(msg.to_string());
        if let Some(auth) = Self::auth(&endpoint) {
            request = request.header("authorization", auth);
        }
        if let Some(project) = &self.opts.project {
            request = request.header("x-coord-default-project", project);
        }
        match request.send().await {
            Ok(res) => self.read_reply(res, msg).await,
            Err(e) => {
                self.set_why(e.to_string());
                Some(self.down(msg))
            }
        }
    }
}
