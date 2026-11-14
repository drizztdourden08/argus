use serde_json::json;

use super::*;

const AT: &str = "2026-11-14T10:00:00.000Z";

fn asked(queue: &mut Queue) -> Approval {
    queue.submit("Bash", "git push", AT, 1_000)
}

// test-tag: critical
#[test]
fn a_call_without_the_right_bearer_never_matches() {
    assert!(bearer_ok(Some("Bearer abc123"), Some("abc123")));
    assert!(!bearer_ok(None, Some("abc123")));
    assert!(!bearer_ok(Some("Bearer abc124"), Some("abc123")));
    assert!(!bearer_ok(Some("Bearer abc12"), Some("abc123")));
    assert!(!bearer_ok(Some("abc123"), Some("abc123")));
    assert!(!bearer_ok(Some("Bearer abc123"), None));
    assert!(!bearer_ok(Some("Bearer "), Some("")));
}

// test-tag: critical
#[test]
fn the_timeout_decides_after_the_whole_wait() {
    assert!(!timed_out(0, 29 * 60_000 + 59_999, timeout_ms(30)));
    assert!(timed_out(0, 30 * 60_000, timeout_ms(30)));
    assert_eq!(timeout_ms(30), 1_800_000);
}

// test-tag: critical
#[test]
fn an_expired_approval_is_denied_with_the_owner_did_not_answer() {
    let mut queue = Queue::default();
    let item = asked(&mut queue);
    let expired = queue.expire(&item.id, "timeout", AT).unwrap();
    assert_eq!(expired.status, Status::Expired);
    assert_eq!(
        reply_for(&expired, &json!({})),
        json!({ "behavior": "deny", "message": "the owner did not answer" })
    );
}

// test-tag: critical
#[test]
fn the_allow_and_deny_replies_have_the_documented_shape() {
    let mut queue = Queue::default();
    let input = json!({ "command": "git push" });
    let first = asked(&mut queue);
    let allowed = queue.decide(&first.id, Decision::Allow, None, AT).unwrap();
    assert_eq!(
        reply_for(&allowed, &input),
        json!({ "behavior": "allow", "updatedInput": { "command": "git push" } })
    );
    let second = asked(&mut queue);
    let denied = queue
        .decide(&second.id, Decision::Deny, Some("not now"), AT)
        .unwrap();
    assert_eq!(
        reply_for(&denied, &input),
        json!({ "behavior": "deny", "message": "not now" })
    );
    let third = asked(&mut queue);
    let bare = queue.decide(&third.id, Decision::Deny, None, AT).unwrap();
    assert_eq!(reply_for(&bare, &input)["message"], json!(DENIED_DEFAULT));
}

// test-tag: critical
#[test]
fn the_tool_result_carries_the_reply_as_text_content() {
    let wrapped = tool_result(&json!(7), &deny("no"));
    assert_eq!(wrapped["id"], json!(7));
    let text = wrapped["result"]["content"][0]["text"].as_str().unwrap();
    let back: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(back, json!({ "behavior": "deny", "message": "no" }));
    assert_eq!(wrapped["result"]["content"][0]["type"], json!("text"));
}

// test-tag: standard
#[test]
fn the_queue_settles_each_approval_once() {
    let mut queue = Queue::default();
    let item = asked(&mut queue);
    assert_eq!(queue.pending(), vec![item.id.clone()]);
    queue.decide(&item.id, Decision::Allow, None, AT).unwrap();
    assert!(queue.pending().is_empty());
    assert_eq!(
        queue.decide(&item.id, Decision::Deny, None, AT),
        Err(DecideFault::Settled)
    );
    assert_eq!(
        queue.expire(&item.id, "timeout", AT),
        Err(DecideFault::Settled)
    );
    assert_eq!(
        queue.decide("nope", Decision::Allow, None, AT),
        Err(DecideFault::Unknown)
    );
    let ids: Vec<_> = (0..3).map(|_| asked(&mut queue).id).collect();
    assert_eq!(queue.recent(2)[0].id, ids[2]);
    assert_eq!(queue.recent(2).len(), 2);
}

// test-tag: standard
#[test]
fn the_queue_never_drops_a_pending_approval() {
    let mut queue = Queue::default();
    for _ in 0..KEEP + 5 {
        asked(&mut queue);
    }
    assert_eq!(queue.pending().len(), KEEP + 5);
}

// test-tag: standard
#[test]
fn the_rpc_answers_the_handshake_and_the_listing_and_finds_the_question() {
    let init = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} });
    let Parsed::Now(reply) = parse(init.to_string().as_bytes()) else {
        panic!("initialize answers at once");
    };
    assert_eq!(
        reply["result"]["serverInfo"]["name"],
        json!("argus-overlord")
    );
    let note = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
    assert_eq!(parse(note.to_string().as_bytes()), Parsed::Quiet);
    let list = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" });
    let Parsed::Now(listing) = parse(list.to_string().as_bytes()) else {
        panic!("tools/list answers at once");
    };
    assert_eq!(
        listing["result"]["tools"][0]["name"],
        json!("permission_prompt")
    );
    let call = json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
        "name": "permission_prompt",
        "arguments": { "tool_name": "Bash", "input": { "command": "ls" }, "tool_use_id": "tu1" }
    } });
    let Parsed::Ask(prompt) = parse(call.to_string().as_bytes()) else {
        panic!("a call is a question");
    };
    assert_eq!(prompt.tool_name, "Bash");
    assert_eq!(prompt.input, json!({ "command": "ls" }));
    assert_eq!(prompt.tool_use_id.as_deref(), Some("tu1"));
    assert!(matches!(parse(b"not json"), Parsed::Now(_)));
}

// test-tag: critical
#[test]
fn the_mcp_config_names_the_endpoint_and_the_bearer() {
    let text = mcp_config_json("http://127.0.0.1:9/mcp/overlord", "tok");
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let server = &value["mcpServers"]["argus-overlord"];
    assert_eq!(server["url"], json!("http://127.0.0.1:9/mcp/overlord"));
    assert_eq!(server["headers"]["Authorization"], json!("Bearer tok"));
}

// test-tag: standard
#[test]
fn the_summary_names_the_command_or_cuts_the_input() {
    assert_eq!(summarize(&json!({ "command": "git push" })), "git push");
    assert_eq!(summarize(&json!({ "x": 1 })), "{\"x\":1}");
    let long = "é".repeat(500);
    assert_eq!(summarize(&json!({ "command": long })).chars().count(), 200);
}
