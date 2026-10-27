//! The command lines `argus` starts itself with, as data. Every way the server comes up (the
//! tray, `argus start`, the stdio MCP bridge) builds its command line here, so the plans are
//! tested without starting anything and without an icon.
//!
//! The rules the plans keep:
//!
//! - The server always comes up through the tray on Windows, as in Node: the bridge and the
//!   tray's own menu start `argus tray start`, which shows the icon and starts the server.
//!   `argus start` alone is the plain server, as `dashboard start` is in Node.
//! - Every plan names the dashboard home with `--data-dir`, so the child never guesses it.
//! - A home that carries the shadow marker also gets `--shadow` on every plan. The flag is
//!   belt and braces: `serve` reads the marker itself and cannot be told otherwise.

use std::ffi::OsString;
use std::path::Path;

use crate::shadow_mark;

/// True when the home carries the shadow marker.
#[must_use]
pub fn is_shadow(home: &Path) -> bool {
    shadow_mark::marked(&home.join("config.json"))
}

fn plan(words: &[&str], home: &Path, extra: Vec<OsString>, shadow: bool) -> Vec<OsString> {
    let mut args: Vec<OsString> = words.iter().map(OsString::from).collect();
    args.extend(extra);
    args.push("--data-dir".into());
    args.push(home.as_os_str().to_owned());
    if shadow {
        args.push("--shadow".into());
    }
    args
}

/// `argus serve [--port n] --data-dir <home> [--shadow]`: the server itself.
#[must_use]
pub fn serve(home: &Path, port: Option<u16>) -> Vec<OsString> {
    let extra = port
        .into_iter()
        .flat_map(|p| [OsString::from("--port"), p.to_string().into()])
        .collect();
    plan(&["serve"], home, extra, is_shadow(home))
}

/// `argus tray start --data-dir <home> [--shadow]`: the tray icon, and with it the server.
#[must_use]
pub fn tray_start(home: &Path) -> Vec<OsString> {
    plan(&["tray", "start"], home, Vec::new(), is_shadow(home))
}

/// `argus tray run --data-dir <home> [--shadow]`: the process that holds the icon.
#[must_use]
pub fn tray_run(home: &Path) -> Vec<OsString> {
    plan(&["tray", "run"], home, Vec::new(), is_shadow(home))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use super::{serve, tray_run, tray_start};

    fn words(args: &[OsString]) -> Vec<String> {
        args.iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    fn home(dir: &tempfile::TempDir, marked: bool) -> std::path::PathBuf {
        let home = dir.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let text = if marked { r#"{"shadow":true}"# } else { "{}" };
        std::fs::write(home.join("config.json"), text).unwrap();
        home
    }

    fn named(home: &Path, head: &[&str], tail: &[&str]) -> Vec<String> {
        let mut all: Vec<String> = head.iter().map(|s| (*s).to_owned()).collect();
        all.push("--data-dir".to_owned());
        all.push(home.to_string_lossy().into_owned());
        all.extend(tail.iter().map(|s| (*s).to_owned()));
        all
    }

    #[test]
    fn a_plain_home_gets_plain_plans_that_name_the_home() {
        let dir = tempfile::tempdir().unwrap();
        let home = home(&dir, false);
        assert_eq!(words(&serve(&home, None)), named(&home, &["serve"], &[]));
        assert_eq!(
            words(&serve(&home, Some(9))),
            named(&home, &["serve", "--port", "9"], &[])
        );
        assert_eq!(
            words(&tray_start(&home)),
            named(&home, &["tray", "start"], &[])
        );
        assert_eq!(words(&tray_run(&home)), named(&home, &["tray", "run"], &[]));
    }

    #[test]
    fn a_shadow_home_gets_the_shadow_flag_on_every_plan() {
        let dir = tempfile::tempdir().unwrap();
        let home = home(&dir, true);
        let flag = ["--shadow"];
        assert_eq!(words(&serve(&home, None)), named(&home, &["serve"], &flag));
        assert_eq!(
            words(&tray_start(&home)),
            named(&home, &["tray", "start"], &flag)
        );
        assert_eq!(
            words(&tray_run(&home)),
            named(&home, &["tray", "run"], &flag)
        );
    }
}
