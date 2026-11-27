//! The command line of the fake app.

use std::path::PathBuf;

/// What the fake was asked to be.
#[derive(Debug, PartialEq)]
pub struct Args {
    /// The app data folder.
    pub app: PathBuf,
    /// The folder with one `<org>.json` per organization: what the Claude servers hold.
    pub remote: PathBuf,
    /// The file whose appearance asks the app to quit.
    pub quit_file: PathBuf,
    /// Stay running when the quit file appears, like Claude does in the system tray.
    pub tray: bool,
    /// When this file exists, remove it and exit before holding the lock: a failed start.
    pub fail_once: Option<PathBuf>,
    /// The organization of each account: `(account, org)`.
    pub orgs: Vec<(String, String)>,
}

impl Args {
    /// The organization of `account`.
    pub fn org_of(&self, account: &str) -> Result<&str, String> {
        self.orgs
            .iter()
            .find(|(a, _)| a == account)
            .map(|(_, org)| org.as_str())
            .ok_or_else(|| format!("no organization is known for the account {account}"))
    }
}

fn value(list: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    list.next().ok_or_else(|| format!("{flag} needs a value"))
}

fn org_pair(text: &str) -> Result<(String, String), String> {
    text.split_once('=')
        .map(|(account, org)| (account.to_owned(), org.to_owned()))
        .ok_or_else(|| format!("--org wants account=org, got {text}"))
}

/// Reads the arguments (without the program name).
///
/// # Errors
/// On an unknown flag, a flag without its value or a missing required flag.
pub fn parse(mut list: impl Iterator<Item = String>) -> Result<Args, String> {
    let (mut app, mut remote, mut quit_file) = (None, None, None);
    let (mut tray, mut fail_once, mut orgs) = (false, None, Vec::new());
    while let Some(flag) = list.next() {
        match flag.as_str() {
            "--app" => app = Some(PathBuf::from(value(&mut list, &flag)?)),
            "--remote" => remote = Some(PathBuf::from(value(&mut list, &flag)?)),
            "--quit-file" => quit_file = Some(PathBuf::from(value(&mut list, &flag)?)),
            "--fail-once" => fail_once = Some(PathBuf::from(value(&mut list, &flag)?)),
            "--org" => orgs.push(org_pair(&value(&mut list, &flag)?)?),
            "--tray" => tray = true,
            other => return Err(format!("unknown flag {other}")),
        }
    }
    let need = |what: Option<PathBuf>, name: &str| what.ok_or_else(|| format!("{name} is needed"));
    Ok(Args {
        app: need(app, "--app")?,
        remote: need(remote, "--remote")?,
        quit_file: need(quit_file, "--quit-file")?,
        tray,
        fail_once,
        orgs,
    })
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::parse;

    fn words(text: &str) -> impl Iterator<Item = String> + '_ {
        text.split_whitespace().map(str::to_owned)
    }

    #[test]
    fn the_flags_make_the_args() {
        let args = parse(words("--app a --remote r --quit-file q --tray --org x=y")).unwrap();
        assert!(args.tray);
        assert_eq!(args.org_of("x"), Ok("y"));
        assert!(args.org_of("z").is_err());
    }

    #[test]
    fn a_missing_or_unknown_flag_is_an_error() {
        assert!(parse(words("--app a")).is_err());
        assert!(parse(words("--app a --remote r --quit-file q --what")).is_err());
        assert!(parse(words("--app")).is_err());
    }
}
