//! The top-level answers of the Node `dashboard` bin, for a call the shim forwarded: the bare
//! version, the usage text, the `dashboard: unknown command` refusal and their exit codes.
//! The shim marks its call with `ARGUS_FORWARDED_FROM`; a direct `argus` call keeps clap's own
//! output. Anything this module does not answer goes on to the normal parse, and so does a verb
//! only Argus has (`dns`, `self-install`, `parity`): Node is retired, so those must run.

/// The environment variable the Node shim sets on the process it forwards to.
pub const FORWARD_ENV: &str = "ARGUS_FORWARDED_FROM";

/// The usage text of `bin/dashboard.mjs`, byte for byte (the line end is added on print).
const USAGE: &str = "Usage: dashboard <command>
  serve [--port n]            run the server in the foreground
  start [--foreground]        start the server hidden in the background and print its URL
  stop                        stop the server
  status [--json]             health, pid, port, runs (exit 1 when it is down)
  open [<key>] [--print]      open Argus (the overview) or a board in the browser (--print: just print the URL)
  logs [--lines n] [--follow] the server log; with --run R-n --project <key> a run's log
  resync [--project k] [--hard]
  token --rotate              new token (only matters with requireToken in config.json)
  tray [start|stop|status]    system tray icon (Windows): starts the server too; stop closes only the icon
  mcp [--project k]           stdio MCP bridge to the server
  wake-stream --project <key> [--thread <id|*>] [--id <label>]   stay on and print one line per wake for that thread (run it under the Monitor tool with persistent: true)
  install-mcp [--desktop [--write]] [--code [--dir <project dir>]]";

/// The commands the Node bin knows; anything else is refused the Node way.
const VERBS: [&str; 13] = [
    "serve",
    "start",
    "stop",
    "status",
    "open",
    "logs",
    "resync",
    "token",
    "wake-stream",
    "mcp",
    "tray",
    "autostart",
    "install-mcp",
];

/// The commands only Argus has. The Node bin never knew them, so a forwarded call goes straight
/// to the normal parse, `--help` included.
const ARGUS_ONLY: [&str; 3] = ["dns", "self-install", "parity"];

/// The options that take the next argument as their value (`WITH_VALUE` in the Node bin).
const WITH_VALUE: [&str; 5] = ["port", "lines", "project", "run", "thread"];

/// What the Node bin prints and the exit code it ends with.
#[derive(Debug, PartialEq, Eq)]
pub struct Answer {
    pub stdout: String,
    pub stderr: String,
    pub code: u8,
}

fn usage(code: u8) -> Answer {
    Answer {
        stdout: format!("{USAGE}\n"),
        stderr: String::new(),
        code,
    }
}

fn failure(message: &str, code: u8) -> Answer {
    Answer {
        stdout: String::new(),
        stderr: format!("{message}\n"),
        code,
    }
}

/// Whether a `--help` option is on the line, read as the Node parser reads it: a `--name` that
/// takes a value swallows the next argument, and `--help=false` is not a request for help.
fn help_requested(rest: &[String]) -> Result<bool, String> {
    let mut help = false;
    let mut items = rest.iter();
    while let Some(arg) = items.next() {
        let Some(option) = arg.strip_prefix("--") else {
            continue;
        };
        let (name, inline) = option
            .split_once('=')
            .map_or((option, None), |(name, value)| (name, Some(value)));
        if name == "help" {
            help = inline != Some("false");
        } else if WITH_VALUE.contains(&name) && inline.is_none() && items.next().is_none() {
            return Err(format!("dashboard: --{name} needs a value"));
        }
    }
    Ok(help)
}

/// The answer of the Node bin to a command line that only prints (or fails), `None` when the
/// command is a real one and goes on to run.
#[must_use]
pub fn answer(args: &[String], version: &str) -> Option<Answer> {
    let Some((command, rest)) = args.split_first() else {
        return Some(usage(2));
    };
    match command.as_str() {
        "--help" | "-h" | "help" => return Some(usage(0)),
        "--version" | "-v" => {
            return Some(Answer {
                stdout: format!("{version}\n"),
                stderr: String::new(),
                code: 0,
            });
        }
        _ => {}
    }
    if ARGUS_ONLY.contains(&command.as_str()) {
        return None;
    }
    match help_requested(rest) {
        Err(message) => Some(failure(&message, 1)),
        Ok(true) => Some(usage(0)),
        Ok(false) if VERBS.contains(&command.as_str()) => None,
        Ok(false) => Some(failure(
            &format!("dashboard: unknown command \"{command}\"\n{USAGE}"),
            2,
        )),
    }
}

/// Whether this process was started by the Node shim.
#[must_use]
pub fn forwarded() -> bool {
    std::env::var_os(FORWARD_ENV).is_some_and(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{Answer, USAGE, answer};

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    fn code_of(line: &str) -> Option<u8> {
        answer(&args(line), "9.9.9").map(|a| a.code)
    }

    #[test]
    fn the_version_is_bare_for_both_spellings() {
        for line in ["--version", "-v"] {
            let got = answer(&args(line), "9.9.9").expect("an answer");
            assert_eq!(got.stdout, "9.9.9\n");
            assert_eq!((got.stderr.as_str(), got.code), ("", 0));
        }
        assert_eq!(code_of("-V"), Some(2));
    }

    #[test]
    fn usage_goes_to_stdout_with_the_node_codes() {
        let none = answer(&[], "1").expect("an answer");
        assert_eq!(
            (none.stdout.as_str(), none.code),
            (&format!("{USAGE}\n")[..], 2)
        );
        for line in ["--help", "-h", "help", "status --help", "frobnicate --help"] {
            let got = answer(&args(line), "1").expect(line);
            assert_eq!((got.stdout, got.code), (format!("{USAGE}\n"), 0), "{line}");
        }
    }

    #[test]
    fn an_unknown_command_is_named_on_stderr() {
        let got = answer(&args("frobnicate"), "1").expect("an answer");
        let want = Answer {
            stdout: String::new(),
            stderr: format!("dashboard: unknown command \"frobnicate\"\n{USAGE}\n"),
            code: 2,
        };
        assert_eq!(got, want);
    }

    #[test]
    fn a_real_command_goes_on() {
        for line in [
            "serve --port 0",
            "status --json",
            "logs --lines 3",
            "tray stop",
        ] {
            assert_eq!(answer(&args(line), "1"), None, "{line}");
        }
    }

    #[test]
    fn an_argus_only_verb_goes_on_to_the_normal_parse() {
        for line in [
            "dns status",
            "dns --help",
            "self-install --print",
            "parity --help",
            "install-mcp --code",
            "autostart status",
            "tray status",
        ] {
            assert_eq!(answer(&args(line), "1"), None, "{line}");
        }
        assert_eq!(code_of("dnsx status"), Some(2));
    }

    #[test]
    fn help_is_read_the_way_the_node_parser_reads_it() {
        assert_eq!(code_of("logs --lines --help"), None);
        assert_eq!(code_of("status --help=false"), None);
        assert_eq!(code_of("status --help=true"), Some(0));
        assert_eq!(code_of("serve --port"), Some(1));
    }
}
