# Test clean-up: rust-worktrees-wake

Verdicts against the shared test standard (`X:/ai-config/docs/test-standard.md`) and the Rust tag convention. A case is one `#[test]` or `#[tokio::test]`. Nothing was run in this pass (no cargo test, check, clippy or build); the checker runs them. `cargo fmt` was run on the two crates.

## Counts

| | files before | files after | cases before | cases after | critical | standard | e2e |
|---|---|---|---|---|---|---|---|
| argus-worktrees (inline modules in src plus src/specs) | 49 | 56 | 319 | 314 | 85 | 186 | 43 |
| argus-wake (tests/it) | 13 | 13 | 159 | 154 | 146 | 8 | 0 |
| both | 62 | 69 | 478 | 468 | 231 | 194 | 43 |

Untagged after: 0. Every e2e case also carries `#[ignore = "e2e"]`. Files with tests in argus-worktrees: critical 33, standard 16, e2e 7. In argus-wake: critical 11, standard 2.

## Decisions

- **The service harness runs on a fake git (I-53).** `Harness` wraps `ScriptedExec`, which hands every call except a simulated `worktree remove` to the git of the fixture. `Fixture::fake()` registers a `FakeGit` (`src/fakegit`) for its folder, so `git_in`, `commit`, `listed`, `branches` and the harness all reach it, and no process starts. `Fixture::new()` stays on the real git. The fake keeps repositories in memory (commits with one parent, branches, remote branches, tags, HEAD per checkout, worktree records, locks, upstreams) and reads the dirty and untracked files from the temporary folder against the files the HEAD commit tracks, with ignore patterns from `info/exclude` and the root `.gitignore`. It writes the folders git keeps (`HEAD`, `index`, loose refs, `worktrees/<name>`) so the scan fingerprints behave. It answers `rev-parse`, `status --porcelain=v1 [-b]`, `log`, `rev-list --count`, `merge-base --is-ancestor`, `for-each-ref --contains`, `cherry`, `symbolic-ref`, `config --get`, `remote get-url|add`, `worktree list|add|remove|lock|unlock|prune`, `update-ref -d`, `bundle create|verify`, `diff HEAD --binary --output=`, `ls-files`, `check-ignore -q`, `add` and `commit -m`; anything else fails with a clear sentence. 170 cases moved to standard; none was deleted.
- **What stays e2e on the real git, and why.** `safety.rs` (squash merges, a bare origin and a push, `git cherry` patch ids, a real tag: the fake has no merges, no patch ids, no network remotes), `probe.rs` (the fingerprint reads the files real git rewrites), `scan_git.rs` (the real git processes a scan starts and what they leave untouched), `links.rs` (a real `git worktree remove` beside junctions: the smoke for the `node_modules` junction journey), `stuck.rs` (a real 3 s time limit), `exec_real.rs` (real child processes), and `remove_real.rs` (one smoke per journey: remove a merged worktree, refuse a dirty one, a real `git fetch` of the bundle a discard wrote, the git timeout fallback).
- **Link fixtures are standard, not critical.** `make_link` creates a junction with `cmd /C mklink /J` on Windows, which is a child process. Every case that makes a link (fs.rs, tree.rs, links_fs.rs) is `standard`, and the cases of mixed files were split so no file mixes tags.
- **Pure files stay critical:** the status judge and wake text (service_status.rs), the nested git walk (rules_walk.rs), the rule and scan cases on a git that says not a repository (scan.rs), the JSON key spelling (wire_answer.rs), and 29 inline modules.
- **argus-wake is all fake clock.** Debounce and digest cases (wake_policy.rs, engine.rs, items.rs) use `EngineRig` and `Rig` with `ManualClock`: K1 and critical, none touched. `service.rs` runs the whole `WakeService` on in-memory storage (S2), so it is standard; `driver.rs` runs a tokio task with a short real sleep, so it is standard too.

## W7 (sleeps, real clocks, polling)

| case | change |
|---|---|
| git.rs `at_most_two_git_processes_at_a_time...` | made deterministic: the 15 ms sleep is a `Barrier(2)`, so two running at once is proven and a third would show |
| queue.rs `tasks_run_in_order...` | made deterministic: the 5 ms sleep inside each task removed (order does not depend on it) |
| turns.rs `a_waiting_turn_comes_before...` | made deterministic: the 50 ms sleep became a wait on the ticket counter (`yield_now`, no clock) |
| specs/turns.rs, specs/requests.rs | made deterministic: `within_a_minute` (real clock deadline, 20 ms sleep) became `until` (a `yield_now` loop on the state another thread sets) |
| specs/stuck.rs | partly: the 6 s `thread::sleep` of the hung git became a `Gate` and the 120 s elapsed assertion was dropped. The 3 s `target_limit_ms` under test is a real `recv_timeout` in production code, so the case still waits 3 s. It stays e2e (K5, a request that stayed running for an hour). Not deterministic without an injectable clock in `remove_watched`: listed as undecided |
| argus-wake driver.rs | made deterministic: `SystemClock`, a 30 ms window and a 5 s `tokio::time::timeout` became a `ManualClock` advanced past the window, no timeout. The driver still sleeps a few ms in real time between ticks (standard) |
| specs/hygiene.rs `the_threads_release_answers_at_once...` | kept: `recv_timeout(30 s)` is only a failsafe; the outcome is decided by the held turn, not by the time |
| exec.rs, new specs/exec_real.rs | kept as e2e: a hung program and a child that holds the pipe cannot be tested without a real process and a real time limit (150 ms and 8 s sleepers) |

## Deleted cases that looked like K1 to K4 (one line each)

- review.rs `a_gone_worktree_on_a_detached_head_no_ref_keeps_needs_discard_and_the_owners_say_to_prune` (K4): round2.rs `a_gone_worktree_on_a_detached_head_...by_id_and_for_all` asserts the same refusals and the same owner entry path, plus the all-prune (W4).
- review.rs `a_prune_refuses_when_git_would_also_forget_a_record_argus_did_not_vet` (K4): round2.rs `a_prunable_record_no_rule_vetted_stops_the_prune...` proves the same refusal by id and for all, and fallback.rs keeps the variant that finishes a folder (W4).
- review.rs `a_nested_repository_is_not_safe_it_needs_discard_and_is_copied_into_the_backup` (K3): round2.rs `an_ignored_nested_repository_is_refused_without_discard_then_removed_with_the_pages_say_and_copied` is the same case with the same copy check (W4). Its helper `nested_repo_in_an_ignored_folder` went with it.
- fallback.rs `the_job_removes_links_as_links_a_junction_deep_in_node_modules_and_one_at_the_top` (K3): `the_job_deletes_the_folder_and_every_junction_target_survives` in the same file and links.rs `the_folder_fallback_job_unlinks_the_links_git_left_and_never_follows_them` assert the same end state (W4).
- fallback.rs `real_git_on_the_pnpm_shape_the_worktree_is_gone_and_no_junction_target_lost_a_file` (K3): links.rs `a_worktree_removal_by_git_never_reaches_a_sibling_through_a_link` runs the real git on both junction shapes and checks both siblings (W4).
- rules.rs `rotp_web_include_and_rotp_exclude_share_a_parent_every_child_belongs_to_exactly_one` (K4): scan.rs `every_child_belongs_to_exactly_one_of_two_rules_sharing_a_parent` asserts the exclusive ownership without a git process (W4).
- scan.rs `include_and_exclude_choose_direct_child_folder_names` (K1): paths.rs `name_filters_choose_direct_child_names` and rule.rs `filters_apply_to_names` hold the same boundaries (W4, W9).
- scan.rs `folder_candidates_come_from_directory_listings_only_none_for_a_missing_path_or_a_wildcard` (K1): a subset of `a_folder_rule_takes_every_subfolder_of_its_literal_path_and_a_wildcard_rule_is_invalid` (W4).
- argus-wake driver.rs `a_second_listener_replaces_the_first_and_the_first_is_told_through_its_channel` (K1): engine.rs `a_newer_v2_listener_for_the_same_project_and_thread_key_replaces_the_older_one` asserts the replacement and the Replaced message (W4).
- argus-wake service.rs `a_wake_nobody_listens_for_stays_queued_and_goes_once_to_the_next_listener`: engine.rs `with_no_listener_the_event_stays_queued_and_goes_once_to_the_first_listener_that_connects` (W4).
- argus-wake service.rs `the_pass_that_completes_the_bundle_wakes_once`: policy.rs `the_pass_that_leaves_no_pending_manual_test_wakes_once_with_the_counts` (W4).
- argus-wake service.rs `an_answer_a_note_a_reassigned_item_and_a_resume_wake_the_right_thread_and_a_star_listener_hears_every_thread`: policy.rs has one case per reason and engine.rs the star listener (W4).
- argus-wake service.rs `a_portfolio_send_wakes_the_lead_of_the_project_it_was_delivered_to`: policy.rs `a_portfolio_send_wakes_the_lead_of_every_project_it_was_delivered_to` (W4).
- board.rs `callers_say_who_asked` and jobs.rs `a_doc_names_the_folder` (not K1 to K4): a text getter and a one-line accessor (W5).

## Verdicts: argus-worktrees

Paths are under `crates/argus-worktrees/src/`.

| file | verdict | rule | tag | cases before | cases after | note |
|---|---|---|---|---|---|---|
| approval.rs | keep | K4 | critical | 5 | 5 | owner approval check; five cases, each a different way to be refused |
| backup.rs | keep | K1 | critical | 1 | 1 |  |
| board.rs | trim | K1 | critical | 3 | 2 | deleted callers_say_who_asked (W5) |
| branch.rs | keep | K4 | critical | 1 | 1 | a commit id that could be read as an option is refused |
| clock.rs | keep | K1 | critical | 1 | 1 |  |
| deleting.rs | keep | K1 | critical | 1 | 1 |  |
| error.rs | keep | K2 | critical | 2 | 2 | status code of each error code, io error names |
| exec.rs | split | K1 | critical | 5 | 3 | 3 cases stay critical; the two cases that start a real child process moved to specs/exec_real.rs (e2e) |
| finish.rs | keep | K1 | critical | 1 | 1 |  |
| fs.rs | keep | K3 | standard | 2 | 2 | standard: the link fixture is a junction made by `cmd /C mklink /J` on Windows |
| gh.rs | keep | K4 | critical | 1 | 1 | the default runner only runs `gh pr list` |
| git.rs | keep | K4 | critical | 7 | 7 | reads are read-only, timeouts, the two-process gate; the gate case lost its 15 ms sleep (Barrier) |
| guard.rs | keep | K4 | critical | 3 | 3 | which git commands may run |
| guards.rs | keep | K4 | critical | 4 | 5 | path rules and who may ask; I-55: the sentence for a folder git does not list under a repo rule |
| holders/mod.rs | keep | K1 | critical | 3 | 3 |  |
| hygiene/judge.rs | keep | K1 | critical | 2 | 2 |  |
| hygiene/text.rs | keep | K1 | critical | 1 | 1 |  |
| jobs.rs | trim | K1 | critical | 2 | 1 | deleted a_doc_names_the_folder (W5) |
| measure.rs | split | K1 | critical | 4 | 3 | 3 cases stay critical; the link case moved to specs/links_fs.rs (standard) |
| parse.rs | keep | K1 | critical | 3 | 3 |  |
| paths.rs | keep | K1 | critical | 5 | 5 |  |
| prune.rs | keep | K1 | critical | 3 | 3 |  |
| queue.rs | keep | K1 | critical | 2 | 2 | the 5 ms sleep inside the ordered tasks was removed |
| request.rs | keep | K2 | critical | 1 | 1 |  |
| requests/resolve.rs | keep | K1 | critical | 1 | 1 |  |
| requests/store.rs | keep | K3 | critical | 2 | 2 |  |
| requests/text.rs | keep | K2 | critical | 3 | 3 |  |
| requests/watch.rs | keep | K1 | critical | 1 | 1 |  |
| rule.rs | keep | K4 | critical | 4 | 4 |  |
| tree.rs | keep | K3 | standard | 4 | 4 | standard: link fixture, as fs.rs |
| turns.rs | keep | K1 | critical | 2 | 2 | the 50 ms sleep became a wait on the ticket counter |
| specs/fallback.rs | trim | K3, K4 | standard | 25 | 23 | 2 deleted (W4); on the fake git |
| specs/holders.rs | keep | K4 | standard | 2 | 2 | on the fake git |
| specs/hygiene.rs | keep | K3, K4 | standard | 27 | 27 | on the fake git |
| specs/leftover.rs | trim | K3, K4 | standard | 29 | 27 | 2 moved out (links_fs, service_status), nothing deleted; on the fake git |
| specs/links.rs | split | K3 | e2e | 8 | 4 | 4 e2e cases stay; 4 cases moved to links_fs.rs (standard) |
| specs/probe.rs | keep | K1 | e2e | 7 | 7 |  |
| specs/remove.rs | split | K3, K4 | standard | 25 | 25 | on the fake git; the real `git fetch` of the bundle moved to remove_real.rs, its activity-line half stays here |
| specs/requests.rs | keep | K3, K4 | standard | 15 | 15 | the polling helper within_a_minute became a yield loop; on the fake git |
| specs/review.rs | trim | K3, K4 | standard | 9 | 6 | 3 deleted (W4); on the fake git |
| specs/round2.rs | keep | K3, K4 | standard | 10 | 10 | on the fake git |
| specs/rules.rs | split | K4 | standard | 15 | 11 | 1 deleted (W4); 3 pure walk cases moved to rules_walk.rs (critical); on the fake git |
| specs/safety.rs | keep | K3, K4 | e2e | 17 | 17 |  |
| specs/scan.rs | split | K1 | critical | 12 | 2 | 2 deleted (W4); 2 pure cases stay critical; 8 real git cases moved to scan_git.rs (e2e) |
| specs/service.rs | split | K2 | standard | 24 | 16 | 8 pure cases moved to service_status.rs (critical); 16 on the fake git |
| specs/slow.rs | keep | K3 | standard | 7 | 7 | on the fake git (the timeout is scripted) |
| specs/stuck.rs | keep | K5 | e2e | 1 | 1 | the 6 s sleep became a Gate; the 3 s limit under test is a real time limit (e2e) |
| specs/turns.rs | keep | K5 | standard | 2 | 2 | the polling helper became a yield loop; on the fake git |
| specs/wire.rs | split | K2 | standard | 4 | 3 | 3 cases stay on the fake git; 1 pure case moved to wire_answer.rs (critical) |
| specs/exec_real.rs | new | S3 | e2e | 0 | 2 | two real child process cases moved out of exec.rs |
| specs/remove_real.rs | new | K3, K4 | e2e | 0 | 4 | one smoke per removal journey on the real git: merged goes, dirty is refused, the bundle fetched back, the timeout fallback |
| specs/links_fs.rs | new | K3 | standard | 0 | 6 | standard: junction fixtures; links.rs pure cases, the leftover scan link case, the measure link case |
| specs/rules_walk.rs | new | K4 | critical | 0 | 3 | pure cases of the nested git walk |
| specs/scan_git.rs | new | K1 | e2e | 0 | 8 | the real git cases of scan.rs |
| specs/service_status.rs | new | K1 | critical | 0 | 9 | the pure cases of service.rs and the many_text case of leftover.rs |
| specs/wire_answer.rs | new | K2 | critical | 0 | 1 | the pure case of wire.rs |

## Verdicts: argus-wake

Paths are under `crates/argus-wake/tests/it/`. The crate has no inline test modules.

| file | verdict | rule | tag | cases before | cases after | note |
|---|---|---|---|---|---|---|
| activity.rs | keep | K2 | critical | 3 | 3 | fault lines and the words of each reason |
| agents.rs | keep | K1 | critical | 11 | 11 |  |
| driver.rs | trim | S2 | standard | 2 | 1 | W7: wall clock and 5 s timeout replaced by a manual clock; the second case deleted (W4) |
| engine.rs | keep | K1 | critical | 20 | 20 | debounce and window cases: K1 and critical |
| items.rs | keep | K1 | critical | 18 | 18 |  |
| policy.rs | keep | K1 | critical | 17 | 17 |  |
| presence.rs | keep | K1 | critical | 20 | 20 |  |
| registry.rs | keep | K1 | critical | 18 | 18 |  |
| requests.rs | keep | K4 | critical | 7 | 7 | which outside reasons and who may clear a fault |
| service.rs | trim | S2 | standard | 11 | 7 | 4 deleted (W4) |
| stream.rs | keep | K2 | critical | 6 | 6 | the one line printed per wake, byte for byte |
| sync_wakes.rs | keep | K1 | critical | 17 | 17 |  |
| wake_policy.rs | keep | K1 | critical | 9 | 9 | debounce and digest cases: K1 and critical |
