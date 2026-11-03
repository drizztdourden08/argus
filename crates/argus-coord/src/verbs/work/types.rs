//! `coord types [<type>]`: the item type registry as the board enforces it. Without a type, one
//! line per type; with one, its stages and moves, the rules to enter a stage, its fields with
//! their limits and the flag that sets each, and the rule that must hold to be done.

use std::fmt::Write;

use argus_board::registry::{
    ContentType, DoneRule, Field, ItemType, Mode, Registry, Stage, registry,
};

use crate::cli::out::table;
use crate::cli::{Call, CoordError, Outcome};

fn mode_word(mode: Mode) -> &'static str {
    match mode {
        Mode::Warn => "warns",
        Mode::Refuse => "refuses",
    }
}

fn flow(ty: &ItemType) -> String {
    if ty.stages.is_empty() {
        return ty.statuses.join(" | ");
    }
    let ids: Vec<&str> = ty.stages.iter().map(|s| s.id.as_str()).collect();
    ids.join(" > ")
}

fn parents(ty: &ItemType) -> String {
    if ty.parents.is_empty() {
        "root".to_owned()
    } else {
        ty.parents.join(", ")
    }
}

fn overview(reg: &Registry) -> Vec<String> {
    let mut rows: Vec<Vec<String>> = vec![
        ["type", "label", "parent", "flow"]
            .iter()
            .map(|h| (*h).to_owned())
            .collect(),
    ];
    for ty in &reg.types {
        rows.push(vec![ty.id.clone(), ty.label.clone(), parents(ty), flow(ty)]);
    }
    let kinds: Vec<String> = reg
        .sub_items
        .iter()
        .map(|s| format!("{} {}-n", s.id, s.prefix))
        .collect();
    vec![
        table(&rows),
        format!("Under an item: {}.", kinds.join(", ")),
        "coord types <type> shows the stages, fields and rules of one type.".to_owned(),
    ]
}

fn stage_line(stage: &Stage) -> String {
    let mut line = format!("  {} (lane {})", stage.id, stage.lane);
    if let Some(next) = &stage.next {
        let _ = write!(line, ", then {}", next.join(" | "));
    }
    if !stage.requires.is_empty() {
        let _ = write!(line, ", needs {}", stage.requires.join(", "));
    }
    for gate in &stage.gates {
        let _ = write!(line, ", {} {}", mode_word(gate.mode), gate.need);
    }
    line
}

fn describe(field: &Field) -> String {
    let mut parts: Vec<String> = Vec::new();
    match field.content {
        ContentType::Enum => parts.push(format!("one of {}", field.values.join("|"))),
        ContentType::Ref | ContentType::Refs => parts.push(format!(
            "{} of {}",
            if field.content == ContentType::Refs {
                "ids"
            } else {
                "id"
            },
            field.to.join(", ")
        )),
        other => parts.push(format!("{other:?}").to_lowercase()),
    }
    if let Some(max) = field.max {
        parts.push(format!("max {max}"));
    }
    if let Some(list) = field.list {
        parts.push(format!("up to {list}"));
    }
    if field.required {
        parts.push("required".to_owned());
    }
    parts.join(", ")
}

fn field_line(reg: &Registry, id: &str) -> String {
    let what = reg.field(id).map_or_else(String::new, describe);
    format!("  --{id} {what}")
}

fn done_line(done: &DoneRule) -> String {
    format!(
        "Done {} unless: {}.",
        mode_word(done.mode),
        done.needs.join(", ")
    )
}

fn detail(reg: &Registry, ty: &ItemType) -> Vec<String> {
    let mut lines = vec![format!("{} ({}), parent: {}", ty.id, ty.label, parents(ty))];
    if ty.stages.is_empty() {
        lines.push(format!("Statuses: {}.", ty.statuses.join(", ")));
    } else {
        lines.push("Stages:".to_owned());
        lines.extend(ty.stages.iter().map(stage_line));
    }
    lines.push("Fields (coord item add|set --field k=v, or the flag):".to_owned());
    lines.extend(ty.fields.iter().map(|f| field_line(reg, f)));
    lines.extend(ty.done.as_ref().map(done_line));
    lines
}

fn unknown(reg: &Registry, name: &str) -> CoordError {
    let all: Vec<&str> = reg.types.iter().map(|t| t.id.as_str()).collect();
    CoordError::refuse(format!(
        "no type \"{name}\": the types are {}.",
        all.join(", ")
    ))
}

/// `coord types [<type>]`.
///
/// # Errors
/// Refused for a type the registry does not have.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let reg = registry();
    let lines = match call.positional.first() {
        None => overview(reg),
        Some(name) => match reg.item_type(name) {
            Some(ty) => detail(reg, ty),
            None => return Err(unknown(reg, name)),
        },
    };
    call.inv.say(&[&lines.join("\n")]);
    Ok(())
}
