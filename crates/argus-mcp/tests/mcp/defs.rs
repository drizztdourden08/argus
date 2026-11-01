//! The tool definitions.
//! K2: the tool definitions are the contract of the MCP layer.
// test-tag: critical

use std::collections::HashSet;

use argus_mcp::{is_mutating, tool_def, tool_defs};

#[test]
fn there_are_23_tools_with_unique_names_object_schemas_and_annotations() {
    let defs = tool_defs();
    assert_eq!(defs.len(), 23);
    let names: HashSet<&str> = defs.iter().filter_map(|d| d["name"].as_str()).collect();
    assert_eq!(names.len(), 23);
    for d in defs {
        let name = d["name"].as_str().expect("a name");
        assert_eq!(d["inputSchema"]["type"], "object", "{name}");
        assert_eq!(d["inputSchema"]["additionalProperties"], false, "{name}");
        let described = d["description"].as_str().is_some_and(|t| t.len() > 20);
        assert!(described && d["title"].is_string(), "{name}");
        assert!(d["annotations"]["readOnlyHint"].is_boolean(), "{name}");
        if d["annotations"]["readOnlyHint"] == false {
            assert!(!d["annotations"]["destructiveHint"].is_null(), "{name}");
        }
    }
    let read_only = defs
        .iter()
        .filter(|d| d["annotations"]["readOnlyHint"] == true);
    assert_eq!(read_only.count(), 14);
    assert_eq!(
        tool_def("run_stop").expect("run_stop")["annotations"]["destructiveHint"],
        true
    );
    assert_eq!(
        tool_def("run_start").expect("run_start")["annotations"]["openWorldHint"],
        true
    );
}

#[test]
fn mutating_means_not_read_only() {
    assert!(is_mutating("post_entry"));
    assert!(is_mutating("notify_owner"));
    assert!(!is_mutating("board_summary"));
    assert!(!is_mutating("no_such_tool"));
}

#[test]
fn every_property_schema_is_typed_and_every_required_name_is_a_property() {
    for d in tool_defs() {
        let schema = &d["inputSchema"];
        let props = schema["properties"].as_object().expect("properties");
        for required in schema["required"].as_array().into_iter().flatten() {
            let key = required.as_str().expect("a name");
            assert!(props.contains_key(key), "{}: {key}", d["name"]);
        }
        for (key, prop) in props {
            assert!(prop["type"].is_string(), "{}: {key}", d["name"]);
        }
    }
}
