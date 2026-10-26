//! A stream that keeps what is written to it, for the tests that run `coord` in-process.

use std::io::{self, Write};
use std::sync::{Arc, Mutex, PoisonError};

/// A shared buffer: clones write into the same bytes.
#[derive(Debug, Clone, Default)]
pub struct Capture(Arc<Mutex<Vec<u8>>>);

impl Capture {
    /// Everything written so far, as text.
    #[must_use]
    pub fn text(&self) -> String {
        let bytes = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

impl Write for Capture {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
