//! The `argus parity` command line: the flags, and the run that prints the report.

use std::path::PathBuf;

use clap::Args as ClapArgs;

use crate::run::{Options, compare};

/// The flags of `argus parity`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The left server, for example the Node dashboard: `http://127.0.0.1:47615`.
    #[arg(long)]
    left: String,
    /// The right server, for example the shadow: `http://127.0.0.1:47616`.
    #[arg(long)]
    right: String,
    /// The bearer of the left server, when it requires one.
    #[arg(long)]
    token_left: Option<String>,
    /// The bearer of the right server, when it requires one.
    #[arg(long)]
    token_right: Option<String>,
    /// Compare only this project (repeat for more); the portfolio is left out then.
    #[arg(long = "project")]
    projects: Vec<String>,
    /// Print the report as JSON.
    #[arg(long)]
    json: bool,
    /// Also write the report to this file; nothing else is ever written.
    #[arg(long)]
    out: Option<PathBuf>,
    /// How far apart two timestamps may be, in milliseconds, and still be equal.
    #[arg(long, default_value_t = 2000)]
    tolerance_ms: i64,
    /// How many seconds the doc streams are read (0 skips them).
    #[arg(long, default_value_t = 5)]
    stream_secs: u64,
    /// Leave out the routes whose name contains this word (repeat for more).
    #[arg(long = "skip")]
    skip: Vec<String>,
    /// How many differing paths a route shows.
    #[arg(long, default_value_t = 8)]
    max_diffs: usize,
}

impl Args {
    fn options(&self) -> Options {
        let mut o = Options::new(&self.left, &self.right);
        o.token_left.clone_from(&self.token_left);
        o.token_right.clone_from(&self.token_right);
        o.projects.clone_from(&self.projects);
        o.tolerance_ms = self.tolerance_ms;
        o.stream_secs = self.stream_secs;
        o.skip.clone_from(&self.skip);
        o
    }
}

/// Runs the comparison, prints the report and answers the exit code: 0 when every route is
/// equal, 1 when any differs or is missing.
///
/// # Errors
/// A server cannot be reached, or the `--out` file cannot be written.
pub async fn run(args: Args) -> Result<u8, String> {
    let report = compare(&args.options()).await?;
    let text = if args.json {
        serde_json::to_string_pretty(&report.json(args.max_diffs)).map_err(|e| e.to_string())?
            + "\n"
    } else {
        report.text(args.max_diffs)
    };
    print!("{text}");
    if let Some(file) = &args.out {
        std::fs::write(file, &text).map_err(|e| format!("cannot write {}: {e}", file.display()))?;
    }
    Ok(report.exit_code())
}
