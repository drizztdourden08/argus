//! Checks on the fields, the limits and the content kinds.

use super::super::model::{ContentType, Field, Registry};

/// Every problem in the fields, the base fields, the limits and the rich rule.
pub(super) fn verify(reg: &Registry, out: &mut Vec<String>) {
    if reg.content.rich.rule != "rich-v1" {
        out.push(format!(
            "content.rich: unknown rule {}",
            reg.content.rich.rule
        ));
    }
    if reg.content.rich.image.0.captures_len() < 2 {
        out.push("content.rich.image: needs a group that captures the image number".to_owned());
    }
    for (id, field) in &reg.fields {
        verify_field(reg, id, field, out);
    }
    for name in &reg.base_fields {
        if !reg.fields.contains_key(name) {
            out.push(format!("baseFields: {name} is not a field"));
        }
    }
    for (kind, n) in &reg.limits {
        if *n == 0 {
            out.push(format!("limits.{kind}: must be above zero"));
        }
    }
}

fn verify_field(reg: &Registry, id: &str, field: &Field, out: &mut Vec<String>) {
    let mut found = verify_shape(field);
    found.extend(verify_links(reg, field));
    out.extend(found.into_iter().map(|text| format!("fields.{id}: {text}")));
}

/// The settings that must match the content type of a field.
fn verify_shape(field: &Field) -> Vec<String> {
    let mut found = Vec::new();
    let text = matches!(
        field.content,
        ContentType::Line | ContentType::Slug | ContentType::Rich
    );
    if text != field.max.is_some_and(|n| n > 0) {
        found.push("a max is set exactly on line, slug and rich fields".to_owned());
    }
    if (field.content == ContentType::Rich) != field.images.is_some() {
        found.push("images is set exactly on rich fields".to_owned());
    }
    if (field.content == ContentType::Enum) == field.values.is_empty() {
        found.push("values are set exactly on enum fields".to_owned());
    }
    if !field.labels.is_empty() && field.labels.len() != field.values.len() {
        found.push("labels must match values one to one".to_owned());
    }
    found
}

/// The references, people and lists a field may name.
fn verify_links(reg: &Registry, field: &Field) -> Vec<String> {
    let mut found = Vec::new();
    let refs = matches!(field.content, ContentType::Ref | ContentType::Refs);
    if refs == field.to.is_empty() {
        found.push("to is set exactly on ref and refs fields".to_owned());
    }
    if let Some(kind) = field
        .to
        .iter()
        .find(|k| !reg.content.reference.kinds.contains_key(*k))
    {
        found.push(format!("unknown ref kind {kind}"));
    }
    if field.leads && field.content != ContentType::Person {
        found.push("leads is only for person fields".to_owned());
    }
    let listable = matches!(field.content, ContentType::Line | ContentType::Refs);
    if field.list.is_some() && !listable {
        found.push("list is only for line and refs fields".to_owned());
    }
    found
}
