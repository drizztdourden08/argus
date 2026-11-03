//! What a process runs, in one word: the script or module after the executable.

use crate::text::slashed;

const GENERIC: [&str; 16] = [
    "cli", "index", "main", "bin", "server", "run", "start", "app", "dist", "src", "lib", "out",
    "build", ".", "..", "",
];
const MAX_TOKENS: usize = 12;
const MAX_WORD: usize = 30;
const PATH_TAIL: usize = 3;

fn tokens(cmd: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = cmd.trim_start();
    while !rest.is_empty() && out.len() < MAX_TOKENS {
        let quoted = rest.strip_prefix('"').and_then(|tail| tail.split_once('"'));
        let (token, after) = quoted.unwrap_or_else(|| {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            rest.split_at(end)
        });
        out.push(token.to_string());
        rest = after.trim_start();
    }
    out
}

fn without_extension(word: &str) -> &str {
    match word.rsplit_once('.') {
        Some((stem, ext))
            if (1..=4).contains(&ext.len()) && ext.chars().all(|c| c.is_ascii_alphanumeric()) =>
        {
            stem
        }
        _ => word,
    }
}

fn usable(word: &str) -> bool {
    !GENERIC.contains(&word) && !word.contains(':') && !word.contains('=')
}

fn word_of(token: &str) -> Option<String> {
    let path = slashed(token);
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    parts
        .iter()
        .rev()
        .take(PATH_TAIL)
        .map(|part| without_extension(&part.to_lowercase()).to_string())
        .find(|word| usable(word))
        .map(|word| word.chars().take(MAX_WORD).collect())
}

/// The script or module a command line runs (`vite`, `pnpm`, `tsx`), else `None`.
#[must_use]
pub fn hint_of(cmd: &str) -> Option<String> {
    tokens(cmd)
        .iter()
        .skip(1)
        .filter(|token| !token.is_empty() && !token.starts_with('-'))
        .find_map(|token| word_of(token))
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints_name_the_script() {
        let hint = |cmd| hint_of(cmd);
        assert_eq!(
            hint("node x:/alpha/node_modules/vite/bin/vite.js --port 5173").as_deref(),
            Some("vite")
        );
        assert_eq!(
            hint("node --max-old-space-size=4096 x:/p/node_modules/tsx/dist/cli.mjs watch src/index.ts")
                .as_deref(),
            Some("tsx")
        );
        assert_eq!(
            hint(
                r#""C:\nvm4w\nodejs\node.exe" "C:\nvm4w\nodejs\node_modules\pnpm\bin\pnpm.cjs" dev"#
            )
            .as_deref(),
            Some("pnpm")
        );
        assert_eq!(hint("electron.exe --type=renderer /prefetch:2"), None);
        assert_eq!(hint(""), None);
    }
}
