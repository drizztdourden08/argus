# Test clean-up: Rust crates (rust-other)

Verdicts against the shared test standard (`X:/ai-config/docs/test-standard.md`). A case is one `#[test]` or `#[tokio::test]`. Nothing was run in this pass (no tests, clippy or build); the checker runs them. Tag convention: one `// test-tag:` line per file with tests; every e2e case also carries `#[ignore = "e2e"]`.

Crates: argus-launchers, argus-agents, argus-procs, argus-native, argus-notify, argus-parity, argus-records, argus-store, argus-cache, argus-sync, argus-dns.

## Counts

| crate | files before | files after | cases before | cases after | critical | standard | e2e |
|---|---|---|---|---|---|---|---|
| argus-launchers | 13 | 13 | 118 | 113 | 61 | 44 | 8 |
| argus-agents | 9 | 9 | 108 | 105 | 105 | 0 | 0 |
| argus-procs | 23 | 23 | 78 | 74 | 72 | 0 | 2 |
| argus-native | 21 | 21 | 76 | 68 | 67 | 0 | 1 |
| argus-notify | 17 | 17 | 59 | 56 | 24 | 32 | 0 |
| argus-parity | 14 | 12 | 58 | 51 | 44 | 0 | 7 |
| argus-records | 11 | 10 | 29 | 28 | 18 | 10 | 0 |
| argus-store | 3 | 3 | 26 | 26 | 0 | 26 | 0 |
| argus-cache | 5 | 5 | 24 | 24 | 9 | 12 | 3 |
| argus-sync | 3 | 2 | 21 | 17 | 0 | 15 | 2 |
| argus-dns | 3 | 3 | 12 | 11 | 9 | 2 | 0 |
| total | 122 | 118 | 609 | 573 | 409 | 141 | 23 |

Untagged after: 0.

Tag rules used: a single module or pure function (inline `#[cfg(test)]` modules, file helpers on a temp dir) is critical. A service or store driven through fakes, a temp-dir SQLite database, a temp-dir store or an in-process test server is standard (S2). Real child processes, real OS junctions, real toast calls, the Node or Rust coord CLI and the two-server parity run are e2e.

## Decisions to review

- argus-parity: the crate is parity (E3), but its inline unit tests (diff, pattern, normalise, verdict, report) are plain K1 logic with no server, so they stay critical. Only `tests/two_servers.rs` (two real servers, a 3 second real stream window) is e2e.
- argus-launchers: every `runs_*` file drives the service over the fake host, store and events on tokio paused time. Tagged standard (S2). Retag to critical if the fast ones should run on every change.
- argus-cache: `tests/rig/mod.rs` keeps `#![allow(dead_code)]`. Every helper is used by at least one of the two test binaries; the allow only covers the helper subset each binary does not use.
- argus-sync `tests/export.rs` starts the real file watcher inside `SyncService::start` and writes board files then calls `resync`; a background watcher pass could in theory race an event count. Kept (S2), flagged.

## Deleted cases that would have looked K1 to K4

### launchers

- `launcher_stale::the_reasons_are_the_names_the_wake_store_knows` (K2 by look): two string constants asserted against their own literals (W5).
- `launcher_stale::the_defaults_a_tick_every_10_minutes_...` (K2 by look): `Default` values of a config struct (W5); behaviour covered by the tick tests.
- `runs_logs::output_is_split_into_lines_crlf_is_one_break_...` (K1 by look): splitter rules covered by `units_logs.rs` (W4, W9).
- `runs_logs::a_very_long_line_is_cut_into_pieces_of_at_most_16k_characters` (K1 by look): same boundary asserted on the splitter in `units_logs.rs` (W4, W9).
- `runs_terminal::a_start_that_cannot_happen_is_final_at_once_and_says_why` (S3 by look): same missing working directory failure asserted in `runs_lifecycle.rs` (W4).

### agents

- `export::the_first_export_carries_nothing_live_until_the_watcher_has_read_the_transcript` (K1): the watcher overlay test asserts no `live` key through the same path (W4).
- `export::new_transcript_lines_show_up_in_the_doc_on_the_next_tick` (K1): three watcher cases assert the same tail, activity and turns behaviour (W4).
- `model::a_new_model_for_an_existing_record_needs_a_new_why_with_the_same_rules` (K1): same `check_model_why` inputs as the first, second and fifth cases (W4).

### procs

- `defaults::the_defaults_are_the_node_defaults_in_the_node_order` (K2): snapshot of a constant name list (W5); first-match behaviour is in `tests/monitor/rules.rs`.
- `defaults::a_dev_server_may_use_3_gb_before_it_is_flagged` (K1): one constant, and its title contradicted it (W5).
- `text::memory_reads_in_mb_then_gb` (K1): covered by the edge-table test (W4).
- `responder::the_script_names_the_pids` (K1): substring of a private script builder with no outcome (W9).

### native

- `icon::the_stopped_icon_keeps_the_red_badge_on_a_light_taskbar_and_at_16_pixels` (K1): same render and badge as the stopped icon case at 32 pixels (W4).
- `icon::the_badge_edge_is_smooth_not_a_hard_cut` (K1): private `disc_coverage` helper that `render` already reaches (W9).
- `menu::a_stopped_server_offers_start_between_restart_and_quit` (K2): the exact stopped menu order is asserted by the Node menu case (W4).
- `launch::waiting_gives_up_when_tray_json_never_appears` (K1): needed a real 20 ms deadline and a thread sleep (W7); the finding case stays.
- `toast_payload::the_recording_notifier_keeps_what_was_shown_in_order` (K2): tests the fake notifier (W5).
- `toast_payload::an_invalid_toast_is_refused_by_the_fake_like_by_the_real_notifier` (K4): tests the fake; the payload refusals are asserted by the payload cases (W5, W4).
- `toast_payload::a_failing_platform_is_an_error_not_a_panic` (S3): tests the fake and an error Display string (W5).
- `toast_payload::the_notifier_is_usable_as_a_shared_trait_object` (K2): what the compiler guarantees (W2).

### notify

- `limits::the_knobs_are_what_the_contract_says` (K2): constants (W5); the limits are asserted by their behaviour cases.
- `level::a_value_is_built_and_read_per_level` (K1): generic helper reached by the service cases (W5).
- `settings::the_defaults_are_a_silent_ping_a_loud_urgent_and_no_info_toast` (K2): defaults already asserted by `tests/notify/settings.rs` (W4).

### parity

- `probe::labels_name_the_method_and_the_path`, `probe::the_mcp_calls_are_the_two_reads` (K2 by look): trivial getters of the probe table (W5). The file keeps no test; the module was removed.
- `probes::every_probe_is_a_get_or_an_mcp_read` (K2 by look): repeats the type (W2).
- `probes::a_project_has_every_collection_and_the_portfolio_has_only_the_collections` (K2 by look): asserts the counts 20 and 24, a constant (W5); the two-server run compares every route.
- `kind::every_doc_kind_ignores_seq`, `kind::the_process_sample_numbers_are_never_compared` (K1 by look): assert a constant list (W5); `judge` cases assert the behaviour.
- `got::no_answer_is_status_zero` (K1 by look): a Display string (W5).

### records

- `folder_watch::a_file_appearing_calls_back_once_after_the_burst` (K1 by look): real OS watcher plus a 10 second polling loop with sleeps (W7), not made deterministic without a fake watcher. The module no longer has tests.

### sync

- `watch::a_cli_change_reaches_the_stream_within_a_second` (K2 by look): a real watcher with a one second timing assertion (W7); the CLI version stays as e2e in `watch_cli.rs`.
- `watch::an_owner_post_is_on_the_stream_once_with_no_echo_from_the_watcher` (S3): waits 1.8 real seconds for an event that must not come (W7).
- `watch::a_board_that_appears_after_start_is_mirrored_by_the_poll_or_the_watcher` (S3): polls with a real clock and sleeps up to 12 seconds (W7); the CLI version stays e2e. The file `watch.rs` is deleted.
- `export::health_reports_the_watcher` (K2 by look): a getter (W5).

### dns

- `responder::garbage_gets_no_answer_and_does_not_stop_the_responder` (K4 by look): needs a real 300 ms silence timeout (W7); short and broken packets are asserted by the `wire` and `zone` unit cases.

## W7 conversions and deletions

- Converted to deterministic: `argus-agents/tests/it/schedule.rs` now runs on tokio's paused clock (`test-util` added to the dev-dependency). `argus-store/tests/db.rs` busy case now sets `busy_timeout=0` instead of waiting 50 ms.
- Deleted for W7: `launch::waiting_gives_up...`, `folder_watch` case, three `argus-sync/tests/watch.rs` cases, the dns garbage case (listed above).
- Already deterministic (virtual or injected time): launchers `runs_*` (paused time) and `launcher_stale` (`ManualClock`), agents `reconciler` (`ManualClock`), procs monitor (time from the sample), notify harness (`ManualClock`), native tray closures.
- Real clock kept on purpose in e2e files: launchers `system_host`, procs `sampler`, parity `two_servers`, sync `watch_cli`.

## Helpers deleted

- agents `export.rs`: `World.transcript` and the imports `user_line`, `tool_result`, `Write`, `PathBuf`.
- launchers: `request()` in `runs_terminal.rs` moved with its test to `units_defs.rs`; unused imports removed.
- native: `disc_coverage` import in the icon tests, `Notifier` and `RecordingNotifier` imports in `toast_payload.rs`.
- notify: `PerLevel`, `SOUND_DEFAULTS`, `SUMMARY_EVERY_MS`, `THREAD_PER_MINUTE` test imports.
- dns: the `UdpSocket` import.
- parity: the test modules of `probe.rs` and `probes.rs`.
- sync: the shared `support` doc line now names one binary.
- No `#[allow(dead_code)]` hides an unused helper (see cache above).

## Verdicts

### argus-launchers (tests/it)

| file | verdict | rule | tag |
|---|---|---|---|
| launcher_bundles.rs | keep | K1, K4 | critical |
| launcher_rules.rs | keep | K1, K4 | critical |
| launcher_stale.rs | keep, 2 cases deleted (W5) | K1, K2 | critical |
| units_defs.rs | keep, one case moved in, page doc case made synchronous | K1, K2 | critical |
| units_logs.rs | keep | K1, K4 | critical |
| runs_attach.rs | keep | K3, K4, S3 | standard |
| runs_kill.rs | keep | S2, S3 | standard |
| runs_lifecycle.rs | keep | K1, K4, S2, S3 | standard |
| runs_limits.rs | keep | K1, S2 | standard |
| runs_logs.rs | keep, 2 cases deleted (W4, W9) | K1, K4, S2 | standard |
| runs_recover.rs | keep | K3, S2 | standard |
| runs_terminal.rs | keep, one case moved out, one deleted (W4) | S2, K2, K4 | standard |
| system_host.rs | keep, real processes | E2 | e2e |

Undecided: `runs_lifecycle` `parallel_2_lets_two_run...` is a W4 sibling of the parallel-limit test; kept as a K1 boundary case. The two cap cases in `runs_logs` share a scenario; both kept.

### argus-agents

| file | verdict | rule | tag |
|---|---|---|---|
| tests/it/basics.rs | keep | K1 | critical |
| tests/it/export.rs | keep 4, delete 2 (W4) | K2, K3 | critical |
| tests/it/hook_logic.rs | keep | K1, K4 | critical |
| tests/it/model.rs | keep 8, delete 1 (W4) | K1 | critical |
| tests/it/reaper.rs | keep | K1 | critical |
| tests/it/reconciler.rs | keep | K1 | critical |
| tests/it/schedule.rs | keep, W7 conversion | K2 | critical |
| tests/it/watcher.rs | keep | K1 | critical |
| src/evidence/walk.rs | keep | K1 | critical |

### argus-procs

| file | verdict | rule | tag |
|---|---|---|---|
| src/clock.rs, config.rs, hint.rs, project.rs | keep | K1 | critical |
| src/defaults.rs | keep 1, delete 2 (W5) | K1 | critical |
| src/normalize.rs, num.rs, tree.rs | keep | K1 | critical |
| src/reaper/debounce.rs, mod.rs, pick.rs | keep | K1 | critical |
| src/responder.rs | keep 1, delete 1 (W9) | K1 | critical |
| src/rules_file.rs | keep | K3 | critical |
| src/text.rs | keep, delete 1 (W4) | K1 | critical |
| tests/monitor/alerts.rs | keep | K1 | critical |
| tests/monitor/kill.rs | keep | K4 | critical |
| tests/monitor/publish.rs | keep | K1 | critical |
| tests/monitor/reaper.rs | keep | K4 | critical |
| tests/monitor/rules.rs | keep | K2 | critical |
| tests/monitor/scope.rs | keep | K4 | critical |
| tests/monitor/self_kill.rs | keep | K4 | critical |
| tests/monitor/status.rs | keep | K1 | critical |
| tests/sampler.rs | keep, real sampling, ignore | E2 | e2e |

### argus-native

| file | verdict | rule | tag |
|---|---|---|---|
| src/identity.rs, text.rs | keep | K1, K2 | critical |
| src/tray/balloon.rs, log.rs, paths.rs, record.rs, quit_flag.rs, timestamp.rs | keep | K1, K2, K3 | critical |
| src/tray/fallback.rs, status.rs, stopping.rs, watch.rs | keep | K1, S2 (closures injected) | critical |
| src/tray/gate.rs, state.rs | keep | K1 | critical |
| src/tray/icon.rs | keep 6, delete 2 (W4, W9) | K1, K2 | critical |
| src/tray/launch.rs | keep 4, delete 1 (W7) | K1 | critical |
| src/tray/menu.rs | keep 4, delete 1 (W4) | K2 | critical |
| src/tray/native/handlers.rs, session.rs | keep | K1 | critical |
| tests/toast_payload.rs | keep 10, delete 4 (W5, W4, W2) | K2, K4 | critical |
| tests/toast_winrt.rs | keep, real WinRT, ignore | E2 | e2e |

### argus-notify

| file | verdict | rule | tag |
|---|---|---|---|
| src/clock.rs, config.rs, configure.rs, kinds.rs, links.rs, naming.rs, seen.rs, sentence.rs | keep | K1, K2 | critical |
| src/level.rs | keep 1, delete 1 (W5) | K1 | critical |
| src/limits.rs | keep | K1 | critical |
| src/settings.rs | keep 1, delete 1 (W4) | K1 | critical |
| tests/notify/auto.rs | keep | S2 | standard |
| tests/notify/delivery.rs | keep | S2, S3 | standard |
| tests/notify/levels.rs | keep | S2 | standard |
| tests/notify/limits.rs | keep 4, delete 1 (W5) | S2 | standard |
| tests/notify/records.rs | keep | S2 | standard |
| tests/notify/settings.rs | keep | S2 | standard |

### argus-parity

| file | verdict | rule | tag |
|---|---|---|---|
| src/diff.rs, findings.rs, normalize.rs, pattern.rs, report.rs, run.rs, side.rs, stream.rs, verdict.rs | keep | K1, K2 | critical |
| src/got.rs | keep 1, delete 1 (W5) | K1 | critical |
| src/kind.rs | keep 4, delete 2 (W5) | K1 | critical |
| src/probe.rs, probes.rs | delete all (W5, W2) | none | none |
| tests/two_servers.rs | keep, two real servers, ignore | E3 | e2e |

### argus-records

| file | verdict | rule | tag |
|---|---|---|---|
| src/registry.rs, replay.rs, stamp.rs | keep | K1, K3 | critical |
| src/safe_path.rs | keep | K4 | critical |
| src/releases/kind.rs, range.rs, semver.rs, yml.rs | keep | K1, K2 | critical |
| src/folder_watch.rs | delete (W7) | none | none |
| tests/checks.rs | keep | K3, K4 | standard |
| tests/releases.rs | keep | K2, K4 | standard |

### argus-store

| file | verdict | rule | tag |
|---|---|---|---|
| tests/activity.rs | keep | S2 | standard |
| tests/db.rs | keep, busy case made deterministic | K3, S2 | standard |
| tests/snapshot.rs | keep | K3 | standard |

### argus-cache

| file | verdict | rule | tag |
|---|---|---|---|
| src/lockfile.rs, remove.rs, prune.rs | keep | K1, K4 | critical |
| tests/prune.rs | keep | K1, S2 | standard |
| tests/links.rs | keep, real junctions through `cmd`, ignore | E2 | e2e |

### argus-sync

| file | verdict | rule | tag |
|---|---|---|---|
| tests/export.rs | keep 15, delete 1 (W5) | S2 | standard |
| tests/watch.rs | delete file (W7) | none | none |
| tests/watch_cli.rs | keep, real coord CLIs, ignore | E2 | e2e |

### argus-dns

| file | verdict | rule | tag |
|---|---|---|---|
| src/wire.rs, zone.rs | keep | K1, K4 | critical |
| tests/responder.rs | keep 2, delete 1 (W7) | S2 | standard |
