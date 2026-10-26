//! Whether a window responds. Only Windows can say, and only through its process API, which needs
//! unsafe code to call natively, so the answer comes from one hidden PowerShell asked about just the
//! processes whose rule wants the hung flag, and only when there are some.

use crate::ports::Responder;
use crate::shell::{TIMEOUT, run_hidden};
use std::collections::HashMap;

/// Asks PowerShell about the windows of a few processes.
#[derive(Debug, Default, Clone, Copy)]
pub struct PowerShellResponder;

/// Answers that nothing is known, so no process is ever hung. For tests and for systems without windows.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoResponder;

fn script(pids: &[u32]) -> String {
    let list = pids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "$ErrorActionPreference='SilentlyContinue'; foreach($i in {list}){{ $p=[System.Diagnostics.Process]::GetProcessById($i); if($p -and $p.MainWindowHandle -ne [IntPtr]::Zero){{ \"$i=$($p.Responding)\" }} }}"
    )
}

/// `pid=True` / `pid=False` lines to a map; anything else is ignored.
#[must_use]
pub fn parse_responding(stdout: &str) -> HashMap<u32, bool> {
    stdout
        .lines()
        .filter_map(|line| {
            let (pid, state) = line.trim().split_once('=')?;
            let responds = match state.trim().to_lowercase().as_str() {
                "true" => true,
                "false" => false,
                _ => return None,
            };
            Some((pid.trim().parse().ok()?, responds))
        })
        .collect()
}

impl Responder for PowerShellResponder {
    fn responding(&self, pids: &[u32]) -> Result<HashMap<u32, bool>, String> {
        if !cfg!(windows) || pids.is_empty() {
            return Ok(HashMap::new());
        }
        let args = [
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
        ]
        .map(String::from)
        .into_iter()
        .chain([script(pids)])
        .collect::<Vec<_>>();
        let out = run_hidden("powershell.exe", &args, TIMEOUT)?;
        if out.ok {
            Ok(parse_responding(&out.stdout))
        } else {
            Err(format!(
                "powershell.exe failed: {}",
                out.stderr.trim().lines().next().unwrap_or("")
            ))
        }
    }
}

impl Responder for NoResponder {
    fn responding(&self, _pids: &[u32]) -> Result<HashMap<u32, bool>, String> {
        Ok(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_answer_is_read_per_pid() {
        let map = parse_responding("10=True\r\n11=False\nnoise\n12=maybe\n");
        assert_eq!(map, HashMap::from([(10, true), (11, false)]));
    }

    #[test]
    fn the_script_names_the_pids() {
        assert!(script(&[4, 9]).contains("in 4,9"));
    }
}
