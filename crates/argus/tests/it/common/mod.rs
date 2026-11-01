//! Helpers shared by the end to end CLI tests.

use std::process::Output;

/// What a finished `argus` child process left behind.
pub struct Out {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl From<Output> for Out {
    fn from(out: Output) -> Out {
        Out {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }
}
