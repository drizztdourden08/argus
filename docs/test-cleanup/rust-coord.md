# Test clean-up: rust-coord

Verdicts against the shared test standard (`X:/ai-config/docs/test-standard.md`) for the Rust tests
of `crates/argus-core`, `crates/argus-board` and `crates/argus-coord`. A case is one `#[test]` or
`#[tokio::test]`. Nothing was run in this pass (no cargo build, check, clippy or test); the checker
runs them. Tags follow the Rust convention: one `// test-tag:` line per file, and every e2e test
also carries `#[ignore = "e2e"]`.

## Counts

| crate | files before | cases before | files after | cases after | critical | standard | e2e |
|---|---|---|---|---|---|---|---|
| argus-core | 11 | 47 | 10 | 38 | 38 | 0 | 0 |
| argus-board | 8 | 53 | 8 | 46 | 46 | 0 | 0 |
| argus-coord | 50 | 437 | 58 | 415 | 300 | 42 | 73 |
| total | 69 | 537 | 76 | 499 | 384 | 42 | 73 |

Before and after come from a scan of the base commit (3a35cac) and of this branch. Cases that moved
to a split file count in the new file, so a "trim" row can show fewer cases than it lost.

## Rules applied to the tags

- critical: no socket, no child process, no sleep, temp dirs only.
- standard: an in-process server or stand-in on 127.0.0.1 (the Argus test server, the loopback
  stand-ins of the hook tests, the tiny listeners of the client tests), and any test that dials a
  closed port.
- e2e: a spawned process (node stub, `ping`/`sleep`, a fake `brock` script, the `argus` binary, a
  run script started by the server), and every Node against Rust parity file (`remote_differ`,
  `hook_differ`, `thread_diff`, `thread_diff_records`, `agent_diff`, `work_differential`,
  `differ_examples`, `differ_harness_node`) and the latency bench (E3).

## Split files

- `client.rs` (standard) and `client_config.rs` (critical: the port, the token and the event reader).
- `differ_harness.rs` (critical: the normaliser and the judge) and `differ_harness_node.rs` (e2e).
- `hook_listen.rs` (standard) and `hook_listen_process.rs` (e2e); the stand-in stream moved to
  `hook_support/listen.rs`.
- `dashboard_verb.rs` (critical) and `dashboard_verb_run.rs` (e2e).
- `lead_verbs.rs` (critical) and `lead_verbs_wake.rs` (standard).
- `pr_verbs.rs` (critical) and `pr_verbs_run.rs` (e2e: fake `brock` merge and check runs).
- `runs_cli.rs` (critical), `runs_cli_server.rs` (standard) and `runs_cli_run.rs` (e2e); the shared
  helpers moved to `runs_support/mod.rs`.

## Helpers deleted

`Rig::checkout`, `Rig::put`, `Rig::threads`, `Rig::board` and the constant `WORKER` in
`argus-coord/tests/it/remote/mod.rs` (no test uses them any more), the `Trickle` reader in
`client.rs`, and the helpers of the deleted wake-fault cases (`faults`, `faulted`, `as_session`,
`clear`). The module-level `#![allow(dead_code)]` of `remote/mod.rs` and `hook_support/mod.rs` was
left, because a shared module can only be proven clean by a compile.

## W7 (sleeps, real clocks, timing)

Made deterministic:

- `argus-board` `lock.rs`: the async lock test waited 300 ms and counted ticks of a timer. It now
  uses a zero wait and asserts the `Locked` refusal only.
- `argus-coord` `hook_listen.rs`: the unregistered session case also waited 700 ms of real time. The
  wait half is gone; the stranger half stays.
- `argus-core` `fs.rs`: the rename retry pause is 0 ms under `cfg!(test)`, so the retry cases take no
  real time.

Deleted: the client timeout case (a real 1 s timeout), the board lock "runs once released" case
(150 ms sleep), and the four wake-fault cases whose fixture polls a real server tick for up to 10 s.

Left on purpose, all in e2e files: the node stub polls of `hook_autostart.rs`, the parent-gone poll
of `hook_listen_process.rs`, the real run scripts of `runs_cli_run.rs`.

## Verdicts

| file | verdict | rule | tag | cases before | cases after |
|---|---|---|---|---|---|
| argus-core/src/fs.rs | trim | K3 | critical | 10 | 9 |
| argus-core/src/launch.rs | keep | K2 | critical | 2 | 2 |
| argus-core/src/names.rs | trim | K1 | critical | 4 | 3 |
| argus-core/src/page_url.rs | keep | K1 | critical | 2 | 2 |
| argus-core/src/remote.rs | trim | K1 | critical | 4 | 3 |
| argus-core/src/shadow_mark.rs | keep | K1, K4 | critical | 4 | 4 |
| argus-core/tests/it/clock.rs | trim | K1 | critical | 4 | 1 |
| argus-core/tests/it/config.rs | keep | K2, K3, K4 | critical | 9 | 9 |
| argus-core/tests/it/json_text.rs | trim | K2 | critical | 4 | 3 |
| argus-core/tests/it/one_writer.rs | delete | W1 |  | 2 |  |
| argus-core/tests/it/unique.rs | keep | K1 | critical | 2 | 2 |
| argus-board/src/store/damage.rs | trim | K3 | critical | 3 | 2 |
| argus-board/tests/it/entry_images.rs | trim | K2, K3 | critical | 4 | 3 |
| argus-board/tests/it/export_import.rs | keep | K2, K3 | critical | 9 | 9 |
| argus-board/tests/it/lock.rs | trim | K3 | critical | 10 | 6 |
| argus-board/tests/it/names.rs | trim | K1 | critical | 5 | 4 |
| argus-board/tests/it/ops.rs | keep | K1, K3, K4 | critical | 9 | 9 |
| argus-board/tests/it/portfolio.rs | keep | K1, K2, K3 | critical | 6 | 6 |
| argus-board/tests/it/prs.rs | keep | K1, K4 | critical | 7 | 7 |
| argus-coord/src/verbs/remote/support/collate.rs | keep | K1 | critical | 1 | 1 |
| argus-coord/src/verbs/remote/worktree/idle.rs | keep | K1 | critical | 1 | 1 |
| argus-coord/src/verbs/remote/worktree/remove.rs | keep | K2 | critical | 1 | 1 |
| argus-coord/src/verbs/shared/lock.rs | keep | K1 | critical | 2 | 2 |
| argus-coord/tests/it/agent_diff.rs | keep | E3 | e2e | 11 | 11 |
| argus-coord/tests/it/agent_model.rs | keep | K1 | critical | 13 | 13 |
| argus-coord/tests/it/board_shortcut.rs | keep | K1 | critical | 14 | 14 |
| argus-coord/tests/it/board_shortcut_verb.rs | trim | K2 | critical | 7 | 4 |
| argus-coord/tests/it/build_verbs.rs | trim | K2 | critical | 21 | 20 |
| argus-coord/tests/it/caller_lookup.rs | keep | K4 | critical | 12 | 12 |
| argus-coord/tests/it/cli_args.rs | keep | K1 | critical | 14 | 14 |
| argus-coord/tests/it/client.rs | trim | S2 | standard | 19 | 11 |
| argus-coord/tests/it/client_config.rs | split (new) | K1 | critical |  | 7 |
| argus-coord/tests/it/dashboard_verb.rs | trim | K2 | critical | 3 | 2 |
| argus-coord/tests/it/dashboard_verb_run.rs | split (new) | E2 | e2e |  | 1 |
| argus-coord/tests/it/differ_examples.rs | trim | E3 | e2e | 4 | 1 |
| argus-coord/tests/it/differ_harness.rs | trim | K1 | critical | 11 | 8 |
| argus-coord/tests/it/differ_harness_node.rs | split (new) | E3 | e2e |  | 1 |
| argus-coord/tests/it/entry_context.rs | keep | K2 | critical | 9 | 9 |
| argus-coord/tests/it/entry_images.rs | trim | K3 | critical | 6 | 4 |
| argus-coord/tests/it/format.rs | keep | K1 | critical | 9 | 9 |
| argus-coord/tests/it/hook_agents.rs | trim | K1 | critical | 16 | 15 |
| argus-coord/tests/it/hook_agents_binding.rs | trim | K1 | critical | 12 | 11 |
| argus-coord/tests/it/hook_agents_id.rs | keep | K1 | critical | 8 | 8 |
| argus-coord/tests/it/hook_autostart.rs | keep | E2 | e2e | 6 | 6 |
| argus-coord/tests/it/hook_differ.rs | keep | E3 | e2e | 3 | 3 |
| argus-coord/tests/it/hook_guard.rs | keep | K4 | critical | 16 | 16 |
| argus-coord/tests/it/hook_listen.rs | trim | S2 | standard | 11 | 7 |
| argus-coord/tests/it/hook_listen_process.rs | split (new) | E2 | e2e |  | 4 |
| argus-coord/tests/it/hook_notices.rs | keep | S2 | standard | 5 | 5 |
| argus-coord/tests/it/hook_session.rs | keep | K1 | critical | 13 | 13 |
| argus-coord/tests/it/hook_transcript.rs | keep | K1 | critical | 5 | 5 |
| argus-coord/tests/it/launch_plan.rs | trim | K2 | critical | 4 | 3 |
| argus-coord/tests/it/launcher_verbs.rs | keep | K2 | critical | 12 | 12 |
| argus-coord/tests/it/lead_verbs.rs | trim | K2 | critical | 6 | 5 |
| argus-coord/tests/it/lead_verbs_wake.rs | split (new) | S2 | standard |  | 1 |
| argus-coord/tests/it/limits.rs | trim | K1 | critical | 8 | 7 |
| argus-coord/tests/it/ping_cli.rs | keep | S2 | standard | 4 | 4 |
| argus-coord/tests/it/portfolio_cli.rs | keep | K2 | critical | 6 | 6 |
| argus-coord/tests/it/pr_verbs.rs | trim | K2 | critical | 9 | 3 |
| argus-coord/tests/it/pr_verbs_run.rs | split (new) | E2 | e2e |  | 6 |
| argus-coord/tests/it/procs_cli.rs | keep | S2 | standard | 7 | 7 |
| argus-coord/tests/it/project_lookup.rs | keep | K1 | critical | 10 | 10 |
| argus-coord/tests/it/purge_check_tick.rs | keep | K3 | critical | 7 | 7 |
| argus-coord/tests/it/remote_differ.rs | keep | E3 | e2e | 8 | 8 |
| argus-coord/tests/it/remote_switch.rs | keep | K2 | critical | 12 | 12 |
| argus-coord/tests/it/route.rs | trim | K2 | critical | 16 | 14 |
| argus-coord/tests/it/runs_cli.rs | trim | K2 | critical | 12 | 4 |
| argus-coord/tests/it/runs_cli_run.rs | split (new) | E2 | e2e |  | 6 |
| argus-coord/tests/it/runs_cli_server.rs | split (new) | S2 | standard |  | 2 |
| argus-coord/tests/it/thread_diff.rs | keep | E3 | e2e | 9 | 9 |
| argus-coord/tests/it/thread_diff_records.rs | keep | E3 | e2e | 2 | 2 |
| argus-coord/tests/it/thread_verbs.rs | keep | K2 | critical | 15 | 15 |
| argus-coord/tests/it/wake_faults_cli.rs | trim | K2 | standard | 5 | 1 |
| argus-coord/tests/it/work_differential.rs | keep | E3 | e2e | 14 | 14 |
| argus-coord/tests/it/work_inbox.rs | keep | S2 | standard | 4 | 4 |
| argus-coord/tests/it/worktree_verbs.rs | keep | K2 | critical | 13 | 13 |
| argus-coord/tests/latency_bench.rs | keep | E3 | e2e | 1 | 1 |

## Deleted cases that looked like K1 to K4

Every deleted case is listed, with the rule it broke.

| case | file | rule | reason |
|---|---|---|---|
| the_manual_clock_moves_when_told | argus-core clock.rs | W5 | tests the test double, which every other test uses |
| clones_of_a_manual_clock_share_one_time | argus-core clock.rs | W5 | same |
| the_system_clock_reads_the_wall_clock | argus-core clock.rs | W7 | reads the real clock, asserts nothing the OS does not guarantee |
| the_text_reads_back_as_the_same_number | argus-core json_text.rs | W4 | the whole-float case already pins the text |
| no_crate_names_its_own_temp_file | argus-core one_writer.rs | W1 | reads source files; a lint or structure rule, not a test |
| only_the_listed_files_rename | argus-core one_writer.rs | W1 | same |
| the_temp_file_sits_beside_the_target_and_names_the_pid | argus-core fs.rs | W9 | private helper reached by the public write cases |
| a_missing_file_is_local | argus-core remote.rs | W4 | the change case ends with the missing file and asserts the same mode |
| no_key_is_off_with_the_default_address_and_suffix | argus-core names.rs | W5 | asserts defaults and constants |
| with_board_async_runs_once_the_lock_is_released_and_saves | argus-board lock.rs | W7 | 150 ms sleep and a spawned release; the save is covered by the create case |
| save_board_retries_a_rename_that_windows_refuses_then_succeeds | argus-board lock.rs | W4 | the retry is argus-core `fs.rs`; the board only delegates |
| save_board_goes_through_the_atomic_writer_whole_file_in_a_temp_beside_the_board | argus-board lock.rs | W9 | asserts the private temp name; the saved board is covered elsewhere |
| save_board_gives_up_after_8_attempts_and_leaves_no_temp_file | argus-board lock.rs | W4 | same fact as the argus-core give-up case |
| an_entry_without_images_is_exported_as_it_is | argus-board entry_images.rs | W4 | the other export case shows the same record |
| the_ids_stay_the_shown_names_are_argus_and_overlord | argus-board names.rs | W5 | asserts three constants |
| the_message_of_a_nul_board_says_what_happened_and_what_to_do | argus-board damage.rs | W10 | wording; the lock case checks the NUL byte count |
| a_server_that_is_up_but_does_not_answer_in_time_is_a_timeout_never_not_running | argus-coord client.rs | W7 | a real 1 s timeout |
| the_same_call_again_changes_nothing | argus-coord board_shortcut_verb.rs | W4 | same fact in `board_shortcut.rs` |
| the_overlord_gets_two_rows_and_other_projects_one | argus-coord board_shortcut_verb.rs | W4 | same fact in `board_shortcut.rs` |
| a_reserved_key_gets_no_shortcut_and_the_argus_project_is_argus_app | argus-coord board_shortcut_verb.rs | W4 | same fact in `board_shortcut.rs` |
| rm_retires_every_launcher_made_for_the_bundle_once_and_for_good | argus-coord build_verbs.rs | W4 | the forced rm case asserts the same retirement |
| an_entry_without_images_has_no_images_field | argus-coord entry_images.rs | W4 | the refusal cases keep nothing; the ask case lists the images |
| extra_words_without_an_image_flag_are_ignored_as_before | argus-coord entry_images.rs | W4 | same absent field in another state |
| only_a_true_marker_makes_a_home_a_shadow | argus-coord launch_plan.rs | W4 | the same marker rule is pinned in argus-core `shadow_mark.rs` |
| the_limit_of_each_kind_is_the_documented_one | argus-coord limits.rs | W5 | asserts a table of constants |
| a_record_of_the_lead_with_no_claude_id_is_never_taken_by_a_start_that_reads_like_it | argus-coord hook_agents.rs | W4 | same bug and assertion as the first case of `hook_agents_binding.rs` |
| the_agent_call_result_never_rewrites_a_record_of_another_agent_with_its_tool_use_id | argus-coord hook_agents_binding.rs | W4 | same bug and assertion as the second case of `hook_agents_id.rs` |
| read_verbs_list_show_and_refuse | argus-coord differ_examples.rs | W4 | a demo of the harness; `thread_diff` and `work_differential` run the same verbs |
| write_verbs_add_move_and_read_back | argus-coord differ_examples.rs | W4 | same |
| inbox_and_a_hook_with_stdin | argus-coord differ_examples.rs | W4 | `work_differential` (inbox) and `hook_differ` (prompt) cover it |
| twin_homes_hold_the_same_boards_and_a_closed_dashboard_port | argus-coord differ_harness.rs | W5 | tests a fixture of the harness; the node case reads the same homes |
| equal_runs_are_the_same | argus-coord differ_harness.rs | W5 | the trivial arm of the judge |
| list_works_for_any_session_and_the_cleared_one_is_not_listed | argus-coord wake_faults_cli.rs | W7 | the fixture polls a real server tick |
| clear_is_refused_for_everyone_but_the_overlord | argus-coord wake_faults_cli.rs | W7 | K4 by its look; same fixture. See the open question below |
| clear_needs_a_note_a_target_and_an_open_fault | argus-coord wake_faults_cli.rs | W7 | same fixture |
| with_both_faults_cleared_the_list_no_longer_shows_them | argus-coord wake_faults_cli.rs | W7 | same fixture |
| an_alias_of_a_module_is_served_by_that_module | argus-coord route.rs | W4 | the alias table case lists `todo` to `action` |
| the_session_registry_finds_the_project_for_a_verb | argus-coord route.rs | W9 | project lookup is `project_lookup.rs` |

## Open questions

- The wake-fault cases (list, clear refused for non-Overlords, clear needs a note) cover a
  permission rule (K4). They were deleted for W7 because the server's wake engine ticks on a real
  timer. They can come back once the testkit server can tick its wake engine by hand
  (Overlord or the server lead: a manual tick seam in `argus-testkit`).
- Inside the long scenario cases of `build_verbs.rs`, `launcher_verbs.rs`, `thread_verbs.rs` and
  `worktree_verbs.rs` several facts are asserted in one test; they were judged as one case each and
  not split.
