//! Content types of the feed. Only plain data types are named; anything else, a page or a
//! script included, is sent as bytes to download.
// test-tag: critical

const BYTES: &str = "application/octet-stream";

/// The content type of a feed file, by extension.
#[must_use]
pub fn content_type(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".tar.gz") {
        return "application/gzip";
    }
    match lower.rsplit_once('.').map(|(_, ext)| ext) {
        Some("yml" | "yaml") => "text/yaml; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("md") => "text/markdown; charset=utf-8",
        Some("txt" | "log") => "text/plain; charset=utf-8",
        Some("tgz" | "gz") => "application/gzip",
        Some("zip") => "application/zip",
        _ => BYTES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_updater_files_have_their_types_and_the_rest_is_bytes() {
        assert_eq!(content_type("latest.yml"), "text/yaml; charset=utf-8");
        assert_eq!(
            content_type("Brock-Setup-1.0.0.exe"),
            "application/octet-stream"
        );
        assert_eq!(
            content_type("Brock-Setup-1.0.0.exe.blockmap"),
            "application/octet-stream"
        );
        assert_eq!(content_type("core-1.0.0.tgz"), "application/gzip");
        assert_eq!(content_type("core.TAR.GZ"), "application/gzip");
        assert_eq!(
            content_type("release.json"),
            "application/json; charset=utf-8"
        );
        assert_eq!(content_type("page.html"), "application/octet-stream");
        assert_eq!(content_type("noext"), "application/octet-stream");
    }
}
