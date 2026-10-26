//! A tiny local HTTP server on an ephemeral port that mounts the handler at `POST /mcp` and
//! answers `GET /api/health`, with the bearer rule of the real server: a token that is sent
//! and wrong is 401, and a missing one only when the token is required.

use std::collections::HashMap;

use argus_mcp::{McpRequest, ToolHost, handle_other, handle_post};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

pub const TOKEN: &str = "tokentokentokentokentoken";

pub struct Stub {
    pub port: u16,
    task: JoinHandle<()>,
}

struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Stub {
    pub async fn start(host: ToolHost, require_token: bool) -> Stub {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(serve(stream, host.clone(), require_token));
            }
        });
        Stub { port, task }
    }

    pub async fn close(self) {
        self.task.abort();
        let _ = self.task.await;
    }
}

async fn read_request(stream: &mut TcpStream) -> Option<Request> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        if let Some(at) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break at;
        }
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let (method, path) = (first.next()?.to_owned(), first.next()?.to_owned());
    let headers: HashMap<String, String> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_owned()))
        .collect();
    let wanted: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut body = buf[head_end + 4..].to_vec();
    while body.len() < wanted {
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    Some(Request {
        method,
        path,
        headers,
        body,
    })
}

async fn respond(stream: &mut TcpStream, status: u16, body: &str) {
    let head = format!(
        "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes()).await;
    let _ = stream.write_all(body.as_bytes()).await;
    let _ = stream.shutdown().await;
}

fn unauthorized(req: &Request, require_token: bool) -> bool {
    let sent = req.headers.get("authorization");
    let wrong = sent.is_some_and(|s| s != &format!("Bearer {TOKEN}"));
    wrong || (require_token && sent.is_none())
}

async fn serve(mut stream: TcpStream, host: ToolHost, require_token: bool) {
    let Some(req) = read_request(&mut stream).await else {
        return;
    };
    if unauthorized(&req, require_token) {
        return respond(&mut stream, 401, r#"{"error":{"code":"unauthorized"}}"#).await;
    }
    let header = |name: &str| req.headers.get(name).map(String::as_str);
    let reply = match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/api/health") => return respond(&mut stream, 200, r#"{"ok":true}"#).await,
        ("POST", "/mcp") => {
            let request = McpRequest {
                protocol_version: header("mcp-protocol-version"),
                default_project: header("x-coord-default-project"),
                client: header("x-coord-client"),
                user_agent: header("user-agent"),
                host: header("host"),
                body: &req.body,
            };
            handle_post(&host, &request).await
        }
        ("GET" | "DELETE", "/mcp") => handle_other(),
        _ => return respond(&mut stream, 404, r#"{"error":{"code":"not_found"}}"#).await,
    };
    let body = reply.body().map(|b| b.to_string()).unwrap_or_default();
    respond(&mut stream, reply.status(), &body).await;
}
