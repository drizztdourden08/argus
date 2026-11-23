//! The `laneAt` stamp: when an item entered the lane it is in. Every write that changes the
//! `status` of an item stamps it, through [`stamp_lane`] or [`set_lane`].

use argus_core::clock::now_iso;
use serde_json::Value;

use crate::json::set;

/// Stamps `laneAt` on `item` when its `status` is no longer `before`.
pub fn stamp_lane(item: &mut Value, before: &Value) {
    if item["status"] != *before {
        set(item, "laneAt", now_iso());
    }
}

/// Sets the `status` of `item` and stamps `laneAt` when it changed.
pub fn set_lane(item: &mut Value, status: &str) {
    let before = item["status"].clone();
    set(item, "status", status);
    stamp_lane(item, &before);
}
