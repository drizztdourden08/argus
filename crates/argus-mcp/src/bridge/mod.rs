//! `argus mcp [--project <key>]`: a stdio MCP server that forwards to the running server's
//! `/mcp`. Claude Code and Desktop spawn it, so it answers `initialize` and `tools/list` at
//! once from the static tool list, brings the server up in the background and forwards
//! everything else, one line in and one line out. stdout carries JSON-RPC lines and nothing
//! else; every log line goes through [`BridgeEnv::log`].

mod env;
mod forward;
mod link;
mod local;

use std::io;
use std::sync::Arc;

use argus_core::js;
use serde_json::Value;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::task::JoinSet;

pub use env::{BridgeEnv, BridgeOptions, Endpoint};
pub use forward::CLOSED_FROM_TRAY;
use link::Link;
use local::{LOCAL_METHODS, LocalHost};

use crate::context::CallContext;
use crate::rpc::{SUPPORTED_VERSIONS, code, handle_rpc, rpc_error};

type Out<W> = Arc<Mutex<W>>;

async fn send<W: AsyncWrite + Unpin>(out: &Out<W>, msg: &Value) -> io::Result<()> {
    let mut line = msg.to_string();
    line.push('\n');
    let mut writer = out.lock().await;
    writer.write_all(line.as_bytes()).await?;
    writer.flush().await
}

fn note_initialize<E: BridgeEnv>(link: &Link<E>, msg: &Value) {
    if msg["method"] != "initialize" {
        return;
    }
    let (client, _) = link.session();
    let named = msg["params"]["clientInfo"]
        .get("name")
        .filter(|n| !n.is_null())
        .map_or(client, js::string);
    let asked = msg["params"]["protocolVersion"]
        .as_str()
        .filter(|v| SUPPORTED_VERSIONS.contains(v))
        .map(str::to_owned);
    link.set_session(Some(named), asked);
}

async fn answer<E: BridgeEnv>(link: &Link<E>, msg: &Value) -> Option<Value> {
    let has_id = msg.get("id").is_some_and(|id| !id.is_null());
    let method = msg["method"].as_str().unwrap_or_default();
    if !has_id || LOCAL_METHODS.contains(&method) {
        return handle_rpc(&LocalHost, msg, &CallContext::default()).await;
    }
    if js::truthy(&msg["method"]) {
        link.forward(msg).await
    } else {
        None
    }
}

async fn handle_line<E: BridgeEnv, W: AsyncWrite + Unpin>(
    link: &Link<E>,
    out: &Out<W>,
    raw: &str,
) -> io::Result<()> {
    let Ok(msg) = serde_json::from_str::<Value>(raw) else {
        return send(out, &rpc_error(None, code::PARSE, "not valid JSON")).await;
    };
    note_initialize(link, &msg);
    match answer(link, &msg).await {
        Some(reply) => send(out, &reply).await,
        None => Ok(()),
    }
}

/// Run the bridge until `input` ends and every pending call is answered. Calls run
/// concurrently, each answered on its own line.
///
/// # Errors
/// A failure to read `input`.
pub async fn run_bridge<E, R, W>(env: E, opts: BridgeOptions, input: R, output: W) -> io::Result<()>
where
    E: BridgeEnv,
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Send + Unpin + 'static,
{
    let boot = opts.boot_launch;
    let link = Arc::new(Link::new(env, opts));
    if boot && !link.env.quit_flag_active() {
        link.launch(true);
    }
    let out: Out<W> = Arc::new(Mutex::new(output));
    let mut lines = input.lines();
    let mut pending = JoinSet::new();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let (link, out) = (Arc::clone(&link), Arc::clone(&out));
        pending.spawn(async move {
            if let Err(e) = handle_line(&link, &out, &line).await {
                link.say(&format!("failed: {e}"));
            }
        });
    }
    while pending.join_next().await.is_some() {}
    Ok(())
}
