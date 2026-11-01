//! The stand-in dashboard of the hook tests: a local server that records the path of every
//! request and hands the stream to the test's own answer, and the fixture that points a checkout
//! at its port.

use std::fs;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use super::Fixture;

pub struct StandIn {
    pub port: u16,
    paths: Arc<Mutex<Vec<String>>>,
}

fn request_path(stream: &TcpStream) -> String {
    let mut reader = BufReader::new(stream);
    let mut first = String::new();
    reader.read_line(&mut first).ok();
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok_and(|n| n > 2) {
        line.clear();
    }
    first
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_owned()
}

impl StandIn {
    /// Serves on a free port; each request is recorded, then `on_request` gets its number, the
    /// stream and the path.
    pub fn start<F>(on_request: F) -> StandIn
    where
        F: Fn(usize, TcpStream, &str) + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let paths = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&paths);
        thread::spawn(move || {
            for (n, stream) in listener.incoming().flatten().enumerate() {
                let path = request_path(&stream);
                seen.lock().unwrap().push(path.clone());
                on_request(n, stream, &path);
            }
        });
        StandIn { port, paths }
    }

    pub fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }
}

/// A fixture whose dashboard config names the stand-in's port.
pub fn fixture(server: &StandIn) -> Fixture {
    let f = Fixture::new();
    let config = f.home.dashboard_home().join("config.json");
    fs::write(config, format!(r#"{{"port":{}}}"#, server.port)).unwrap();
    f
}
