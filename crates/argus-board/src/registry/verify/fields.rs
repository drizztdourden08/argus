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
    let mut say = |text: &str| out.push(format!("fields.{id}: {text}"));
    let text = matches!(
        field.content,
        ContentType::Line | ContentType::Slug | ContentType::Rich
    );
    if text != field.max.is_some_and(|n| n > 0) {
        say("a max is set exactly on line, slug and rich fields");
    }
    if (field.content == ContentType::Rich) != field.images.is_some() {
        say("images is set exactly on rich fields");
    }
    if (field.content == ContentType::Enum) == field.values.is_empty() {
        say("values are set exactly on enum fields");
    }
    if !field.labels.is_empty() && field.labels.len() != field.values.len() {
        say("labels must match values one to one");
    }
    let refs = matches!(field.content, ContentType::Ref | ContentType::Refs);
    if refs == field.to.is_empty() {
        say("to is set exactly on ref and refs fields");
    }
    if let Some(kind) = field
        .to
        .iter()
        .find(|k| !reg.content.reference.kinds.contains_key(*k))
    {
        say(&format!("unknown ref kind {kind}"));
    }
    if field.leads && field.content != ContentType::Person {
        say("leads is only for person fields");
    }
    let listable = matches!(field.content, ContentType::Line | ContentType::Refs);
    if field.list.is_some() && !listable {
        say("list is only for line and refs fields");
    }
}
