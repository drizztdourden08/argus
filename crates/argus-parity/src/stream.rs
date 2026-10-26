//! The doc stream of one scope, read from a server for a few seconds: the snapshot frames, one
//! per collection, and the `ready` frame that ends them. Only these are compared; the `changes`
//! frames that follow are live and belong to one server's own history.

use std::time::Duration;

use argus_core::Coll;
use serde_json::{Value, json};
use tokio::time::{Instant, timeout_at};

use crate::got::Got;
use crate::kind::Kind;
use crate::side::Side;

/// One server-sent frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    /// The `event` line.
    pub event: String,
    /// The `data` line as JSON (`null` when it is not JSON).
    pub data: Value,
}

/// What a frame contributes to the comparison: a label, the rules' kind and the answer.
pub type Entry = (String, Kind, Got);

fn parse_block(block: &str) -> Option<Frame> {
    let mut event = "message".to_owned();
    let mut data = Vec::new();
    for line in block.lines() {
        if let Some(v) = line.strip_prefix("event:") {
            v.trim().clone_into(&mut event);
        } else if let Some(v) = line.strip_prefix("data:") {
            data.push(v.strip_prefix(' ').unwrap_or(v));
        }
    }
    if data.is_empty() {
        return None;
    }
    let data = serde_json::from_str(&data.join("\n")).unwrap_or(Value::Null);
    Some(Frame { event, data })
}

/// Takes the complete frames off the front of the buffer (comments and bare lines are dropped).
fn take_frames(buf: &mut Vec<u8>) -> Vec<Frame> {
    let mut out = Vec::new();
    while let Some(at) = buf.windows(2).position(|w| w == b"\n\n") {
        let block: Vec<u8> = buf.drain(..at + 2).collect();
        if let Some(frame) = parse_block(&String::from_utf8_lossy(&block)) {
            out.push(frame);
        }
    }
    out
}

/// Reads frames until `ready`, the end of the stream, or the time is up.
async fn read_frames(side: &Side, path: &str, secs: u64) -> Result<Vec<Frame>, String> {
    let mut res = side.open_stream(path).await?;
    let until = Instant::now() + Duration::from_secs(secs);
    let (mut buf, mut frames) = (Vec::new(), Vec::new());
    while !frames.iter().any(|f: &Frame| f.event == "ready") {
        match timeout_at(until, res.chunk()).await {
            Ok(Ok(Some(bytes))) => {
                buf.extend(bytes.iter().filter(|b| **b != b'\r'));
                frames.extend(take_frames(&mut buf));
            }
            Ok(Err(e)) => return Err(e.without_url().to_string()),
            Ok(Ok(None)) | Err(_) => break,
        }
    }
    Ok(frames)
}

fn entry(frame: &Frame) -> Option<Entry> {
    match frame.event.as_str() {
        "ready" => Some((
            "ready".to_owned(),
            Kind::Ready,
            Got::Doc(json!({ "ready": true })),
        )),
        "snapshot" => {
            let name = frame.data["coll"].as_str()?;
            let coll = Coll::ALL.iter().find(|c| c.as_str() == name)?.as_str();
            let docs = frame.data["docs"].clone();
            Some((
                format!("snapshot {coll}"),
                Kind::Doc(coll),
                Got::Doc(json!({ "docs": docs })),
            ))
        }
        _ => None,
    }
}

/// The entries of one server's stream of `scope`; a stream that cannot be read is one refused
/// `ready` entry.
pub async fn read(side: &Side, scope: &str, secs: u64) -> Vec<Entry> {
    let path = format!("/api/p/{scope}/stream");
    match read_frames(side, &path, secs).await {
        Ok(frames) => frames.iter().filter_map(entry).collect(),
        Err(why) => vec![("ready".to_owned(), Kind::Ready, Got::none(why))],
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Frame, entry, take_frames};
    use crate::kind::Kind;

    #[test]
    fn complete_frames_are_taken_and_the_rest_waits() {
        let mut buf = b": ping\n\nevent: ready\ndata: {\"seq\":1}\n\nevent: sna".to_vec();
        let frames = take_frames(&mut buf);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].event, "ready");
        assert_eq!(buf, b"event: sna");
    }

    #[test]
    fn a_snapshot_becomes_the_docs_of_its_collection() {
        let frame = Frame {
            event: "snapshot".into(),
            data: json!({ "scope": "a", "seq": 9, "coll": "items", "docs": [{ "id": "I-1" }] }),
        };
        let (label, kind, _) = entry(&frame).expect("an entry");
        assert_eq!(label, "snapshot items");
        assert_eq!(kind, Kind::Doc("items"));
    }

    #[test]
    fn other_frames_are_not_compared() {
        let frame = Frame {
            event: "changes".into(),
            data: json!({}),
        };
        assert!(entry(&frame).is_none());
    }
}
