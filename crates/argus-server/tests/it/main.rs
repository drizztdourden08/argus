#[path = "support/cache_rows.rs"]
mod cache_rows;
#[path = "support/doors.rs"]
mod doors;
#[path = "worktrees/fixture.rs"]
mod fixture;
mod machine;
#[path = "support/mux_rig.rs"]
mod mux_rig;
#[path = "support/node_home.rs"]
mod node_home;
#[path = "support/page_send.rs"]
mod page_send;
mod posting;
mod pr_rig;
#[path = "runs/rig.rs"]
mod runs_rig;
mod toasts;
#[path = "support/wait.rs"]
mod wait;
mod wake_rig;
#[path = "worktrees/rig.rs"]
mod worktrees_rig;

mod activity;
mod agents;
mod alias;
mod alias_rules;
mod board_icon;
mod cache;
mod cache_worktree;
mod deps;
mod doc_stream;
mod early_refusal;
mod first_start;
mod http_rules;
mod http_security;
mod item_images;
mod json_numbers;
mod launcher_overlay;
mod local_names;
mod local_names_rules;
mod localhost_door;
mod localhost_door_rules;
mod mcp;
mod mux_lifecycle;
mod mux_limits;
mod mux_presence;
mod mux_stream;
mod mux_tails;
mod notify;
mod owner_text;
mod portfolio_send;
mod portfolio_send_cli;
mod posts;
mod pr_checks;
mod pr_checks_runs;
mod preview;
mod procs;
mod records;
mod records_node;
mod runs;
mod runs_cli;
mod runs_config;
mod runs_server;
mod runs_stdin;
mod runs_store;
mod shadow;
mod shadow_server;
mod sse_buffer;
mod stalls;
mod trail;
mod wake;
mod wake_faults;
mod wake_owner_alert;
mod wake_run_complete;
mod worktree_requests;
mod worktrees;
