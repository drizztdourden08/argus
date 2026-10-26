//! The owner writes of the tools, through the sync service's post path with the source `mcp`:
//! entries, notes to the lead and proofs. Shadow mode refuses them all.

use std::sync::Arc;

use argus_mcp::{Image, ToolError};
use argus_sync::{ProofSource, Source, SyncService};
use serde_json::{Value, json, to_value};

use super::errors::{api, read_only};
use crate::Services;

fn writer(svc: &Services) -> Result<Arc<SyncService>, ToolError> {
    if svc.shadow {
        return Err(read_only("shadow mode: Argus does not write boards"));
    }
    svc.sync
        .clone()
        .ok_or_else(|| ToolError::unavailable("the sync service is not running"))
}

/// Applies an owner post as the MCP client `via`.
pub async fn entry(svc: &Services, key: &str, entry: Value, via: &str) -> Result<Value, ToolError> {
    let posted = writer(svc)?
        .post_entry(key, &entry, Source::Mcp, Some(via))
        .await;
    posted
        .map(|r| to_value(r).unwrap_or(Value::Null))
        .map_err(api)
}

/// Sends a note down to the lead of a project, or of all.
pub async fn lead(
    svc: &Services,
    target: &str,
    text: &str,
    decision: bool,
    via: &str,
) -> Result<Value, ToolError> {
    let sent = writer(svc)?.send_to_lead(target, text, decision, via).await;
    sent.map(|r| to_value(r).unwrap_or(Value::Null))
        .map_err(api)
}

/// Keeps a picture on a test: `{n, url}`.
pub async fn proof(
    svc: &Services,
    (key, test): (&str, &str),
    image: &Image,
    caption: &str,
    via: &str,
) -> Result<Value, ToolError> {
    let source = ProofSource::Bytes {
        bytes: image.bytes.clone(),
        mime: Some(image.mime.to_owned()),
    };
    let kept = writer(svc)?
        .attach_proof(key, test, source, caption, via)
        .await;
    kept.map(|p| json!({ "n": p.n, "url": p.url })).map_err(api)
}
