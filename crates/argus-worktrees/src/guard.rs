//! The allowlists of the git door: what the scan may read and what a removal may write.

fn positional(rest: &[&str]) -> usize {
    rest.iter().filter(|a| !a.starts_with('-')).count()
}

fn config_key(key: &str) -> bool {
    let mut chars = key.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

fn read_command_ok(command: &str, rest: &[&str]) -> bool {
    match command {
        "rev-parse" | "log" | "merge-base" | "rev-list" | "for-each-ref" | "cherry" => true,
        "status" => rest.iter().any(|a| a.starts_with("--porcelain")),
        "symbolic-ref" => positional(rest) == 1,
        "worktree" => rest.first() == Some(&"list"),
        "submodule" => rest == ["status", "--recursive"],
        "remote" => rest.len() == 2 && rest[0] == "get-url" && !rest[1].starts_with('-'),
        "config" => rest.len() == 2 && rest[0] == "--get" && config_key(rest[1]),
        _ => false,
    }
}

/// Only a short list of reading commands may run in the scan. Anything else is refused before a process starts.
pub fn assert_read_only(args: &[&str]) -> Result<(), String> {
    let refuse = || {
        format!(
            "git {} is not a read-only command the worktree scanner may run",
            args.join(" ")
        )
    };
    let Some((command, rest)) = args.split_first() else {
        return Err(refuse());
    };
    if rest.iter().any(|a| a.starts_with("--output")) || !read_command_ok(command, rest) {
        return Err(refuse());
    }
    Ok(())
}

fn worktree_write_ok(rest: &[&str]) -> bool {
    match rest.first() {
        Some(&"remove") => {
            let plain = rest.len() == 2 && !rest[1].starts_with('-');
            let forced = rest.len() == 3 && rest[1] == "--force" && !rest[2].starts_with('-');
            plain || forced
        }
        Some(&"lock") => {
            rest.len() == 4
                && rest[1] == "--reason"
                && !rest[2].is_empty()
                && !rest[3].starts_with('-')
        }
        Some(&"unlock") => rest.len() == 2 && !rest[1].starts_with('-'),
        Some(&"prune") => rest.len() == 1 || rest == ["prune", "--dry-run", "--verbose"],
        _ => false,
    }
}

/// `ls-files --cached -z -- <path>...`: the tracked files at or under the named paths.
fn cached_listing(rest: &[&str]) -> bool {
    rest.len() > 3
        && rest[..3] == ["--cached", "-z", "--"]
        && rest[3..].iter().all(|p| !p.is_empty())
}

fn write_command_ok(command: &str, rest: &[&str]) -> bool {
    match command {
        "worktree" => worktree_write_ok(rest),
        "update-ref" => {
            rest.len() == 3
                && rest[0] == "-d"
                && rest[1].starts_with("refs/heads/")
                && rest[2].len() >= 40
                && rest[2].bytes().all(|b| b.is_ascii_hexdigit())
        }
        "bundle" => {
            (rest.first() == Some(&"create") && rest.len() >= 3)
                || (rest.first() == Some(&"verify") && rest.len() == 2)
        }
        "diff" => {
            rest.first() == Some(&"HEAD")
                && rest.contains(&"--binary")
                && rest.iter().any(|a| a.starts_with("--output="))
        }
        // The two questions asked before Argus clears an ignored `node_modules`, and which links hold tracked paths.
        "check-ignore" => rest == ["-q", "node_modules"],
        "ls-files" => {
            (rest.contains(&"--others") && rest.contains(&"--exclude-standard"))
                || cached_listing(rest)
        }
        "rev-list" => true,
        _ => false,
    }
}

/// Only the removal's own commands may write.
pub fn assert_write_ok(args: &[&str]) -> Result<(), String> {
    match args.split_first() {
        Some((command, rest)) if write_command_ok(command, rest) => Ok(()),
        _ => Err(format!(
            "git {} is not a command the worktree removal may run",
            args.join(" ")
        )),
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_reading_commands_pass() {
        let ok: [&[&str]; 7] = [
            &["rev-parse", "--show-toplevel"],
            &["status", "--porcelain=v1", "-b"],
            &["log", "-1", "--format=%H"],
            &["merge-base", "--is-ancestor", "HEAD", "main"],
            &["symbolic-ref", "-q", "--short", "refs/remotes/origin/HEAD"],
            &["worktree", "list", "--porcelain"],
            &["rev-list", "--count", "main..HEAD"],
        ];
        for args in ok {
            assert!(assert_read_only(args).is_ok(), "{args:?}");
        }
        let bad: [&[&str]; 15] = [
            &["fetch"],
            &["gc"],
            &["prune"],
            &["clean", "-fd"],
            &["checkout", "main"],
            &["reset", "--hard"],
            &["worktree", "remove", "x"],
            &["worktree", "prune"],
            &["worktree", "add", "x"],
            &["symbolic-ref", "HEAD", "refs/heads/x"],
            &["status"],
            &["branch", "-D", "x"],
            &["config", "a", "b"],
            &[],
            &["constructor"],
        ];
        for args in bad {
            let err = assert_read_only(args).unwrap_err();
            assert!(err.contains("not a read-only command"), "{args:?}");
        }
    }

    #[test]
    fn the_newer_reads_pass_and_nothing_that_writes() {
        let ok: [&[&str]; 4] = [
            &[
                "for-each-ref",
                "--contains",
                "HEAD",
                "--count=1",
                "--format=%(refname)",
                "refs/heads",
            ],
            &["cherry", "main", "refs/heads/b"],
            &["remote", "get-url", "origin"],
            &["config", "--get", "init.defaultBranch"],
        ];
        for args in ok {
            assert!(assert_read_only(args).is_ok(), "{args:?}");
        }
        let bad: [&[&str]; 8] = [
            &["remote", "add", "x", "y"],
            &["remote", "set-url", "origin", "z"],
            &["remote", "get-url", "--push"],
            &["config", "--unset", "a.b"],
            &["config", "a.b", "c"],
            &["config", "--get"],
            &["branch", "-D", "x"],
            &["log", "--output=file.txt"],
        ];
        for args in bad {
            assert!(assert_read_only(args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn the_write_door_allows_only_the_removals_own_commands() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let ok: [&[&str]; 14] = [
            &["worktree", "remove", "X:/w"],
            &["worktree", "remove", "--force", "X:/w"],
            &["worktree", "prune"],
            &["worktree", "prune", "--dry-run", "--verbose"],
            &["worktree", "unlock", "X:/w"],
            &["worktree", "lock", "--reason", "held", "X:/w"],
            &["update-ref", "-d", "refs/heads/b", sha],
            &["bundle", "create", "f", "b", "--not", "--remotes"],
            &["bundle", "verify", "f"],
            &["diff", "HEAD", "--binary", "--output=f"],
            &["check-ignore", "-q", "node_modules"],
            &["ls-files", "--cached", "-z", "--", "node_modules"],
            &["ls-files", "--cached", "-z", "--", ":(literal)src/a", ":(literal)b"],
            &["ls-files", "--others", "--exclude-standard", "-z"],
        ];
        for args in ok {
            assert!(assert_write_ok(args).is_ok(), "{args:?}");
        }
        let bad: [&[&str]; 24] = [
            &["worktree", "unlock"],
            &["worktree", "unlock", "--force", "X:/w"],
            &["worktree", "lock", "X:/w"],
            &["worktree", "remove", "--force", "--force", "X:/w"],
            &["worktree", "add", "x"],
            &["branch", "-D", "b"],
            &["branch", "-D", "--all"],
            &["update-ref", "-d", "refs/heads/b"],
            &["update-ref", "-d", "refs/heads/b", "abc"],
            &["update-ref", "-d", "refs/tags/t", sha],
            &["update-ref", "refs/heads/b", sha],
            &["reset", "--hard"],
            &["clean", "-fdx"],
            &["push"],
            &["diff", "HEAD"],
            &["worktree", "prune", "--expire", "now"],
            &["worktree", "prune", "--dry-run"],
            &["worktree", "prune", "--verbose", "--dry-run"],
            &["check-ignore", "-q", "src"],
            &["check-ignore", "node_modules"],
            &["ls-files", "--cached", "-z", "--"],
            &["ls-files", "--cached", "-z", "--", ""],
            &["ls-files", "--cached"],
            &[],
        ];
        for args in bad {
            let err = assert_write_ok(args).unwrap_err();
            assert!(
                err.contains("not a command the worktree removal may run"),
                "{args:?}"
            );
        }
    }
}
