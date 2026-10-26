//! The shapes of project keys, launch names, run ids and agent ids.

use std::sync::LazyLock;

use regex::Regex;

use crate::error::RunError;

static KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[a-z0-9][a-z0-9-]*$").unwrap());
static NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[a-z0-9][a-z0-9-]{0,39}$").unwrap());
static RID: LazyLock<Regex> = LazyLock::new(|| Regex::new("^R-[0-9]+$").unwrap());
static AGENT: LazyLock<Regex> = LazyLock::new(|| Regex::new("^A-[0-9]+$").unwrap());

pub fn check_key(key: &str) -> Result<&str, RunError> {
    if KEY.is_match(key) {
        Ok(key)
    } else {
        Err(RunError::bad_request(format!(
            "\"{key}\" is not a project key"
        )))
    }
}

pub fn check_name(name: &str) -> Result<&str, RunError> {
    if NAME.is_match(name) {
        Ok(name)
    } else {
        Err(RunError::bad_request(format!(
            "\"{name}\" is not a launch name (lowercase letters, digits and dashes, up to 40)"
        )))
    }
}

pub fn check_rid(rid: &str) -> Result<&str, RunError> {
    if RID.is_match(rid) {
        Ok(rid)
    } else {
        Err(RunError::bad_request(format!(
            "\"{rid}\" is not a run id (R-<n>)"
        )))
    }
}

pub fn check_agent(agent: &str) -> Result<&str, RunError> {
    if AGENT.is_match(agent) {
        Ok(agent)
    } else {
        Err(RunError::bad_request(format!(
            "\"{agent}\" is not an agent id (A-<n>)"
        )))
    }
}

pub(crate) fn is_launch_name(name: &str) -> bool {
    NAME.is_match(name)
}
