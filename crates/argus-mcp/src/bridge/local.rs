//! The messages the bridge answers itself, so `initialize` and `tools/list` work before the
//! server is even up: the tool list is static and the resources are listed by the server.

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::context::CallContext;
use crate::defs::tool_defs;
use crate::error::ToolError;
use crate::outcome::ToolOutcome;
use crate::resources::list_resource_templates;
use crate::rpc::RpcHost;

/// The methods answered without the server.
pub const LOCAL_METHODS: [&str; 4] = [
    "initialize",
    "ping",
    "tools/list",
    "resources/templates/list",
];

/// A host with the static parts only.
pub struct LocalHost;

#[async_trait]
impl RpcHost for LocalHost {
    fn list_tools(&self) -> Vec<Value> {
        tool_defs().to_vec()
    }

    fn list_resources(&self) -> Vec<Value> {
        Vec::new()
    }

    fn list_resource_templates(&self) -> Vec<Value> {
        list_resource_templates()
    }

    async fn call_tool(&self, _name: &str, _args: &Value, _ctx: &CallContext) -> ToolOutcome {
        ToolOutcome {
            result: json!({}),
            is_error: true,
        }
    }

    async fn read_resource(&self, _uri: &str, _ctx: &CallContext) -> Result<Value, ToolError> {
        Ok(json!({ "contents": [] }))
    }
}
