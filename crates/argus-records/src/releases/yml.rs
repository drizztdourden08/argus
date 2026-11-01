//! electron-builder's `latest.yml`, read as lines: the version it names, and a copy whose file
//! urls point into a version folder. Only the `url` and `path` lines change; the hashes, sizes
//! and notes stay as electron-builder wrote them.
// test-tag: critical

fn unquote(value: &str) -> (&str, &str) {
    for quote in ["'", "\""] {
        if let Some(inner) = value
            .strip_prefix(quote)
            .and_then(|v| v.strip_suffix(quote))
        {
            return (quote, inner);
        }
    }
    ("", value)
}

fn rewrite_line(line: &str, version: &str) -> String {
    let body = line.trim_start();
    let indent = &line[..line.len() - body.len()];
    let (dash, rest) = body
        .strip_prefix("- ")
        .map_or(("", body), |r| ("- ", r.trim_start()));
    let Some((key, value)) = rest.split_once(':') else {
        return line.to_owned();
    };
    let (quote, inner) = unquote(value.trim());
    let base = inner.rsplit(['/', '\\']).next().unwrap_or_default();
    if !(key == "url" || key == "path") || base.is_empty() {
        return line.to_owned();
    }
    format!("{indent}{dash}{key}: {quote}{version}/{base}{quote}")
}

/// The text with every `url:` and `path:` value rewritten to `<version>/<file name>`.
#[must_use]
pub fn rewrite_urls(text: &str, version: &str) -> String {
    let mut out: Vec<String> = text.lines().map(|l| rewrite_line(l, version)).collect();
    out.push(String::new());
    out.join("\n")
}

/// The top level `version:` of the file.
#[must_use]
pub fn version_of(text: &str) -> Option<String> {
    let value = text.lines().find_map(|l| l.strip_prefix("version:"))?;
    let (_, inner) = unquote(value.trim());
    (!inner.is_empty()).then(|| inner.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LATEST: &str = "version: 1.2.0\nfiles:\n  - url: Brock-Setup-1.2.0.exe\n    sha512: abc==\n    size: 10\n  - url: 'https://github.com/o/r/releases/download/v1.2.0/Brock Mac.dmg'\n    sha512: def==\npath: Brock-Setup-1.2.0.exe\nsha512: abc==\nreleaseDate: '2026-10-29T10:00:00.000Z'\n";

    #[test]
    fn urls_and_paths_point_into_the_version_folder_and_the_rest_is_kept() {
        let out = rewrite_urls(LATEST, "1.2.0");
        assert!(out.contains("  - url: 1.2.0/Brock-Setup-1.2.0.exe\n    sha512: abc==\n"));
        assert!(out.contains("  - url: '1.2.0/Brock Mac.dmg'\n"));
        assert!(out.contains("\npath: 1.2.0/Brock-Setup-1.2.0.exe\nsha512: abc==\n"));
        assert!(out.contains("releaseDate: '2026-10-29T10:00:00.000Z'"));
        assert_eq!(rewrite_urls(&out, "1.2.0"), out, "idempotent");
    }

    #[test]
    fn the_version_is_the_top_level_line() {
        assert_eq!(version_of(LATEST).as_deref(), Some("1.2.0"));
        assert_eq!(
            version_of("version: '2.0.0-beta.1'\n").as_deref(),
            Some("2.0.0-beta.1")
        );
        assert_eq!(version_of("files:\n  version: 3\n"), None);
    }
}
