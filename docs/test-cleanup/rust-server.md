# Test clean-up: rust-server (argus-server, argus, argus-mcp)

Baseline `3a35cac`. Rules from `X:/ai-config/docs/test-standard.md`. Tags follow the Rust tag convention: `// test-tag:` once per file (above `#[cfg(test)]` for inline modules); e2e cases also carry `#[ignore = "e2e"]`.

## Counts

| Crate | Files before | Cases before | Files after | Cases after | critical (files/cases) | standard (files/cases) | e2e (files/cases) | untagged (files/cases) |
|---|---|---|---|---|---|---|---|---|
| argus-server | 35 | 325 | 45 | 298 | 9/33 | 24/206 | 6/17 | 6/42 |
| argus | 19 | 99 | 19 | 84 | 9/45 | 3/10 | 7/29 | 0/0 |
| argus-mcp | 11 | 80 | 12 | 79 | 9/69 | 1/7 | 2/3 | 0/0 |

Untagged in argus-server: `wake.rs`, `wake_faults.rs`, `worktrees.rs`, `worktree_requests.rs` and the inline tests of `adapters/worktrees/links.rs` and `routes/worktrees/requests.rs`. A separate agent (R3, worktrees and wake) was running on them, so this part did not edit them, to avoid a merge clash. The wake and worktrees rig helpers (`wake_rig`, `worktrees/fixture.rs`, `worktrees/rig.rs`) keep their `#![allow(dead_code)]` for the same reason.

## Verdicts

### argus-server

| File | Cases | Verdict | Rule | Tag |
|---|---|---|---|---|
| argus-server/src/adapters/proc_folders.rs | 1 | KEEP | K1: the porcelain list and the rule paths parse to their folders. | critical |
| argus-server/src/adapters/worktrees/links.rs | 1 | NOT TOUCHED (R3 agent owns worktrees and wake) |  | NONE |
| argus-server/src/routes/check_guard.rs | 2 | KEEP | K1: which paths a guard run changed, and the note that names them. | critical |
| argus-server/src/routes/worktrees/requests.rs | 1 | NOT TOUCHED (R3 agent owns worktrees and wake) |  | NONE |
| argus-server/tests/it/activity.rs | 6 | KEEP | K2, K3: the activity route shape, the portfolio docs and the sink adapters. | standard |
| argus-server/tests/it/agents.rs | 6 | KEEP | K2, S2: the reconcile report and the reaper through the sync service. | standard |
| argus-server/tests/it/alias.rs | 8 | KEEP | K2, K4: the redirect and overview answers and the host and origin refusals; the pure parse rules are in `alias | standard |
| argus-server/tests/it/alias_rules.rs | 3 | KEEP | K1, K4: the alias host rules in isolation: which hosts parse as a board alias (with the | critical |
| argus-server/tests/it/cache.rs | 5 | KEEP | K2, K3: the listing shape and the prune; the worktree and lockfile case is in `cache_worktree. | standard |
| argus-server/tests/it/cache_worktree.rs | 1 | KEEP | K3, E2: a worktree of a project keeps the build cache entries its lockfile resolves, on a real | e2e |
| argus-server/tests/it/doc_stream.rs | 15 | KEEP | K2, S3: the wire format and the resume rules of the page stream; the buffer limit of one | standard |
| argus-server/tests/it/early_refusal.rs | 1 | KEEP | K4, K5: a refusal sent before the body is read reaches the client. | standard |
| argus-server/tests/it/first_start.rs | 3 | KEEP | K3: the one backup of the Node database before the first write. | critical |
| argus-server/tests/it/http_rules.rs | 11 | KEEP | K1, K2, K4: the pure rules of the HTTP layer with no server: token and cookie compare, login | critical |
| argus-server/tests/it/http_security.rs | 35 | KEEP | K2, K4: the refusal matrix, the token mode and the file routes; the pure rules are in `http_rules. | standard |
| argus-server/tests/it/json_numbers.rs | 2 | KEEP | K2: whole numbers carry no fraction in docs, answers and the page stream. | standard |
| argus-server/tests/it/launcher_overlay.rs | 1 | KEEP | K2: the stale marks of a launchers doc. | standard |
| argus-server/tests/it/local_names.rs | 8 | KEEP | K2, K4, S3: the responder, the front door and its host refusals on sockets of port 0; the pure host rules are  | standard |
| argus-server/tests/it/local_names_rules.rs | 3 | KEEP | K1, K4: the host rules of the local names with no socket: a name under the suffix is an alias | critical |
| argus-server/tests/it/localhost_door.rs | 6 | KEEP | K2, K4, S3: the localhost door on a socket of port 0; the pure rules are in localhost_door_rules.rs. | standard |
| argus-server/tests/it/localhost_door_rules.rs | 2 | KEEP | K1, K4: the host and origin rules of the localhost door at port 80, where the Host carries no | critical |
| argus-server/tests/it/mcp.rs | 18 | KEEP | K2, K4: the MCP layer on a live server: auth and refusals, tool results, owner writes and the policy. | standard |
| argus-server/tests/it/notify.rs | 18 | KEEP | K1, K2, K4: the notification routes, limits and automatic pings, with a manual clock and a worker that takes e | standard |
| argus-server/tests/it/owner_text.rs | 5 | KEEP | K1, K3, K4: the owner text limit of 50000 on a page post and the short limits of the CLI source. | standard |
| argus-server/tests/it/portfolio_send.rs | 4 | KEEP | K3, K4: the owner send of the Overlord page, whole to the lead, with the 50000 net and the short limits of the | standard |
| argus-server/tests/it/portfolio_send_cli.rs | 1 | KEEP | E2: the lead reads a 20000 character send of the Overlord page whole with the real Node | e2e |
| argus-server/tests/it/posts.rs | 18 | KEEP | K2, K3, K4: owner posts over HTTP, idempotent ids, the result-note rule, uploads and shadow mode. | standard |
| argus-server/tests/it/pr_checks.rs | 4 | KEEP | K2, K4: the route shapes and the refusals. | standard |
| argus-server/tests/it/pr_checks_runs.rs | 6 | KEEP | E2: check runs on the run engine with a fake `brock` that is a real Node process, and a real | e2e |
| argus-server/tests/it/procs.rs | 10 | KEEP | K2, K4: the process monitor routes over a fake machine, its rules and the kill guards. | standard |
| argus-server/tests/it/records.rs | 12 | KEEP | K2, K3, K4: the local remote, the release feed, the check runs and the registry over HTTP; the Node status scr | standard |
| argus-server/tests/it/records_node.rs | 1 | KEEP | E2: the registry's Node status script run for real with the registry home, and the failure of a | e2e |
| argus-server/tests/it/runs.rs | 9 | KEEP | K2, K3, K4: the run routes, logs, streams, attachments and launches over a fake process host. | standard |
| argus-server/tests/it/runs_cli.rs | 4 | KEEP | E2: the real Node coord run client against a server with real processes. | e2e |
| argus-server/tests/it/runs_config.rs | 2 | KEEP | K2: the live config limits of the run service. | standard |
| argus-server/tests/it/runs_server.rs | 4 | KEEP | E1, E2: the run service in the whole server with real processes: a Play, a failing launch, Stop on the process | e2e |
| argus-server/tests/it/runs_stdin.rs | 5 | KEEP | K2, K4: the stdin route of a run and its refusals. | standard |
| argus-server/tests/it/runs_store.rs | 5 | KEEP | K3, K1: the run rows in the database over a fake process host: crash recovery, retention, the log cap and the  | standard |
| argus-server/tests/it/shadow.rs | 6 | KEEP | K3, K1: seeding a shadow home from the Node home in temp folders: nothing the shadow has is overwritten and th | critical |
| argus-server/tests/it/shadow_server.rs | 3 | KEEP | K3, K4: a server in shadow mode: it starts from the Node database, a marked home serves in | standard |
| argus-server/tests/it/sse_buffer.rs | 2 | KEEP | K1, S3: the buffer limit of one page stream client, with no server: a client a megabyte behind | critical |
| argus-server/tests/it/wake.rs | 15 | NOT TOUCHED (R3 agent owns worktrees and wake) |  | NONE |
| argus-server/tests/it/wake_faults.rs | 6 | NOT TOUCHED (R3 agent owns worktrees and wake) |  | NONE |
| argus-server/tests/it/worktree_requests.rs | 4 | NOT TOUCHED (R3 agent owns worktrees and wake) |  | NONE |
| argus-server/tests/it/worktrees.rs | 15 | NOT TOUCHED (R3 agent owns worktrees and wake) |  | NONE |

### argus

| File | Cases | Verdict | Rule | Tag |
|---|---|---|---|---|
| argus/src/install/replace.rs | 5 | KEEP | K3: replacing the installed binary never loses the old copy. | critical |
| argus/src/install/replace/link_tests.rs | 1 | KEEP | K3, K4: an old copy that is a junction or a symlink goes as a link and its target stays. | e2e |
| argus/src/lifecycle/dns/script.rs | 3 | KEEP | K2, K4: the printed NRPT script is the one rule and nothing else. | critical |
| argus/src/lifecycle/dns/status.rs | 6 | KEEP | K2: the lines of argus dns status. | critical |
| argus/src/lifecycle/serve.rs | 1 | KEEP | K1: the order of the port flag, the environment and the config. | critical |
| argus/src/main.rs | 3 | KEEP | K2: the verbs and flags of the argus command line. | critical |
| argus/src/mcp/launch.rs | 1 | KEEP | K2: the tray start words of a home, and the shadow flag. | critical |
| argus/src/node_form.rs | 6 | KEEP | K1, K2: the Node command line, answered the way the Node parser answers it. | critical |
| argus/src/tray/menu.rs | 4 | KEEP | S2: the tray menu commands against an in-process server. | standard |
| argus/src/tray/server.rs | 2 | KEEP | S2: the tray reads the server state from a home and an in-process server. | standard |
| argus/tests/it/cli.rs | 10 | KEEP | E1, E2, K2, K4: the server verbs of the real binary: start, status, stop, tokens, the data dir and the Node gu | e2e |
| argus/tests/it/coord_fast_path.rs | 2 | KEEP | E2, K2: help and an unknown verb of the real binary. | e2e |
| argus/tests/it/dns_cli.rs | 5 | KEEP | E2, K2, K4: the dns verbs of the real binary print scripts and run nothing. | e2e |
| argus/tests/it/install.rs | 18 | KEEP | K2, K3: the install-mcp, autostart and self-install verbs, their files and their refusals. | critical |
| argus/tests/it/mcp_stdio.rs | 2 | KEEP | K1, K2: the config and boot rules of the dashboard environment. | critical |
| argus/tests/it/mcp_stdio_process.rs | 2 | KEEP | E2, K2: the real `argus mcp` process answers locally with no config, exits when stdin ends, | e2e |
| argus/tests/it/node_form.rs | 2 | KEEP | E2, K2: the real binary answers a forwarded call with the Node text. | e2e |
| argus/tests/it/tray_cli.rs | 7 | KEEP | E2, K2: the tray verbs of the real binary, with no tray started. | e2e |
| argus/tests/it/wake_stream.rs | 4 | KEEP | S2, K2: one stdout line per wake, the reconnect, the replaced stream and a broken stdout. | standard |

### argus-mcp

| File | Cases | Verdict | Rule | Tag |
|---|---|---|---|---|
| argus-mcp/tests/mcp/bridge.rs | 7 | KEEP | K2: the stdio bridge on its in-memory wire: local answers, the start rules, the quit flag. | critical |
| argus-mcp/tests/mcp/bridge_stub.rs | 7 | KEEP | S2, K2: the stdio bridge against a local HTTP stub that mounts the real handler: a call is | standard |
| argus-mcp/tests/mcp/defs.rs | 3 | KEEP | K2: the tool definitions are the contract of the MCP layer. | critical |
| argus-mcp/tests/mcp/http.rs | 16 | KEEP | K2, K4: the /mcp framing, envelope and refusals. | critical |
| argus-mcp/tests/mcp/node_parity.rs | 2 | KEEP | E3: the tool list and initialize text against the Node server. | e2e |
| argus-mcp/tests/mcp/node_results.rs | 1 | KEEP | E3: result texts against the Node tool host. | e2e |
| argus-mcp/tests/mcp/notify.rs | 4 | KEEP | K2, K4: notify_owner speaks to the owner and refuses bad arguments. | critical |
| argus-mcp/tests/mcp/policy.rs | 3 | KEEP | K4: deny, allow and readOnly decide which tools run. | critical |
| argus-mcp/tests/mcp/proof.rs | 6 | KEEP | K4, K2: attach_proof sources, magic bytes, allowed folders and the size cap. | critical |
| argus-mcp/tests/mcp/read_tools.rs | 19 | KEEP | K2: the shape of every read tool. | critical |
| argus-mcp/tests/mcp/resources.rs | 5 | KEEP | K2: the resource list and reads. | critical |
| argus-mcp/tests/mcp/write_tools.rs | 6 | KEEP | K2, K4: the tools that act as the owner and their refusals. | critical |

## Deleted cases (41 names) and why

Reasons are one line each. Where a case looked like K1 to K4 it says so ("K4 by look"). None was a K5 regression.

### argus-server

| Case | Rule | Reason |
|---|---|---|
| alias.rs reserved_aliases_follow_the_same_origin_rule_as_every_alias | W4 | The origin refusal of an alias run again for the reserved names (K4 by look). The kept alias route test and `alias_rules.rs` hold it. |
| alias.rs reserved_names_are_reserved_on_our_port_only | W4 | Same host rule as the kept unit rules, a second table of hosts. |
| alias.rs reserved_aliases_pass_the_host_check_on_our_port_and_nowhere_else | W4 | The host check again over HTTP; the pure rule is kept in `alias_rules.rs`. |
| alias.rs reserved_hosts_and_origins_in_the_unit_rules | W4 | Overlapped the kept parse and origin rule cases. |
| alias.rs the_redirect_target_and_the_api_are_served_on_the_alias_host | W4 | The alias host served pages already, covered by the kept redirect test. |
| alias.rs overlord_alias_edge_cases_and_the_board_page_name | W4 | Edge states of the same alias routing, one behavior in different states. |
| doc_stream.rs a_two_megabyte_snapshot_over_several_collections_arrives_whole_with_ready | W7 | A size and time bound stream read; the buffer rule is kept deterministic in `sse_buffer.rs`. |
| early_refusal.rs the_shutdown_answers_a_client_whose_body_comes_late | W7 | Repeated five times and ordering dependent (a late body); the kept test of the file covers the refusal. |
| http_security.rs pages_files_streams_and_static_files_are_open_too | W4 | Open GET routes in the default mode; one state of the matrix the kept tests already walk. |
| http_security.rs token_mode_login_next_is_always_a_local_path | W4 | Same local-path rule as `token_compare_cookies_and_safe_next` in `http_rules.rs` (K4 by look). |
| http_security.rs no_cors_headers_on_any_response | W5 | A framework default (no layer adds CORS); one header assert. |
| http_security.rs post_routes_follow_the_same_rules_and_a_bearer_does_not_lift_them | W4 | The write refusal matrix again on one more route (K4 by look); the kept matrix test stays. |
| http_security.rs db_batch_stops_at_the_first_error | W4 | Batch semantics covered by the records and posts tests. |
| http_security.rs resync_is_unavailable_without_sync | W4 | A 503 with the service off, a state of `resync_is_delegated_to_the_sync_service`. |
| http_security.rs assets_are_cached_for_a_year | W5 | One cache-control constant. |
| mcp.rs a_refused_post_is_an_error_result_or_a_refusal_with_the_board_reason | W8 | The assertion accepted "applied", an error or a refusal, so it could not fail on the behavior. |
| owner_text.rs the_safety_net_is_50000_characters | W5 | Asserts a constant. |
| owner_text.rs answers_and_decisions_from_the_page_are_not_cut_at_240_or_200 | W4 | The same no-truncation fact as the 20000 character result note test, for two more kinds. |
| portfolio_send.rs coord_portfolio_send_refuses_over_240_and_decisions_over_200 | W4 | Spawned the coord CLI for a length refusal the route test already holds. |
| posts.rs the_same_id_of_a_refused_result_gives_the_stored_refusal | W4 | The idempotent replay is held by the applied case; the refused variant repeats it. |
| posts.rs the_page_path_put_of_entries_is_a_post | W4 | The page route is exercised by the kept page posts. |
| posts.rs a_proof_that_is_not_an_image_type_is_refused | W4 | The proof refusals are kept in the `attach_proof` tests of argus-mcp (K4 by look). |
| posts.rs a_page_post_to_all_projects_is_delivered_to_each | W4 | Covered by the kept portfolio send delivery test. |
| posts.rs a_post_waits_for_a_held_board_lock_and_applies_once_it_is_released | W7 | A 400 ms sleep, an elapsed time bound and a lock file race. Deleted, not made deterministic. |
| pr_checks.rs a_quick_check_run_is_linked_to_its_record_before_it_is_reported_ended | W7 | Twenty runs raced against the link, ordering luck. |
| runs.rs a_line_typed_into_a_running_launch_reaches_its_stdin_and_the_log | W4 | The stdin route is held by `runs_stdin.rs`. |
| shadow.rs a_home_without_the_marker_stays_a_normal_server | W5 | Asserts a default flag is false. |

### argus

| Case | Rule | Reason |
|---|---|---|
| lifecycle/live.rs a_pid_file_that_shows_up_late_is_still_read | W7 | A real 120 ms sleep against a retry loop with a real interval. |
| lifecycle/live.rs no_pid_file_gives_up_after_the_tries | W7 | Needs the real sleeping loop; nothing else is checked. |
| lifecycle/stop.rs the_wait_ends_on_the_clock_when_every_probe_is_slow | W7 | A real socket, a real one second clock and a 6 second bound. |
| main.rs the_command_line_is_well_formed | W2 | A clap `debug_assert`; the parse tests build the same command. |
| main.rs the_exit_does_not_wait_for_blocking_work | W7 | A 30 second blocking task against a 5 second bound. |
| tests/it/dns_cli.rs status_says_off_until_the_config_turns_the_names_on | W4 | The off lines are held by `dns/status.rs`; the other status cases cover the wiring. |
| tests/it/mcp_stdio.rs a_call_goes_through_with_the_default_project_and_no_token | W4 | Same fact as `tools_call_is_forwarded...` in argus-mcp `bridge_stub.rs`. |
| tests/it/mcp_stdio.rs with_require_token_the_bridge_sends_it_and_a_wrong_one_reads_as_not_running | W4 | Same as the argus-mcp stub case (K4 by look). |
| tests/it/mcp_stdio.rs the_first_call_starts_it_once_waits_then_says_the_dashboard_is_not_running | W4 | Same as the argus-mcp bridge case. |
| tests/it/mcp_stdio.rs the_server_is_started_as_soon_as_the_bridge_runs | W4 | Same as the argus-mcp bridge case. |
| tests/it/mcp_stdio.rs with_the_quit_flag_... and without_the_flag_a_crashed_dashboard_... | W4 | Kept in argus-mcp `bridge.rs` and `bridge_stub.rs` with the same names; the argus copies went. |
| tests/it/node_form.rs a_forwarded_version_is_bare and a_forwarded_unknown_command_is_refused_the_node_way | W4 | The `node_form.rs` unit tests hold the answers; the kept e2e case holds the wiring. |
| tests/it/tray_cli.rs an_unknown_tray_verb_is_a_usage_error | W5 | A clap usage error. |

### argus-mcp

| Case | Rule | Reason |
|---|---|---|
| http.rs tools_list_has_23_well_formed_tools | W4 | `defs.rs` checks the same 23 tools and their schemas. |

## W7 (timing) conversions

Made deterministic by waiting for an outcome (`crate::wait::until`, which polls an outcome for up to 20 s) or by removing the sleep:

- argus-server notify.rs: absence by sleep became a FIFO ordering sentinel (a later wake proves the earlier ones were processed).
- argus-server runs_server.rs, runs_store.rs, records.rs, procs.rs, mcp.rs: fixed loops and sleeps replaced by `until`; two elapsed bounds removed from runs_server.rs.
- argus wake_stream.rs: the 600 ms and 2500 ms sleeps went; the 300 ms one became a bounded poll for the empty listener list.
- argus mcp_stdio.rs and argus-mcp bridge*.rs: the 300 ms and 600 ms start waits of the bridge became 5 ms.
- argus cli.rs: the 5 second bound on a stop with an open stream was removed; the process-ended assertion stays.

Deleted instead of converted: the W7 rows above (doc_stream, early_refusal, posts lock, pr_checks race, live.rs x2, stop.rs, main.rs).

## Helpers and structure

- Dead helpers: `#![allow(dead_code)]` removed from `posting/mod.rs`, `support/wait.rs`, `runs/rig.rs` (also `unused_imports`) and from `mod toasts` in `main.rs`. The wake and worktrees rigs keep theirs (R3).
- `machine/mod.rs` and `toasts/mod.rs` are unchanged (argus-coord includes both by `#[path]`).
- Split by tag: alias, local_names, localhost_door, http (rules files, critical); cache_worktree, portfolio_send_cli, pr_checks_runs, records_node (e2e); shadow_server, sse_buffer; mcp_stdio_process (e2e); install/replace/link_tests (e2e); argus-mcp bridge_stub (standard).
- Nothing was compiled or run, so imports and visibility of the split files are untested.
