<!-- @layer docs @kind doc -->

# Test clean-up: web (apps/web)

Verdicts against the shared test standard (`X:/ai-config/docs/test-standard.md`). A case is one `it` or `test` call site. Nothing was run in this pass (no tests, lint or typecheck); the checker runs them.

## Counts

| | before | after |
|---|---|---|
| test files | 106 | 106 |
| cases | 1036 | 759 |
| tagged critical | 0 | 281 |
| tagged standard | 0 | 478 |
| tagged e2e | 0 | 0 |
| untagged | 1036 | 0 |

Per folder (cases before to after): board-tests 187 to 156, board-kanban 169 to 123, board-agents, board-builds and board-thread 157 to 102, board-worktrees, board, data and format 210 to 130, overview 163 to 119, markdown, remote, palette, routing, stream and api 150 to 129. No file was left empty, so no file was deleted. No root-level tooling tests exist outside `apps/web/tests`.

## Tag form

The vitest native tag on the `describe`, one tag per test, the KEEP rule at the start of the describe title (for example `describe('K1 the board model', { tags: ['critical'] }, ...)`). Pure `.test.ts` model tests are `critical`. Mounted `.test.tsx` tests are `standard` even when the rule is K3 or K4, because they mount a Tessera tree (over 1 s). No web test is e2e: the real app journey is not driven from vitest.

## Tooling (standards 0.13.0)

- The root and `apps/web` vitest configs go through `defineStandardsVitest`. The web config keeps Brock's base (automatic JSX, react dedupe, inlined Tessera source) and wraps it. The old `testTimeout: 15000` and `maxWorkers: 3` are gone: the tags carry the timeouts and the standard caps workers at 3.
- Spec `@drizztdourden08/standards` `^0.13.0` in the root devDependencies; root scripts `lint`, `lint:all`, `test`, `test:pr`, `test:e2e` and the placeholder `test:rust`; the `test` script of `apps/web` is gone (the root runs the web project).

## Judgement calls

- Pure rules reached through a mounted markdown render (the parser) stay as S1 `standard`.
- Fixtures that build timestamps relative to `Date.now()` (agents, builds, thread, remote helpers) were left: no kept assertion depends on a minute value. Tests that read the clock for an assertion were frozen with `vi.useFakeTimers({ toFake: ['Date'] })` or deleted, as each part says under W7.

## board-tests

| file | verdict | rule | tag | cases before | cases after |
|---|---|---|---|---|---|
| board-page.test.tsx | keep | S1 | standard | 2 | 2 |
| bundle-cards.test.tsx | trim | S1 (W3 deleted: draft or ready badge and tooltip) | standard | 8 | 7 |
| bundle-extras.test.tsx | keep | S1 | standard | 6 | 6 |
| closed-bundles.test.tsx | trim | S1 (W4 deleted: no toggle when nothing is closed) | standard | 5 | 4 |
| conflicts.test.tsx | keep | S1, S3 | standard | 7 | 7 |
| drawer-nav.test.tsx | keep | S1 (W3 class name assertions removed inside one case) | standard | 8 | 8 |
| drawer.test.tsx | trim | S1 (W3 deleted: sections and thumbnails shown after mount) | standard | 17 | 9 |
| filters.test.tsx | trim | S1 (W3 deleted: empty state sentence) | standard | 8 | 7 |
| focus.test.tsx | keep | S1 | standard | 2 | 2 |
| handing.test.tsx | keep | S1 | standard | 2 | 2 |
| keys.test.tsx | trim | S1 (W10 deleted: hint wording; W4 deleted: menu item run, same as row-menu) | standard | 12 | 10 |
| launch-box.test.tsx | trim | S1 (W3 deleted: command, cwd and link shown after mount) | standard | 7 | 6 |
| model-view.test.ts | keep | K1, K2, K3 | critical | 14 | 14 |
| model.test.ts | keep | K1 | critical | 15 | 15 |
| note-dialog.test.tsx | keep | S1, S3 | standard | 7 | 7 |
| note-editor.test.tsx | trim | S1 (W3 deleted: toolbar, rows, maxLength, hint; W10 deleted: placeholders; W4 deleted: Enter and Escape, already in outcome-buttons) | standard | 8 | 3 |
| note-limits.test.tsx | trim | S3 (W3 deleted: screenshot thumbnails in the note dialog) | standard | 3 | 2 |
| outcome-buttons.test.tsx | trim | S1 (W3 and W10 deleted: placeholder, aria-pressed and which buttons remain) | standard | 11 | 9 |
| outcome-extras.test.tsx | keep | S1, S3 | standard | 8 | 8 |
| outcome-refusal.test.tsx | trim | S3 (W4 deleted: long note cases, already in note-limits) | standard | 5 | 3 |
| play.test.tsx | trim | S1 (W3 deleted: idle state; W9 deleted: queued and launching labels, covered by runLine in model.test.ts) | standard | 12 | 9 |
| row-menu.test.tsx | trim | S1 (W3 deleted: menu contents; W4 deleted: Run again, same as outcome-extras) | standard | 5 | 3 |
| views.test.tsx | trim | S1 (W4 deleted: no test at all, same as bundle-cards) | standard | 9 | 8 |
| wake-chip.test.tsx | trim | S1 (W9 deleted: waking label, covered by wakeChip in model-view.test.ts) | standard | 6 | 5 |

Total: 187 before, 156 after.

### Deleted helpers

none. Helper files inside the folder (board-dom.ts, filter-dom.ts, menu-dom.ts, outcome-dom.ts, fixtures.ts, mount-tab.tsx) are all still imported. One dead export removed: `searchBox` from filter-dom.ts (only used inside the file). Inside test files: `TOOLS`, `toolbarOf`, `PROOFS` and `withProofs` removed; the `text` parameter of `failWith` removed.

### Possibly unused shared helpers

none. (board-dom.ts, fixtures.ts and mount-tab.tsx are imported by apps/web/tests/board-builds and apps/web/tests/remote, so their exports stay. Nothing under apps/web/tests/helpers became unused from this folder: the fakes, snapshot, render-at and launch-docs are still used.)

### Deleted tests that looked critical

none. model.test.ts and model-view.test.ts lost no case. Cases deleted as covered by a pure test: play.test.tsx "running with nothing said yet reads Launching..." and "queued says it waits for a slot" (runLine in model.test.ts); wake-chip.test.tsx "says waking while a queued wake waits" (wakeChip in model-view.test.ts).

### W7

- wake-chip.test.tsx (converted): `fresh()` and the "earlier result" case read `Date.now()`. Now `vi.useFakeTimers({ toFake: ['Date'] })` and `vi.setSystemTime(NOW)` in `beforeEach`, `vi.useRealTimers()` in `afterEach`, and both timestamps derive from a fixed `NOW`. Only Date is faked, so `findBy*` and `waitFor` keep working. The component reads `Date.now()` in useWakeClock and useTestsScreen, so the fixed system time drives it.
- No other sleeps, setTimeout or real clocks were found in the folder.

### Unsure

- wake-chip.test.tsx: relies on `toFake: ['Date']` leaving setInterval real (useWakeClock ticks with setInterval and re-reads the fixed Date, so the chip stays stable). Not run, as ordered.
- drawer-nav.test.tsx "widens the drawer": kept with localStorage and aria-pressed assertions only, after removing the `.drawer--xl` and `.drawer--top` class checks.
- Judgement calls kept as S1 although near W3: board-page "counts the tests to run in its badge", conflicts "shows the field by field table in the drawer", closed-bundles "hide again when the toggle is turned off", launch-box retired, missing, none and legacy states (distinct launcher rules, not one state in variants).
- Mounted rules that are K3 or K4 by nature (note length, refusals, file type and size) are titled S3 because they are mounted and tagged standard.
- Nothing was compiled or run. The unused-import scan was done with a script over the folder.

## board-kanban

| file | verdict | rule | tag | cases before | cases after |
|---|---|---|---|---|---|
| kanban-board.test.tsx | trim | S1 (done toggle, stream, group radio, needs you, search, clear filters), K2 (filters stored for the Node page) | standard | 16 | 7 |
| kanban-chrome.test.tsx | trim | S1 (focus mode, link, deep link, [ ] walking, editor Escape) | standard | 11 | 8 |
| kanban-columns.test.tsx | trim | S1 (add to a column, shift range, paused, focus), S3 (stale focus) | standard | 9 | 5 |
| kanban-criteria.test.tsx | trim | K3 (tick, add, edit, remove, move), S1 (inline Markdown) | standard | 9 | 8 |
| kanban-drawer.test.tsx | trim | S1 (drawer data, needs you, stream), K4 (description never markup), K5 (I-86 tests count), K3 (archive, delete with Undo) | standard | 12 | 7 |
| kanban-edits.test.tsx | trim | K3 (title, description edits and the title refusal) | standard | 6 | 3 |
| kanban-model.test.ts | keep | K1 (quiet limits, needs, columns, groupings, drawer tests) | critical | 22 | 22 |
| kanban-moves.test.tsx | trim | K2 (move contract), S1 (order and grouping), S3 (conflict, unreachable) | standard | 11 | 7 |
| kanban-notices.test.tsx | trim | K3 (conflict and refusal notices, bulk) | standard | 7 | 5 |
| kanban-ops.test.tsx | trim | K3 (add item, bulk archive with Undo), S3 (pick forgotten), S1 (card menu) | standard | 13 | 7 |
| kanban-plans.test.ts | keep | K1 (drop plans, op notices, stored page choices, text limits, bulk ops) | critical | 22 | 22 |
| kanban-record.test.tsx | trim | K3 (to-dos, answers, decisions), S1 (activity), S3 (empty post) | standard | 13 | 8 |
| kanban-records-model.test.ts | keep | K1 (presets, range, paused, record), K3 (purge counts) | critical | 8 | 8 |
| kanban-trash.test.tsx | trim | K3 (trash list, restore, agent unarchive, purge answer), K4 (purge confirm by project key) | standard | 10 | 6 |

### Deleted helpers

- kanban-docs.ts: `cardOf` and its `CardData` import (no test used it); `MIN` is no longer exported (still used inside the file).
- kanban-moves.test.tsx: local `pressed` helper and the `Doc`, `Posted` type imports (only the deleted tests used them).
- kanban-trash.test.tsx: local `sleepy` board and the `thread` import.

### Possibly unused shared helpers

none (nothing under apps/web/tests/helpers was touched; fake-source and snapshot are still imported by kanban-harness.tsx)

### Deleted tests that looked critical

- kanban-columns.test.tsx: "counts the quiet items of a column in a chip" - quiet limit rule is the K1 model test (quiet limits), the chip is markup.
- kanban-columns.test.tsx: "adds an item to a thread column ... and to a subject column" - the preset rule is the K1 addPreset test, the post wiring is the status-column test.
- kanban-columns.test.tsx: "counts a pause the owner posted that the board has not applied yet" - K1 pausedThreads test covers it.
- kanban-board.test.tsx: "does not mark an item that waits on the owner" and "puts a quiet tag" - K1 quiet tests cover the rule.
- kanban-board.test.tsx: "stops saying it once the owner answered the question" - K1 needsMap tests cover it, and the drawer test covers the live transition.
- kanban-drawer.test.tsx: "offers Unarchive for an archived item" - W4 variant of the archive test.
- kanban-notices.test.tsx: "says nothing about an op on another collection or one the board applied" - K1 opNotices skip test covers it.
- kanban-notices.test.tsx: "shows only the cards with a conflict for the conflict filter" - K1 view filter test covers the rule.
- kanban-ops.test.tsx: "deletes them with restore as the Undo" (bulk) - K1 bulkSofts test covers the inverse, the archive test covers the Undo wiring.
- kanban-ops.test.tsx: "archives and deletes with the same ops and Undo as the drawer" (card menu) - W4 of the drawer tests.
- kanban-trash.test.tsx: "purges a thread chosen in the dialog" and "never counts the coordinator thread" - K1 purgeCounts and purgeCriteria tests cover the rules; the trash purge test keeps the confirm gate.
- kanban-trash.test.tsx: "restores a range of rows picked with shift-click" - rangeOf K1 test plus the single restore test.
- kanban-moves.test.tsx: "gives only the status for a column with no card", "moves to another column without a place when the order is by update" - K1 planDrop tests cover both. "says a refusal with its reason" - W4 of the conflict message test.
- kanban-edits.test.tsx: "refuses a description over 400 characters" and "edits the subject, and may empty it" - K1 textProblem tests cover the rule, title refusal test keeps the wiring.
- kanban-criteria.test.tsx: "cancels an edit with Escape" - W4 of the unchanged-edit test and the description Escape test.
- kanban-record.test.tsx: "refuses a to-do over 200 characters" - K1 textProblem-style rule, W4 of the other refusals.

### W7

- Real clock: the harness `ago` helper read `Date.now()`. Converted: `ago` now counts from a fixed `CLOCK` (2026-10-26T12:00:00Z) and `cleanKanban` calls `vi.setSystemTime(CLOCK)` in `beforeEach` (Date only, real timers stay so `waitFor` and `findBy*` keep working) and `vi.useRealTimers()` in `afterEach`. No test was deleted for W7.
- No test sleeps or uses setTimeout. `waitFor`/`findBy*` on outcomes stay.

### Unsure

- `vi.setSystemTime` without `vi.useFakeTimers` mocks only `Date`; I could not run it. If Tessera toast code or a data layer timer relies on real `Date.now()` advancing, a test may need the clock moved on. Easy rollback: remove the `beforeEach` and the `useRealTimers` line from `cleanKanban` and restore `ago` to `Date.now()`.
- kanban-drawer.test.tsx keeps "counts no archived test ..." as K5 because it names I-86; I added "on I-86" to its title. If I-86 was not a real bug, it still stands as S2 against the K1 itemTests tests, so it could go.
- kanban-chrome.test.tsx "copies the page address" asserts only `clipboard.writeText` (W6 shaped), kept because the copied link is the outcome.
- The describe prefix "K2/K3/K4" on mounted files carries the `standard` tag as the brief says.

## board-agents, board-builds, board-thread

| file | verdict | rule | tag | cases before | cases after |
|---|---|---|---|---|---|
| apps/web/tests/board-agents/agents-actions.test.tsx | trim | S1, K3 (stop request, archive, note edit with Undo), S3 (conflict) | standard | 14 | 10 |
| apps/web/tests/board-agents/agents-anchor.test.tsx | trim | S1 | standard | 2 | 1 |
| apps/web/tests/board-agents/agents-live.test.tsx | trim | S1, K4 (markup drawn as text) | standard | 10 | 3 |
| apps/web/tests/board-agents/agents-model.test.ts | trim | K1 | critical | 21 | 20 |
| apps/web/tests/board-agents/agents-reconcile.test.tsx | trim | S1, S2 (reconciler report route) | standard | 7 | 4 |
| apps/web/tests/board-builds/builds-list.test.tsx | trim | K1 (natural name order), K4 (text not markup) | standard | 9 | 2 |
| apps/web/tests/board-builds/builds-model.test.ts | keep | K1 | critical | 12 | 12 |
| apps/web/tests/board-builds/builds-play.test.tsx | trim | S1, K2 (run start/stop requests), K4 (no Play for a draft bundle) | standard | 15 | 6 |
| apps/web/tests/board-builds/builds-runs.test.tsx | trim | S1, K3 (screenshot upload), K4 (10 MB refusal) | standard | 6 | 4 |
| apps/web/tests/board-builds/builds-terminal-live.test.tsx | trim | S1, S2 (log stream), S3 (failed load, refused stdin) | standard | 9 | 7 |
| apps/web/tests/board-builds/builds-terminal.test.tsx | trim | S1, K2 (tail=300 contract) | standard | 8 | 3 |
| apps/web/tests/board-builds/builds-texts.test.ts | trim | K1 | critical | 8 | 7 |
| apps/web/tests/board-builds/builds-usage.test.tsx | trim | S1, K3 (retired toggle saved and restored) | standard | 11 | 3 |
| apps/web/tests/board-thread/thread-actions.test.tsx | trim | S1, K3 (pause, resume, archive, edits), K4 (lead guard, length refusals) | standard | 16 | 14 |
| apps/web/tests/board-thread/thread-drawer.test.tsx | trim | S1, S3 (dropped thread) | standard | 9 | 6 |

Totals: 157 cases before, 102 after.

### Deleted helpers

- `cardOf` in `apps/web/tests/board-builds/list-dom.ts` (no test uses it any more; `withDefs` stays).
- `waitForCard` (local to builds-usage.test.tsx).
- `export type { Posted }` in `apps/web/tests/board-agents/agents-harness.tsx` (the type is only used inside the file).
- The exports `sendLine` and `summaryOf` of `apps/web/tests/board-builds/terminal-dom.ts` (no importer left; the functions stay for internal use).
- No helper file was deleted: every helper file in the three folders is still imported.

### Possibly unused shared helpers

none (nothing under apps/web/tests/helpers or board-tests was left unreferenced by my edits; those files were not touched).

### Deleted tests that looked critical

- agents-model.test.ts: "tones a model by family: sonnet green, haiku blue, opus and others neutral" - W5, tests the MODEL_LOOKS constant and a trivial tagLook mapping.
- builds-texts.test.ts: "says why Play is missing" - W10, asserts one prose string (the same hint is still asserted in builds-play "with no run definition Play is disabled").
- builds-terminal.test.tsx: "draws colour codes as colours, stderr tinted and system lines dim" - W3, asserts CSS class names.
- builds-play.test.tsx: "a start the server refuses leaves Play in place" - W8, asserted only that Play was enabled, which it always is, so it could not fail.
- agents-reconcile.test.tsx: "shows no panel when the report is empty or the route is missing" - W8/W7, asserted absence right after a fetch mock was called, so it could pass before the response was handled.
- builds-usage.test.tsx: "a keep holds a launcher fresh until it ends", "shows the 7d default of a launcher that has no useEvery", "shows the window; unused past it, a stale tag and the header count say so" - W4/W9, the same rules are asserted in builds-model.test.ts (launcherUsage).
- builds-list.test.tsx: "pins master first, then the current ones, then the rest by name, and counts them" - W4/W9, covered by builds-model "launcherViews" and builds-texts "countsText".
- builds-play.test.tsx: "shows only the newest of several finished runs next to Play", "a run of another definition does not show on this launcher", "a retired launcher has no Play at all" - W4/W9, the rules live in launcherViews (builds-model: shown run, runs per launcher, playable false for retired).
- agents-live.test.tsx: "shows the Now line ...", "sorts the working agents by start", "shows the final message of a finished agent ..." - W9, agentNow, agentMessage and sortActive are K1 tested in agents-model.test.ts.
- thread-actions.test.tsx: "offers Unarchive for an archived thread and no Pause or Focus" - W4, a state variant of the archive/pause tests.

### W7

- No test was converted or deleted for W7 alone. Fixtures (`ago` in agents-harness and builds fixtures, `new Date()` in thread-harness) build timestamps relative to the real clock and the assertions only use coarse spans (10m, 4m), so the kept tests do not depend on timing luck. I left them as they are.
- Deleted for the W7-like race: agents-reconcile "shows no panel when the report is empty or the route is missing" (see above).
- Deleted for real clock in the test body: builds-usage "a keep holds a launcher fresh until it ends" (used `Date.now()` inside the test; the rule is K1 tested with a fixed NOW in builds-model).

### Unsure

- Nothing was run, so imports were checked by reading only. Imports removed because no kept test uses them: `waitFor`, `rowOf`, `mockFetch` from agents-reconcile; `screen` from agents-live; `within`, `ago`, `MIN` from builds-play; `runningDoc` from builds-runs and builds-terminal-live; `runDoc`, `waitFor` from builds-terminal; `within`, `buildDoc`, `cardOf` from builds-list; `within`, `buildDoc`, `cardOf` from builds-usage; `MODEL_LOOKS`, `tagLook` from agents-model; `noPlayHint` from builds-texts.
- Kept as S1 although they lean on text or state: agents-actions "counts the working, waiting and stopping agents in the header" (a computation shown), agents-live "marks a stalled working agent and leaves a busy one alone", agents-reconcile "marks a duplicate, a lost record and the mismatches in the drawer" (one test, three marks), thread-drawer "counts only the open questions and the pending decisions". If the lead wants a stricter line, these four are the first to go.
- builds-play "asks for a console window only when the definition has window" is the boundary of the hidden/visible start rule and sits beside the default test; it could be read as W4.
- Describe `K4 where Play is hidden` in builds-play.test.tsx carries the standard tag (it mounts a tree) while the K4 prefix names the rule.

## Worktrees, board, data and format (apps/web/tests/board-worktrees, board, data, format)

| file | verdict | rule | tag | cases before | cases after |
|---|---|---|---|---|---|
| board-worktrees/hygiene.test.tsx | trim | S1 | standard | 4 | 1 |
| board-worktrees/leftover-delete.test.tsx | trim | K3, K4 | standard | 11 | 8 |
| board-worktrees/leftover-rows.test.tsx | trim | K4 | standard | 10 | 4 |
| board-worktrees/live.test.tsx | trim | S1 | standard | 5 | 2 |
| board-worktrees/model.test.ts | trim | K1 | critical | 22 | 20 |
| board-worktrees/remove.test.tsx | trim | K3, K4 | standard | 12 | 6 |
| board-worktrees/request.test.tsx | trim | K3, K4 | standard | 12 | 5 |
| board-worktrees/requests-list.test.tsx | trim | S1, K4 | standard | 4 | 2 |
| board-worktrees/rows.test.tsx | trim | K4, S1 | standard | 13 | 4 |
| board-worktrees/rules.test.tsx | trim | S1 | standard | 6 | 1 |
| board/board-notices.test.tsx | trim | K1, S1 | critical (1), standard (1) | 3 | 2 |
| board/board-shell.test.tsx | trim | S1 | standard | 8 | 2 |
| board/feed-model.test.ts | keep | K1 | critical | 16 | 16 |
| board/owner-entry.test.ts | trim | K1 | critical | 8 | 7 |
| board/owner-feed.cards.test.tsx | trim | S1, K3, K4 | standard | 13 | 8 |
| board/owner-feed.images.test.tsx | trim | S1 | standard | 3 | 1 |
| board/owner-feed.info.test.tsx | trim | S1, S3 | standard | 10 | 8 |
| board/owner-feed.log.test.tsx | trim | S1 | standard | 10 | 2 |
| board/plain-text.test.ts | keep | K1 | critical | 5 | 5 |
| board/presence.test.tsx | trim | S2, K4 | standard | 4 | 3 |
| board/registered-strip.test.tsx | trim | S1, K1 | standard (4), critical (2 in the chip look describe) | 12 | 5 |
| data/hooks.test.tsx | trim | S2 | standard | 5 | 4 |
| data/runs-order.test.ts | keep | K1 | critical | 2 | 2 |
| format/format.test.ts | keep | K1 | critical | 12 | 12 |

Totals: 210 cases before, 130 after. Files deleted: none (every file keeps at least one case).

### Deleted helpers

none. Tidied inside apps/web/tests/board-worktrees: `postedBodies` removed from wt-dom.ts (no user left), the `SCOPE` value and the `Call` type are no longer exported from mount-tab.tsx, the `REMOVE` constant and the `removed` and `request` leftovers that only deleted cases used are gone. Unused imports were removed from every trimmed file.

### Possibly unused shared helpers

none. The shared helpers (board-docs.ts, open-board.tsx, post-mock.ts, seed-board.ts, render-at.tsx, etc.) are still imported by kept cases. Possibly unused exports of `apps/web/tests/helpers/board-docs.ts` after this pass: `oldDecision` (its only user here, owner-feed.info, was deleted; other folders may still use it, grep before touching), `NOW` (no longer imported by owner-feed.log; other folders may).

### Deleted tests that looked critical

- board-worktrees/model.test.ts: "labels: Remove, Delete now, Delete leftover" - W9, the labels are reached by the Remove and Delete now clicks in remove.test.tsx and rows.test.tsx.
- board-worktrees/model.test.ts: "names the states: in use, released, review and kept" - W5, a text mapping of a getter, the hygiene component test shows the same labels.
- board-worktrees/remove.test.tsx: "a new HEAD blocks Confirm too, and reopening the confirm reads the new state" and "a row that can no longer be removed blocks Confirm" - W4, the same stale-confirm rule as the kept dirty case, and confirmStale is covered as K1 in model.test.ts.
- board-worktrees/remove.test.tsx: "sends the HEAD and the dirty count of the moment the confirm opened" - W4, the same body is asserted in the kept Delete now Confirm case.
- board-worktrees/leftover-delete.test.tsx: "a partial folder says a repository deeper down would not have been seen" - W10 prose, leftoverFlags covers the partial flag as K1.
- board-worktrees/request.test.tsx: "takes the thread from the rule when the row has none" and "says there is no thread to tell when the rule names none" - W9/W4, threadOf and askProblem are K1 in model.test.ts.
- board-worktrees/rows.test.tsx and leftover-rows.test.tsx: "posts nothing on its own" (two files) - W8, only asserts that nothing happened after mount.
- board-worktrees/hygiene.test.tsx: "never shows the three day active state" and "keeps the staleness tag only on a row in review or without a hygiene state" - W4, statusShown is K1 in model.test.ts.
- board/owner-entry.test.ts: "the safety net is 50000 characters" - W5, a constant (the boundary is still asserted by textProblem).
- board/owner-feed.cards.test.tsx: "the own answer field is the markdown editor with its toolbar and the sentence for a text over the limit" - W3/W4, the toolbar is markup and the over-limit sentence is asserted in the kept 50000 case.
- board/owner-feed.info.test.tsx: "an old decision without options is dismissed too" and "a notice or message without a context shows no missing context line" - W4, askMode and hasContext are K1 in feed-model.test.ts.
- board/owner-feed.log.test.tsx: "the tests of one bundle are ONE card with a count", "a test with no bundle is its own line", "a closed bundle leaves the feed" - W4, the same rules are K1 in feed-model.test.ts.
- board/owner-feed.log.test.tsx: "lists the newest lines first with the thread of each" - W4, ordering is K1 in feed-model.test.ts.
- board/registered-strip.test.tsx: the five ChipTip tooltip cases (fault reason, Overlord fault, closed board fault, state sentences, board tab closed) - W10 prose wording of tooltips.
- board/registered-strip.test.tsx: "updates live: a fault that clears..." - W4 with the kept "follows the registry stream" case.
- board/presence.test.tsx: "opens nothing without a thread in the URL" - W7/W8, asserted absence after a single microtask tick.
- data/hooks.test.tsx: "useStreamStatus reports the connection" - W5, a status getter of the stream (the stream state is covered where the stream is tested).

### W7

- board-worktrees (all ten files): converted. `cleanAfterEach()` in mount-tab.tsx now freezes only `Date` (`vi.useFakeTimers({ toFake: ['Date'] })` plus `vi.setSystemTime`) to a constant in a `beforeEach`, and restores real timers in the `afterEach`. `fixtures.ts` builds every timestamp (`ago`) from the same constant `NOW` instead of `Date.now()`. `hygiene.test.tsx` builds `goesAt` from `NOW`, `leftover-rows.test.tsx` and `requests-list.test.tsx` use `ago()` instead of `new Date()` and `Date.now()`. `model.test.ts` passes `NOW` to `refusedJobs`. Timers (setTimeout, setInterval) are not faked, so `waitFor` and `findBy*` still work.
- board/plain-text.test.ts "stays quick on a hostile input": converted. The `Date.now()` stopwatch and the 3000 ms assertion are gone; the case now just runs the hostile input and asserts a non-empty result, the critical tag timeout (5 s) is the bound.
- board/presence.test.tsx "opens nothing without a thread in the URL": deleted (see above).
- Left as is: `Date.now()` / `new Date()` used only to build timestamps relative to now, with no wait, race or timer: board/board-notices.test.tsx (`NOW`, `at()`), board/board-shell.test.tsx (`NOW`), board/registered-strip.test.tsx (`minutesAgo`). The `waitFor` and `findBy*` calls everywhere wait on a rendered outcome.

### Unsure

- Nothing was run. The fake `Date` in the board-worktrees `beforeEach` is the riskiest edit: if a component there reads time through something other than `Date` (it should not), or `waitFor` misbehaves with a frozen `Date`, drop the `beforeEach` in `cleanAfterEach` and keep the fixed `NOW` (the relative strings would then drift by the real run time, as before).
- plain-text.test.ts hostile case: the asserted output is only "not empty"; if the checker finds it too weak (W8), delete it, the ReDoS guard then rests on the other cases.
- board/registered-strip.test.tsx holds one file with both tags (standard describes and one critical describe for the pure chipLook); the chipLook "maps every state to a tone, a mark and a word" case is a mapping table that the checker may judge W5/W3.
- board-worktrees/hygiene.test.tsx keeps the `f` (main) row in `docs()` that no assertion reads any more; harmless, not removed to keep the fixture shape.

## apps/web/tests/overview

| file | verdict | rule | tag | cases before | cases after |
|---|---|---|---|---|---|
| activity.model.test.ts | keep | K1 | critical | 7 | 7 |
| activity.tab.test.tsx | trim | S1 | standard | 11 | 6 |
| badges.test.tsx | trim | S1 | standard | 12 | 7 |
| cache.model.test.ts | keep | K1, K4 | critical | 5 | 5 |
| cache.tab.test.tsx | trim | S1, K3 | standard | 11 | 8 |
| dashboards.master.test.tsx | trim | S1 | standard | 4 | 3 |
| dashboards.model.test.ts | keep | K1 | critical | 21 | 21 |
| dashboards.send.test.tsx | trim | S1 | standard | 8 | 5 |
| dashboards.tab.test.tsx | trim | S1 | standard | 8 | 3 |
| notifications.model.test.ts | keep | K1 | critical | 12 | 12 |
| notifications.tab.test.tsx | trim | S1 | standard | 15 | 8 |
| processes.model.test.ts | trim | K1, K4 | critical | 25 | 21 |
| processes.tab.test.tsx | trim | S1, K4 | standard | 17 | 6 |
| seen.test.tsx | keep | S1 | standard | 7 | 7 |

Total: 163 before, 119 after.

### Deleted helpers

none

### Possibly unused shared helpers

none (apps/web/tests/helpers/whole-text.ts is no longer imported from overview, but apps/web/tests/remote/sync-stamp.test.tsx still uses it)

### Deleted tests that looked critical

- processes.model.test.ts: "shows the project by its name" (toProcRow) - W5, a plain field mapping the table-row test already reaches
- processes.model.test.ts: killOne "posts the pid, the startedAt exactly as sampled and tree false" - W6/W4, only asserts the mock call; the Processes tab kill test posts and checks the same body through the UI
- processes.model.test.ts: killOne "posts tree true for Kill with children" - W6/W4, same, covered by the tab test "posts tree true"
- processes.model.test.ts: killStale "posts the rule with idleOnly and counts what was killed" - W6/W4, covered by the tab test "offers Kill stale"
- dashboards.tab.test.tsx: "counts the running leads and what needs you across them" - W9, totalsOf is tested in dashboards.model.test.ts
- dashboards.tab.test.tsx: "shows the state, the counters and the flags of a card" - W3/W9, cardCounts and attentionOf are tested in the model
- dashboards.tab.test.tsx: "writes how long a wake has been on its way as a live time" - W7/W9, relative minutes off the real clock; registryChip is tested in the model
- processes.tab.test.tsx: "says what a 403 and a 404 mean in a toast" - W4, the same sentences are asserted by killOne in processes.model.test.ts
- notifications.tab.test.tsx: "says why there was no toast, and that it is recorded" - W9, testSentence is tested in the model
- notifications.tab.test.tsx: "reads the settings when the tab opens, not before" - W6, only counts fetch calls
- cache.tab.test.tsx: "shows the totals, the folder and the grace period" - W3/W9, text of numbers formatBytes already covers in cache.model.test.ts

### W7

- seen.test.tsx: converted. The file now fakes only Date (`vi.useFakeTimers({ toFake: ['Date'] })` plus `vi.setSystemTime`) and the time helper is fixed to a constant NOW; the `> Date.now() - 5000` assertion is now an exact `toBe(new Date(NOW).toISOString())`.
- badges.test.tsx: converted. The "later" entry uses a fixed far-future timestamp instead of `Date.now() + 60000`.
- activity.tab.test.tsx: converted. The stream entry uses the `minutesAgo(0)` fixture instead of `new Date()` in the test.
- cache.tab.test.tsx: converted. Dates use a fixed base (`ago`), `LATER` is a fixed far-future string, and `PRUNED.at` is a fixed string, so no `Date.now` is left in the file.
- dashboards.master.test.tsx: converted. The `running 2m` text assertion (real clock minutes) was replaced by awaiting the Stop button.
- dashboards.tab.test.tsx: converted. The chip link query no longer matches `10m ago`.
- deleted for W7: dashboards.send.test.tsx "counts a queued run from the moment it was queued ..." (`6m` from a 5.5 minute real-clock offset, plus column labels, W3); dashboards.tab.test.tsx "writes how long a wake has been on its way as a live time".

### Unsure

- Relative times in the shared fixtures (helpers/fixtures.ts `minutesAgo` uses the real clock) are still used by kept tests, but no kept assertion depends on a minute value except the activity row regex `\d+[smhd] ago`, which holds for any clock.
- cache.tab.test.tsx "shows a row per version ..." and activity.tab / notifications.tab "shows the kind/title ... project link" are close to W3 (data rendered after mount); kept because they assert the project link and per-row data from the report that no other test reaches.
- seen.test.tsx is tagged standard (renderHook plus jsdom localStorage), not critical.
- Nothing was run, so the edits are not compile-checked; imports were adjusted by hand (removed `cleanup`, `waitFor`, `within`, `wholeText` where no kept test uses them).

## markdown, remote, palette, routing, stream, api

| file | verdict | rule | tag | cases before | cases after |
|---|---|---|---|---|---|
| markdown/markdown-blocks.test.tsx | trim | S1 | standard | 17 | 17 |
| markdown/markdown-cost.test.tsx | trim | K4 | standard | 3 | 1 |
| markdown/markdown-editor.test.tsx | trim | S1 | standard | 16 | 14 |
| markdown/markdown-inline.test.tsx | keep | S1 | standard | 21 | 21 |
| markdown/markdown-safety.test.tsx | keep | K4 | standard | 7 | 7 |
| remote/banner.test.tsx | trim | K1, S1, S2 | critical, standard | 12 | 10 |
| remote/changes.test.tsx | trim | S1, K4 | standard | 14 | 8 |
| remote/model.test.ts | keep | K1 | critical | 6 | 6 |
| remote/registry.test.tsx | trim | S1 | standard | 6 | 5 |
| remote/releases.test.tsx | trim | S1 | standard | 8 | 5 |
| remote/sync-stamp.test.tsx | trim | K1, S1, S2 | critical, standard | 11 | 7 |
| palette/palette.test.tsx | trim | K1, S1 | critical, standard | 5 | 4 |
| routing/routing.test.tsx | keep | S2 | standard | 6 | 6 |
| stream/scope-stream.test.ts | keep | K1, S3 | critical, standard | 10 | 10 |
| api/api.test.ts | keep | K2 | critical | 8 | 8 |

Totals: 150 cases before, 129 after (markdown 64 to 60, remote 57 to 41, palette 5 to 4, routing 6, stream 10, api 8).

### Deleted helpers

none

### Possibly unused shared helpers

none found (the shared helpers I use are all still imported: board-tests/board-dom, helpers/render-at, helpers/fake-source, helpers/snapshot, helpers/seed-board, helpers/whole-text, overview/helpers/*).

### Deleted tests that looked critical

- markdown/markdown-cost.test.tsx: "stays fast: <name>" (24 cases, one it.each) - asserted a real-clock time under 5 s (W7); replaced by one it.each that renders each hostile input and asserts no script or event attribute (K4), the test timeout bounds the cost.
- markdown/markdown-cost.test.tsx: "time grows with the size, not with its square" - real clock ratio, flaky by nature (W7).
- markdown/markdown-cost.test.tsx: "the output of the long inputs is still free of scripts and event attributes" - now covered by the converted it.each over the full inputs (W4).
- remote/changes.test.tsx: "derives the state of a step from its exit code when it has no status" - stepStatus is already tested in remote/model.test.ts (W9).
- remote/sync-stamp.test.tsx: "turns amber at 20 hours and red past 24 hours" (stamp), "turns amber at 20 hours" and "turns red past 24 hours" (header) - the boundaries are K1 cases of syncTone, the tones still show through the overview cards test (W4, W9).
- remote/banner.test.tsx: "asks again every 60 seconds" - only asserted that setInterval was called with 60000 (W6, W7).

### W7

- Deleted: markdown/markdown-cost.test.tsx real-clock timing tests (performance.now): the 24 "stays fast" cases and the growth ratio test. The hostile-input cost is now bounded by the standard test timeout, with the K4 output check kept.
- Deleted: remote/banner.test.tsx "asks again every 60 seconds" (spied window.setInterval).
- Converted: remote/sync-stamp.test.tsx now fakes only Date (vi.useFakeTimers with toFake: ['Date'] and vi.setSystemTime(NOW)) so the sync ages are fixed instead of derived from the real clock; waitFor and findBy keep their real timers. hoursAgo, the board header updatedAt and the overview cards seen stamp use NOW.
- Not converted, left as is: remote/helpers/fixtures.ts `ago` uses Date.now() to build relative timestamps for changes, registry and releases tests ("5m ago", "3m"). The values are minutes apart, so they hold unless a test takes about a minute; it is a shared helper in my folder and the fix is a one-line fake Date in each file if wanted.

### Unsure

- K4 describes (markdown-safety, markdown-cost, the changes title escaping test) carry the tag standard because they mount a tree, as the brief says; K4 is a critical rule in the standard table, so a lint that ties K-rules to the critical tag would flag them.
- api/api.test.ts: each case asserts the request path, method and body sent to fetch plus the parsed result. Kept as K2 (the wire contract with the Rust server). If the lead reads it as W6, the whole file goes.
- stream/scope-stream.test.ts: Connection and State are tagged critical (in-memory fake source, no timers); Recovery uses fake timers and a stubbed fetch, tagged standard (S3).
- remote/releases.test.tsx lost its version-order assertion (it read the data-version attribute, W3); ordering stays covered by sortVersions in remote/model.test.ts. The beta channel in CHANNELS is now only used by the overview test.
- banner.test.tsx has a critical describe (bannerText) and standard describes in one .tsx file; the critical one imports no DOM but the file does.
- Nothing was run; imports were checked by hand (changes.test.tsx dropped the unused `replay` import).
