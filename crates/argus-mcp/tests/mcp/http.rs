//! The `/mcp` handler: JSON-RPC framing, initialize, tools/list, the tools/call envelope,
//! resources and every transport-level refusal (the cases of `mcp.http` and the protocol side
//! of `mcp.server`).
//! K2, K4: the /mcp framing, envelope and refusals.
// test-tag: critical

use argus_mcp::{
    INSTRUCTIONS, MAX_BYTES, McpReply, McpRequest, SUPPORTED_VERSIONS, handle_other, handle_post,
};
use serde_json::{Value, json};

use crate::support::{Rig, call};

fn init() -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "vitest", "version": "1" } } })
}

async fn post_raw(rig: &Rig, body: &[u8], shape: impl FnOnce(&mut McpRequest<'_>)) -> McpReply {
    let mut request = McpRequest {
        body,
        ..McpRequest::default()
    };
    shape(&mut request);
    handle_post(&rig.host, &request).await
}

async fn post(rig: &Rig, msg: &Value) -> McpReply {
    post_raw(rig, msg.to_string().as_bytes(), |_| {}).await
}

async fn rpc(rig: &Rig, msg: Value) -> Value {
    let reply = post(rig, &msg).await;
    assert_eq!(reply.status(), 200, "{reply:?}");
    reply.body().expect("a body")
}

async fn rpc_with(rig: &Rig, msg: Value, shape: impl FnOnce(&mut McpRequest<'_>)) -> Value {
    let reply = post_raw(rig, msg.to_string().as_bytes(), shape).await;
    reply.body().expect("a body")
}

#[tokio::test]
async fn initialize_answers_with_the_server_info_capabilities_and_instructions() {
    let rig = Rig::new();
    let body = rpc(&rig, init()).await;
    assert_eq!(body["id"], 1);
    assert_eq!(body["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(body["result"]["serverInfo"]["name"], "coord-dashboard");
    assert_eq!(
        body["result"]["capabilities"],
        json!({ "tools": { "listChanged": false }, "resources": {} })
    );
    assert!(
        body["result"]["instructions"]
            .as_str()
            .is_some_and(|i| i.contains("act as the owner"))
    );
    assert_eq!(body["result"]["instructions"], INSTRUCTIONS);
}

#[tokio::test]
async fn an_older_protocol_version_is_echoed_and_an_unknown_one_falls_back() {
    let rig = Rig::new();
    let version = |asked: &str| {
        let mut msg = init();
        msg["params"]["protocolVersion"] = json!(asked);
        msg
    };
    assert_eq!(
        rpc(&rig, version("2025-03-26")).await["result"]["protocolVersion"],
        "2025-03-26"
    );
    assert_eq!(
        rpc(&rig, version("1999-01-01")).await["result"]["protocolVersion"],
        "2025-06-18"
    );
}

#[tokio::test]
async fn get_and_delete_are_405_with_allow_post() {
    let reply = handle_other();
    assert_eq!(reply.status(), 405);
    match &reply {
        McpReply::Refused { allow, error, .. } => {
            assert_eq!(*allow, Some("POST"));
            assert_eq!(error.code, "bad_request");
        }
        other => panic!("{other:?}"),
    }
    let body = reply.body().expect("a body");
    assert_eq!(
        body["error"]["message"],
        "this MCP endpoint takes POST only (no server-initiated stream)"
    );
}

#[tokio::test]
async fn a_notification_is_202_with_no_body_and_a_client_response_is_ignored() {
    let rig = Rig::new();
    let n = post(
        &rig,
        &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
    )
    .await;
    assert_eq!(n, McpReply::Accepted);
    assert_eq!(n.status(), 202);
    assert_eq!(n.body(), None);
    assert_eq!(
        post(&rig, &json!({ "jsonrpc": "2.0", "id": 5, "result": {} }))
            .await
            .status(),
        202
    );
}

#[tokio::test]
async fn bad_bodies_invalid_json_a_batch_not_rpc_and_a_body_over_the_cap() {
    let rig = Rig::new();
    let parse = post_raw(&rig, b"{nope", |_| {}).await;
    assert_eq!(parse.status(), 400);
    assert_eq!(parse.body().expect("a body")["error"]["code"], -32700);
    let batch = post(&rig, &json!([init()])).await;
    assert_eq!(batch.status(), 400);
    assert_eq!(batch.body().expect("a body")["error"]["code"], -32600);
    assert_eq!(
        batch.body().expect("a body")["error"]["message"],
        "send one JSON-RPC message per request"
    );
    let not_rpc = rpc(&rig, json!({ "id": 1, "method": "ping" })).await;
    assert_eq!(not_rpc["error"]["code"], -32600);
    assert_eq!(not_rpc["error"]["message"], "not a JSON-RPC 2.0 message");
    assert_eq!(not_rpc["id"], 1);
    let no_method = rpc(&rig, json!({ "jsonrpc": "2.0", "id": 2 })).await;
    assert_eq!(no_method["error"]["code"], -32600);
    assert_eq!(no_method["error"]["message"], "method is required");
    let big = vec![b'x'; MAX_BYTES + 1];
    let over = post_raw(&rig, &big, |_| {}).await;
    assert_eq!(over.status(), 413);
}

#[tokio::test]
async fn mcp_protocol_version_a_supported_value_passes_an_unknown_one_is_400() {
    let rig = Rig::new();
    let ping = json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" });
    let ok = post_raw(&rig, ping.to_string().as_bytes(), |r| {
        r.protocol_version = Some("2025-06-18");
    })
    .await;
    assert_eq!(ok.status(), 200);
    let bad = post_raw(&rig, ping.to_string().as_bytes(), |r| {
        r.protocol_version = Some("2030-01-01");
    })
    .await;
    assert_eq!(bad.status(), 400);
    let body = bad.body().expect("a body");
    assert_eq!(body["error"]["code"], "bad_request");
    assert_eq!(
        body["error"]["message"],
        "unsupported MCP-Protocol-Version 2030-01-01"
    );
    assert_eq!(
        body["error"]["details"]["supported"],
        json!(SUPPORTED_VERSIONS)
    );
}

#[tokio::test]
async fn ping_unknown_method_unknown_tool_and_a_missing_name() {
    let rig = Rig::new();
    assert_eq!(
        rpc(
            &rig,
            json!({ "jsonrpc": "2.0", "id": "p", "method": "ping" })
        )
        .await,
        json!({ "jsonrpc": "2.0", "id": "p", "result": {} })
    );
    let m = rpc(
        &rig,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "prompts/list" }),
    )
    .await;
    assert_eq!(
        m["error"],
        json!({ "code": -32601, "message": "Method not found: prompts/list" })
    );
    let t = rpc(&rig, call(3, "format_disk", json!({}))).await;
    assert_eq!(
        t["error"],
        json!({ "code": -32602, "message": "Unknown tool: format_disk" })
    );
    let no_name = rpc(
        &rig,
        json!({ "jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {} }),
    )
    .await;
    assert_eq!(
        no_name["error"],
        json!({ "code": -32602, "message": "name is required" })
    );
}

#[tokio::test]
async fn tools_call_wraps_the_result_as_text_structured_content_and_is_error() {
    let rig = Rig::new();
    let body = rpc(
        &rig,
        call(7, "board_summary", json!({ "project": "alpha" })),
    )
    .await;
    let r = &body["result"];
    assert_eq!(r["isError"], false);
    assert_eq!(r["content"][0]["type"], "text");
    let text: Value =
        serde_json::from_str(r["content"][0]["text"].as_str().expect("text")).expect("JSON");
    assert_eq!(text, r["structuredContent"]);
    assert_eq!(r["structuredContent"]["project"], "alpha");
    let keys: Vec<&str> = r
        .as_object()
        .expect("a result")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["content", "structuredContent", "isError"]);
}

#[tokio::test]
async fn the_text_is_compact_json_in_the_key_order_of_the_result() {
    let rig = Rig::new();
    let body = rpc(
        &rig,
        call(7, "get_item", json!({ "project": "alpha", "id": "I-404" })),
    )
    .await;
    let text = body["result"]["content"][0]["text"].as_str().expect("text");
    assert_eq!(
        text,
        r#"{"error":{"code":"not_found","message":"no item I-404 in alpha"}}"#
    );
}

#[tokio::test]
async fn a_failing_call_is_is_error_with_a_code_not_a_protocol_error() {
    let rig = Rig::new();
    let body = rpc(
        &rig,
        call(8, "get_item", json!({ "project": "alpha", "id": "I-404" })),
    )
    .await;
    assert_eq!(body["result"]["isError"], true);
    assert_eq!(
        body["result"]["structuredContent"]["error"]["code"],
        "not_found"
    );
    assert!(body.get("error").is_none());
}

#[tokio::test]
async fn the_default_project_and_client_name_headers_reach_the_tools() {
    let rig = Rig::new();
    let as_beta = |r: &mut McpRequest<'_>| {
        r.default_project = Some("beta");
        r.client = Some("claude-code");
    };
    let items = rpc_with(&rig, call(1, "list_items", json!({})), as_beta).await;
    assert_eq!(
        items["result"]["structuredContent"]["items"][0]["title"],
        "Pedestal in the beta game"
    );
    rpc_with(&rig, call(2, "decision", json!({ "text": "go" })), as_beta).await;
    let posted = rig.backend.calls().post[0].clone();
    assert_eq!(posted["scope"], "beta");
    assert_eq!(posted["via"], "mcp:claude-code");
    let none = rpc(&rig, call(3, "list_items", json!({}))).await;
    assert_eq!(
        none["result"]["structuredContent"]["error"]["code"],
        "bad_request"
    );
    let junk = rpc_with(&rig, call(4, "list_items", json!({})), |r| {
        r.default_project = Some("../etc");
    })
    .await;
    assert_eq!(
        junk["result"]["isError"], true,
        "a malformed default project is ignored, so the project is required"
    );
}

#[tokio::test]
async fn the_client_name_falls_back_to_the_user_agent_product_then_client() {
    let rig = Rig::new();
    let send = |agent: Option<&'static str>| {
        rpc_with(
            &rig,
            call(1, "send_to_lead", json!({ "project": "beta", "text": "x" })),
            move |r| r.user_agent = agent,
        )
    };
    send(Some("claude-code/2.1 (cli)")).await;
    send(None).await;
    let vias: Vec<Value> = rig
        .backend
        .calls()
        .send
        .iter()
        .map(|s| s["via"].clone())
        .collect();
    assert_eq!(vias, [json!("mcp:claude-code"), json!("mcp:client")]);
}

#[tokio::test]
async fn proof_urls_use_the_host_the_request_came_in_on() {
    let rig = Rig::new();
    let body = rpc_with(
        &rig,
        call(1, "get_test", json!({ "project": "alpha", "id": "T-1" })),
        |r| r.host = Some("127.0.0.1:50123"),
    )
    .await;
    let url = &body["result"]["structuredContent"]["test"]["proofs"][0]["url"];
    assert_eq!(url, "http://127.0.0.1:50123/files/alpha/proofs/T-1-1.png");
}

#[tokio::test]
async fn without_a_host_proof_urls_use_the_configured_port() {
    let rig = Rig::new();
    let body = rpc(
        &rig,
        call(1, "get_test", json!({ "project": "alpha", "id": "T-1" })),
    )
    .await;
    let url = &body["result"]["structuredContent"]["test"]["proofs"][0]["url"];
    assert_eq!(url, "http://127.0.0.1:47615/files/alpha/proofs/T-1-1.png");
}

#[tokio::test]
async fn resources_list_templates_read_and_an_unknown_uri() {
    let rig = Rig::new();
    let list = rpc(
        &rig,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "resources/list" }),
    )
    .await;
    assert!(
        list["result"]["resources"]
            .as_array()
            .expect("list")
            .iter()
            .any(|r| r["uri"] == "board://alpha/summary")
    );
    let tpl = rpc(
        &rig,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "resources/templates/list" }),
    )
    .await;
    assert_eq!(
        tpl["result"]["resourceTemplates"]
            .as_array()
            .expect("templates")
            .len(),
        4
    );
    let read = rpc(&rig, json!({ "jsonrpc": "2.0", "id": 3, "method": "resources/read", "params": { "uri": "portfolio://overview" } })).await;
    let content = &read["result"]["contents"][0];
    assert_eq!(content["mimeType"], "application/json");
    let cards: Value = serde_json::from_str(content["text"].as_str().expect("text")).expect("JSON");
    assert_eq!(cards["cards"].as_array().expect("cards").len(), 2);
    let bad = rpc(&rig, json!({ "jsonrpc": "2.0", "id": 4, "method": "resources/read", "params": { "uri": "board://alpha/nope" } })).await;
    assert_eq!(bad["error"]["code"], -32002);
    let none = rpc(
        &rig,
        json!({ "jsonrpc": "2.0", "id": 5, "method": "resources/read", "params": {} }),
    )
    .await;
    assert_eq!(
        none["error"],
        json!({ "code": -32602, "message": "uri is required" })
    );
}

#[tokio::test]
async fn a_service_failure_in_a_resource_is_an_internal_error() {
    let rig = Rig::over(argus_mcp::MemoryBackend::new(crate::support::fixture()));
    let body = rpc(&rig, json!({ "jsonrpc": "2.0", "id": 1, "method": "resources/read", "params": { "uri": "runs://alpha/R-1/log" } })).await;
    assert_eq!(body["error"]["code"], -32603);
    assert_eq!(body["error"]["message"], "the runs service is not running");
}
