#[path = "support/doors.rs"]
mod doors;
#[path = "worktrees/fixture.rs"]
mod fixture;
mod machine;
mod posting;
#[path = "runs/rig.rs"]
mod runs_rig;
#[allow(dead_code)]
mod toasts;
#[path = "support/wait.rs"]
mod wait;
mod wake_rig;
#[path = "worktrees/rig.rs"]
mod worktrees_rig;

mod activity;
mod agents;
mod alias;
mod cache;
mod doc_stream;
mod early_refusal;
mod first_start;
mod http_security;
mod json_numbers;
mod launcher_overlay;
mod local_names;
mod localhost_door;
mod mcp;
mod notify;
mod owner_text;
mod portfolio_send;
mod posts;
mod pr_checks;
mod procs;
mod records;
mod runs;
mod runs_cli;
mod runs_config;
mod runs_server;
mod runs_stdin;
mod runs_store;
mod shadow;
mod wake;
mod wake_faults;
mod worktree_requests;
mod worktrees;
