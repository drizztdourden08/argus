//! One HTTP/1.1 connection to the server's loopback port, read as a body that arrives in pieces
//! (chunked, as the server streams events). Nothing here knows what the text means.

use std::io;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// A request for an event stream.
pub struct Request<'a> {
    /// The loopback port.
    pub port: u16,
    /// The path and query.
    pub target: &'a str,
    /// The bearer, when the server wants one.
    pub token: Option<&'a str>,
}

/// An open response whose body is read piece by piece.
pub struct Connection {
    reader: BufReader<TcpStream>,
    chunked: bool,
    finished: bool,
}

fn refused(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

async fn header_lines(reader: &mut BufReader<TcpStream>) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 {
            return Err(refused("the connection closed before the headers ended"));
        }
        let line = line.trim_end().to_owned();
        if line.is_empty() {
            return Ok(lines);
        }
        lines.push(line);
    }
}

fn status_of(line: &str) -> u16 {
    line.split(' ')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

impl Connection {
    /// Connects, sends the request and reads the head of the answer.
    ///
    /// # Errors
    /// The connection fails, or the server answers anything but 200.
    pub async fn open(request: &Request<'_>) -> io::Result<Connection> {
        let mut stream = TcpStream::connect(("127.0.0.1", request.port)).await?;
        let auth = request
            .token
            .map(|t| format!("Authorization: Bearer {t}\r\n"))
            .unwrap_or_default();
        let head = format!(
            "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAccept: text/event-stream\r\n{auth}\r\n",
            request.target, request.port
        );
        stream.write_all(head.as_bytes()).await?;
        let mut reader = BufReader::new(stream);
        let lines = header_lines(&mut reader).await?;
        let status = status_of(lines.first().map_or("", String::as_str));
        if status != 200 {
            return Err(refused(format!("the dashboard answered HTTP {status}")));
        }
        let chunked = lines.iter().any(|l| {
            let lower = l.to_ascii_lowercase();
            lower.starts_with("transfer-encoding:") && lower.contains("chunked")
        });
        Ok(Connection {
            reader,
            chunked,
            finished: false,
        })
    }

    async fn chunk(&mut self) -> io::Result<Option<Vec<u8>>> {
        let mut size_line = String::new();
        if self.reader.read_line(&mut size_line).await? == 0 {
            return Ok(None);
        }
        let digits = size_line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(digits, 16).map_err(|_| refused("a bad chunk size"))?;
        let mut data = vec![0u8; size + 2];
        self.reader.read_exact(&mut data).await?;
        data.truncate(size);
        Ok((size > 0).then_some(data))
    }

    async fn plain(&mut self) -> io::Result<Option<Vec<u8>>> {
        let mut data = vec![0u8; 8192];
        let n = self.reader.read(&mut data).await?;
        data.truncate(n);
        Ok((n > 0).then_some(data))
    }

    /// The next piece of the body; `None` when the server ended it.
    ///
    /// # Errors
    /// The connection fails or the body is malformed.
    pub async fn next(&mut self) -> io::Result<Option<Vec<u8>>> {
        if self.finished {
            return Ok(None);
        }
        let piece = if self.chunked {
            self.chunk().await?
        } else {
            self.plain().await?
        };
        self.finished = piece.is_none();
        Ok(piece)
    }
}
