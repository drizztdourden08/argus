//! The stage an item sits in, and the stage a change asks for.

use serde_json::Value;

use crate::error::{BoardError, refuse};
use crate::ops::target::{ITEM_STATUSES, one_of};
use crate::registry::{ItemType, Stage};

/// The stage of an item: the one it names when that sits in its lane, else the first stage of
/// the lane its status says.
pub(crate) fn current<'a>(ty: &'a ItemType, item: &Value) -> Option<&'a Stage> {
    let lane = item["status"].as_str().unwrap_or("todo");
    item["stage"]
        .as_str()
        .and_then(|s| ty.stage(s))
        .filter(|s| s.lane == lane)
        .or_else(|| ty.first_stage_in(lane))
}

fn names(ty: &ItemType) -> String {
    let all: Vec<&str> = ty.stages.iter().map(|s| s.id.as_str()).collect();
    all.join(", ")
}

fn named<'a>(ty: &'a ItemType, v: &Value) -> Result<&'a Stage, BoardError> {
    v.as_str()
        .and_then(|s| ty.stage(s))
        .ok_or_else(|| refuse(format!("a {} has the stages {}.", ty.id, names(ty))))
}

/// The stage a lane asks for: the stage the item is in when that is in the lane, else the first
/// stage of the lane.
fn in_lane<'a>(ty: &'a ItemType, item: &Value, lane: &Value) -> Result<&'a Stage, BoardError> {
    one_of(&ITEM_STATUSES, lane, "status")?;
    let lane = lane.as_str().unwrap_or_default();
    match current(ty, item) {
        Some(at) if at.lane == lane => Ok(at),
        _ => ty
            .first_stage_in(lane)
            .ok_or_else(|| refuse(format!("a {} has no stage in {lane}.", ty.id))),
    }
}

fn agrees(stage: &Stage, lane: Option<&Value>) -> Result<(), BoardError> {
    match lane.and_then(Value::as_str) {
        Some(lane) if lane != stage.lane => Err(refuse(format!(
            "stage {} is in {}, not in {lane}.",
            stage.id, stage.lane
        ))),
        _ => Ok(()),
    }
}

/// The stage a `stage` or a `status` argument asks for, or `None` when neither is given. A stage
/// wins, and a status that disagrees with it is refused.
pub(crate) fn asked<'a>(
    ty: &'a ItemType,
    item: &Value,
    status: Option<&Value>,
    stage: Option<&Value>,
) -> Result<Option<&'a Stage>, BoardError> {
    let status = status.filter(|v| !v.is_null());
    match (stage.filter(|v| !v.is_null()), status) {
        (Some(wanted), lane) => {
            let found = named(ty, wanted)?;
            agrees(found, lane)?;
            Ok(Some(found))
        }
        (None, Some(lane)) => in_lane(ty, item, lane).map(Some),
        (None, None) => Ok(None),
    }
}
