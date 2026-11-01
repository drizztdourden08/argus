//! `logs/tray.log`: one line per event, cut when it grows past 200 kB. A failing log never
//! disturbs the tray.
// test-tag: critical

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::time::SystemTime;

use super::paths::TrayFiles;
use super::timestamp::iso_utc;

const LOG_MAX_BYTES: u64 = 200_000;

/// Appends lines to the tray log.
#[derive(Debug, Clone)]
pub struct TrayLog {
    files: TrayFiles,
}

impl TrayLog {
    /// The log of the dashboard home of `files`.
    #[must_use]
    pub fn new(files: TrayFiles) -> Self {
        Self { files }
    }

    /// Appends `<iso time> <message>`; errors are dropped.
    pub fn line(&self, message: &str) {
        let _written = self.append(message);
    }

    fn append(&self, message: &str) -> io::Result<()> {
        fs::create_dir_all(self.files.log_dir())?;
        let path = self.files.log_file();
        if fs::metadata(&path).is_ok_and(|meta| meta.len() > LOG_MAX_BYTES) {
            fs::remove_file(&path)?;
        }
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{} {message}", iso_utc(SystemTime::now()))
    }
}

#[cfg(test)]
mod tests {
    use super::{LOG_MAX_BYTES, TrayLog};
    use crate::tray::paths::TrayFiles;

    #[test]
    fn lines_append_with_a_time_and_a_big_log_starts_over() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        let log = TrayLog::new(files.clone());
        log.line("server running (2 runs)");
        log.line("server stopped");
        let text = std::fs::read_to_string(files.log_file()).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].ends_with("Z server running (2 runs)"));
        let filler = "x".repeat(usize::try_from(LOG_MAX_BYTES).unwrap() + 1);
        std::fs::write(files.log_file(), filler).unwrap();
        log.line("fresh");
        assert_eq!(
            std::fs::read_to_string(files.log_file())
                .unwrap()
                .lines()
                .count(),
            1
        );
    }
}
