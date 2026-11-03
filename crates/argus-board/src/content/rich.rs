//! The `rich` kind, rule rich-v1: a short lead line, bullets and images, read line by line.
//!
//! A line is an image line, a bullet line or plain text. Plain text is allowed only as the first
//! line. Headings, tables, quotes, code fences and raw HTML are refused anywhere. A single short
//! line is accepted as it is; anything longer needs a bullet or an image, so a lone paragraph is
//! refused. The text length counts what is left after the image lines and the bullet markers.

use argus_core::js;
use serde_json::Value;

use super::{Context, Rejection, reject};
use crate::registry::{Field, RichKind, registry};

const LONE_PARAGRAPH: &str = "Write it as bullets (- point) or add a screenshot; a lone paragraph is not accepted. Example: Short lead line, then - first point and - second point, one per line.";

/// One non-blank line of a rich text.
enum Line<'a> {
    /// A bullet; holds the text after the marker.
    Bullet(&'a str),
    /// An image line; holds its number, `None` when too large to be one.
    Image(Option<u32>),
    /// Anything else.
    Plain(&'a str),
}

/// The trimmed text, when it follows rule rich-v1 for this field.
pub(super) fn check(
    field: &str,
    spec: &Field,
    text: &str,
    cx: &Context<'_>,
) -> Result<Value, Rejection> {
    let rule = &registry().content.rich;
    let raw: Vec<&str> = text
        .lines()
        .map(str::trim_end)
        .filter(|l| !js::trim(l).is_empty())
        .collect();
    refuse_forbidden(field, &raw, rule)?;
    let lines: Vec<Line<'_>> = raw.iter().map(|l| classify(l, rule)).collect();
    refuse_lone(&lines, rule)?;
    refuse_plain(field, &lines, rule)?;
    refuse_bullets(field, &lines, rule)?;
    refuse_images(field, spec, &lines, cx)?;
    refuse_long(field, spec, &lines)?;
    Ok(Value::String(js::trim(text).to_owned()))
}

fn classify<'a>(line: &'a str, rule: &RichKind) -> Line<'a> {
    let trimmed = js::trim(line);
    if let Some(caps) = rule.image.0.captures(trimmed) {
        return Line::Image(caps.get(1).and_then(|m| m.as_str().parse().ok()));
    }
    if rule.bullet.0.is_match(line) {
        let after = trimmed.split_once(' ').map_or("", |(_, rest)| rest);
        return Line::Bullet(js::trim(after));
    }
    Line::Plain(trimmed)
}

fn refuse_forbidden(field: &str, raw: &[&str], rule: &RichKind) -> Result<(), Rejection> {
    for banned in &rule.forbidden {
        if raw.iter().any(|l| banned.pattern.0.is_match(l)) {
            return Err(reject(
                "rich-forbidden",
                format!(
                    "{field} cannot hold a {}. Use bullets (- point) and images.",
                    banned.name
                ),
            ));
        }
    }
    Ok(())
}

fn refuse_lone(lines: &[Line<'_>], rule: &RichKind) -> Result<(), Rejection> {
    let marked = lines.iter().any(|l| !matches!(l, Line::Plain(_)));
    let short = matches!(lines, [Line::Plain(t)] if js::len(t) <= rule.short_max);
    if marked || short {
        Ok(())
    } else {
        Err(reject("rich-lone-paragraph", LONE_PARAGRAPH))
    }
}

fn refuse_plain(field: &str, lines: &[Line<'_>], rule: &RichKind) -> Result<(), Rejection> {
    for (i, line) in lines.iter().enumerate() {
        let Line::Plain(text) = line else { continue };
        if i > 0 {
            return Err(reject(
                "rich-lead-late",
                format!(
                    "{field}: only the first line may be plain text; write the rest as bullets (- point), nested at most two levels, or as images ![caption](img:n)."
                ),
            ));
        }
        if js::len(text) > rule.lead_max {
            return Err(reject(
                "rich-lead-long",
                format!(
                    "{field}: the lead line is {} chars, the limit is {}. Shorten it or make it a bullet.",
                    js::len(text),
                    rule.lead_max
                ),
            ));
        }
    }
    Ok(())
}

fn refuse_bullets(field: &str, lines: &[Line<'_>], rule: &RichKind) -> Result<(), Rejection> {
    let bullets: Vec<&str> = lines
        .iter()
        .filter_map(|l| {
            if let Line::Bullet(t) = l {
                Some(*t)
            } else {
                None
            }
        })
        .collect();
    if bullets.len() > rule.bullets {
        return Err(reject(
            "rich-bullets-many",
            format!(
                "{field} has {} bullets, the limit is {}. Merge some.",
                bullets.len(),
                rule.bullets
            ),
        ));
    }
    if let Some(long) = bullets.iter().find(|t| js::len(t) > rule.bullet_max) {
        return Err(reject(
            "rich-bullet-long",
            format!(
                "{field}: a bullet is {} chars, the limit is {}. Shorten it.",
                js::len(long),
                rule.bullet_max
            ),
        ));
    }
    Ok(())
}

fn refuse_images(
    field: &str,
    spec: &Field,
    lines: &[Line<'_>],
    cx: &Context<'_>,
) -> Result<(), Rejection> {
    let images: Vec<Option<u32>> = lines
        .iter()
        .filter_map(|l| {
            if let Line::Image(n) = l {
                Some(*n)
            } else {
                None
            }
        })
        .collect();
    let max = spec.images.unwrap_or(0);
    if images.len() > max {
        return Err(reject(
            "rich-image-many",
            format!("{field} has {} images, the limit is {max}.", images.len()),
        ));
    }
    if images
        .iter()
        .any(|n| n.is_none_or(|n| !cx.images.contains(&n)))
    {
        return Err(reject(
            "rich-image-unknown",
            format!(
                "{field} names an image this record does not have. Add the image first, then use the number it gets."
            ),
        ));
    }
    Ok(())
}

fn refuse_long(field: &str, spec: &Field, lines: &[Line<'_>]) -> Result<(), Rejection> {
    let total: usize = lines
        .iter()
        .map(|l| match l {
            Line::Bullet(t) | Line::Plain(t) => js::len(t),
            Line::Image(_) => 0,
        })
        .sum();
    match spec.max {
        Some(max) if total > max => Err(reject(
            "rich-text-long",
            format!(
                "{field} has {total} chars of text, the limit is {max}. Shorten it; images do not count."
            ),
        )),
        _ => Ok(()),
    }
}
