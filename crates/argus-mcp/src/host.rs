//! The tool host: the one object every transport talks to. It checks the policy and the
//! arguments, resolves the project and hands the call to its tool.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::backend::McpBackend;
use crate::call::Call;
use crate::context::CallContext;
use crate::defs::{tool_def, tool_defs};
use crate::error::ToolError;
use crate::outcome::ToolOutcome;
use crate::policy::refusal;
use crate::project::require_project;
use crate::resources::{list_resource_templates, list_resources, read_resource};
use crate::rpc::RpcHost;
use crate::schema::validate;
use crate::tools::{NO_PROJECT, run};

/// The 23 tools and the resources over one backend.
#[derive(Clone)]
pub struct ToolHost {
    backend: Arc<dyn McpBackend>,
}

impl ToolHost {
    /// A host over `backend`.
    #[must_use]
    pub fn new(backend: Arc<dyn McpBackend>) -> Self {
        Self { backend }
    }

    fn check(&self, name: &str, args: &Value) -> Result<(), ToolError> {
        let Some(def) = tool_def(name) else {
            return Err(ToolError::new(
                "unknown_tool",
                format!("no tool named \"{name}\""),
            ));
        };
        if let Some(why) = refusal(&self.backend.policy(), name) {
            return Err(ToolError::new("forbidden", why));
        }
        let problems = validate(&def["inputSchema"], args, "arguments");
        if problems.is_empty() {
            Ok(())
        } else {
            Err(ToolError::bad_request(problems.join("; ")))
        }
    }

    async fn execute(
        &self,
        name: &str,
        args: &Value,
        ctx: &CallContext,
    ) -> Result<Value, ToolError> {
        let key = if NO_PROJECT.contains(&name) {
            None
        } else {
            let named = args.get("project").and_then(Value::as_str);
            let asked = named.or(ctx.default_project.as_deref());
            Some(require_project(self.backend.as_ref(), asked)?)
        };
        let base = ctx
            .base_url
            .clone()
            .unwrap_or_else(|| self.backend.default_base_url());
        let call = Call {
            backend: self.backend.as_ref(),
            key,
            args,
            via: ctx.via(),
            base,
            ctx,
        };
        run(name, &call).await
    }

    /// Call a tool by name: the policy, the arguments and the project are checked first, and
    /// every failure comes back as an error result, never a protocol error.
    pub async fn call(&self, name: &str, args: &Value, ctx: &CallContext) -> ToolOutcome {
        let empty = json!({});
        let args = if args.is_null() { &empty } else { args };
        let done = match self.check(name, args) {
            Ok(()) => self.execute(name, args, ctx).await,
            Err(e) => Err(e),
        };
        match done {
            Ok(result) => ToolOutcome::of(result),
            Err(e) => ToolOutcome::failed(&e),
        }
    }
}

#[async_trait]
impl RpcHost for ToolHost {
    fn list_tools(&self) -> Vec<Value> {
        tool_defs().to_vec()
    }

    fn list_resources(&self) -> Vec<Value> {
        list_resources(self.backend.as_ref())
    }

    fn list_resource_templates(&self) -> Vec<Value> {
        list_resource_templates()
    }

    async fn call_tool(&self, name: &str, args: &Value, ctx: &CallContext) -> ToolOutcome {
        self.call(name, args, ctx).await
    }

    async fn read_resource(&self, uri: &str, _ctx: &CallContext) -> Result<Value, ToolError> {
        read_resource(self.backend.as_ref(), uri)
    }
}
