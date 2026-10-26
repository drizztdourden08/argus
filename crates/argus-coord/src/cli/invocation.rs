//! Everything one `coord` call touches, so the verbs never read the process: arguments,
//! environment, working directory, the three streams and the clock.

use std::io::{self, Cursor, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;

use argus_core::clock::{Clock, SystemClock};

use super::capture::Capture;
use super::env::Env;

/// One call of `coord`.
pub struct Invocation {
    /// The arguments after `coord`.
    pub argv: Vec<String>,
    /// The environment variables.
    pub env: Env,
    /// The working directory.
    pub cwd: PathBuf,
    /// Standard input; a verb that reads it does so only after the router decided to run it.
    pub stdin: Box<dyn Read + Send>,
    /// Standard output.
    pub stdout: Box<dyn Write + Send>,
    /// Standard error.
    pub stderr: Box<dyn Write + Send>,
    /// The time source.
    pub clock: Arc<dyn Clock>,
}

impl Invocation {
    /// A call of this process: its environment, working directory and streams.
    #[must_use]
    pub fn from_process(argv: Vec<String>) -> Self {
        Invocation {
            argv,
            env: Env::from_process(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            stdin: Box::new(io::stdin()),
            stdout: Box::new(io::stdout()),
            stderr: Box::new(io::stderr()),
            clock: Arc::new(SystemClock),
        }
    }

    /// A call with captured streams and the given standard input: the call, its stdout and its
    /// stderr.
    #[must_use]
    pub fn captured(
        argv: &[&str],
        env: Env,
        cwd: PathBuf,
        stdin: &str,
    ) -> (Self, Capture, Capture) {
        let (out, err) = (Capture::default(), Capture::default());
        let call = Invocation {
            argv: argv.iter().map(|a| (*a).to_owned()).collect(),
            env,
            cwd,
            stdin: Box::new(Cursor::new(stdin.as_bytes().to_vec())),
            stdout: Box::new(out.clone()),
            stderr: Box::new(err.clone()),
            clock: Arc::new(SystemClock),
        };
        (call, out, err)
    }

    /// `console.log(lines.join('\n'))` on stdout.
    pub fn say(&mut self, lines: &[&str]) {
        super::out::say(&mut self.stdout, lines);
    }

    /// `console.error(text)` on stderr.
    pub fn warn(&mut self, text: &str) {
        super::out::say(&mut self.stderr, &[text]);
    }
}
