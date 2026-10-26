//! The web build (`apps/web/dist`): embedded in release builds, read from disk in debug builds,
//! or from a folder given at start (tests). Pages are its `index.html` with the placeholders
//! filled and the `coord-scope` meta guaranteed.

use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

use rust_embed::RustEmbed;
use sha1::{Digest, Sha1};

use crate::reply::escape;

#[derive(RustEmbed)]
#[folder = "../../apps/web/dist"]
#[allow_missing = true]
struct Dist;

const FALLBACK: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>__PROJECT__</title></head><body><div id=\"root\"></div></body></html>";

/// The web build.
#[derive(Debug, Clone, Default)]
pub struct WebAssets {
    root: Option<PathBuf>,
}

fn clean(rel: &str) -> Option<&str> {
    let ok = !rel.is_empty()
        && !rel.contains(['\0', '\\'])
        && Path::new(rel)
            .components()
            .all(|c| matches!(c, Component::Normal(_)));
    ok.then_some(rel)
}

impl WebAssets {
    /// The build, from `root` when given, else the embedded or on-disk `apps/web/dist`.
    #[must_use]
    pub fn new(root: Option<PathBuf>) -> Self {
        WebAssets { root }
    }

    /// A file of the build by its relative path.
    #[must_use]
    pub fn file(&self, rel: &str) -> Option<Cow<'static, [u8]>> {
        let rel = clean(rel)?;
        match &self.root {
            Some(root) => {
                let full = root.join(rel);
                full.is_file()
                    .then(|| std::fs::read(full).ok())
                    .flatten()
                    .map(Cow::Owned)
            }
            None => Dist::get(rel).map(|f| f.data),
        }
    }

    /// The page template: the build's `index.html`, or a bare shell when there is no build.
    #[must_use]
    pub fn index(&self) -> String {
        self.file("index.html")
            .and_then(|b| String::from_utf8(b.into_owned()).ok())
            .unwrap_or_else(|| FALLBACK.to_owned())
    }

    /// The build id: the first 12 hex of the sha1 of `index.html`, `None` without a build.
    #[must_use]
    pub fn build_id(&self) -> Option<String> {
        self.file("index.html")
            .map(|b| argus_core::config::hex(&Sha1::digest(&b))[..12].to_owned())
    }
}

/// What a page is rendered with.
pub struct PageVars<'a> {
    /// The board's display name.
    pub name: &'a str,
    /// The scope.
    pub key: &'a str,
    /// `/p/<key>/` or `/`.
    pub base: &'a str,
    /// The server version.
    pub version: &'a str,
}

fn after_head(html: &str) -> Option<usize> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<head")?;
    let rest = &lower[start + 5..];
    if !rest.starts_with(['>', ' ', '\t', '\n', '\r']) {
        return None;
    }
    Some(start + 5 + rest.find('>')? + 1)
}

/// Placeholders replaced and the `coord-scope` meta inserted when the page lacks it.
#[must_use]
pub fn render_page(html: &str, vars: &PageVars) -> String {
    let out = html
        .replace("__PROJECT__", &escape(vars.name))
        .replace("__KEY__", vars.key)
        .replace("__BASE__", vars.base)
        .replace("__DASHBOARD_VERSION__", vars.version);
    let lower = out.to_ascii_lowercase();
    if lower.contains("<meta name=\"coord-scope\"") || lower.contains("<meta name='coord-scope'") {
        return out;
    }
    let tag = format!(
        "<meta name=\"coord-scope\" content=\"{}\">",
        escape(vars.key)
    );
    match after_head(&out) {
        Some(at) => format!("{}{tag}{}", &out[..at], &out[at..]),
        None => format!("{tag}{out}"),
    }
}

/// The content type of a build file, by extension.
#[must_use]
pub fn content_type(rel: &str) -> Option<&'static str> {
    let ext = rel.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())?;
    Some(match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "txt" => "text/plain; charset=utf-8",
        _ => return None,
    })
}
