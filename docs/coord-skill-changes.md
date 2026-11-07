<!-- @layer docs @kind doc -->
# Changes for the coord skill and the working standard

A request from the Argus lead to the Overlord, item I-70 (the item-type rework). The coord skill
lives in `X:/ai-config/shared/skills/coord`, and the project working standard is the section in
each project `CLAUDE.md`. Both are yours to edit. Every change below is complete as written: you do
not need to read the Argus code. The Rust `coord` has all the verbs and flags listed here; the
authority for the exact text of each is `coord --help` and `coord types`.

Do not apply any of this before the Argus build with the rework is deployed locally: the old Node
CLI refuses the new flags.

## 1. What changed

- Every work item now has a type: `epic`, `story`, `task`, `bug`, `debt` or `test-plan`. No type
  means `task`, so old calls keep working.
- An item can have a parent (epic, then story, then a leaf), an assignee, a stage and typed fields.
- A blocker is always an id. Free text is gone.
- Test plans group test cases and keep runs.
- There are informal notes, assignable to-dos and pictures on items.
- Six new wake reasons tell a thread when something happened to its items.
- The old field `subject` is retired: no verb writes it and no view shows it.

## 2. `SKILL.md`

Add a short section "Item types" with these points.

1. Every item has a type. `coord types` lists them and `coord types <type>` shows the stages, the
   fields, the parents and the done rule of one. The usage text is generated from the same registry,
   so `coord item --help` is always right.
2. The hierarchy is epic, then story, then a leaf (task, bug, debt or test plan). A leaf may sit
   straight under an epic. Any type may be a root. A parent is on the same board; work on another
   board stays `coord dep ask`.
3. `--assignee` says who holds the ball: a thread of the board, `owner`, and for to-dos only
   `lead:<key>`. `--for <thread>` still names the accountable thread and also sets the assignee.
4. Stages map to the five lanes. `coord item status I-n <lane>` keeps working. A stage name is also
   accepted.
5. Done rules. A `warn` rule prints what is open and moves anyway. A `refuse` rule stops the move.
   `--force "<reason>"` passes a refusal and records a decision `D-n` on the item.
6. The no-text-blocker rule (section 4 below).
7. `subject` is retired. Do not set it. Epics do its job.

## 3. `thread-verbs.md`

Replace the `coord item` block with this and add the new verbs. All old forms still work.

```
coord item add "<title>" [--type <t>] [--parent <I>] [--assignee <thread|owner>] [--stage <s>]
               [--field k=v]... [--image <file> "<caption>"]... [--force "<reason>"]
coord item set <I> [--type t] [--parent <I>|none] [--assignee who|none] [--stage s]
               [--field k=v]... [--image <file> "<caption>"]... [--force "<reason>"] [--if-rev n]
coord item status <I> <lane|stage> [--force "<reason>"]
coord item block <I> --on <I|T|H|Q|D-n> | --todo "<what>" --who <owner|thread|lead:key>
coord item unblock <I> [--from <id>]
coord item tree [<I>] [--type t] [--open] [--assignee who|me]
coord item list [--type t] [--assignee who|me] (with the older filters)
coord item migrate-blockers [--dry-run]
coord types [<type>]
coord note <I|T> "<text>"
coord action add "<step>" [--item <I|T>] [--who owner|<thread>|lead:<key>]
```

Notes to add under the block:

- No `--type` is a task. Each field of a type also has a named flag, for example `--severity S2`
  and `--repro "..."`. Read `coord types bug` for the list.
- `--field` keys and values are checked by the registry and a refusal says which rule failed.
- `--image` is repeatable. The captions are the words after the title (add) or after the item id
  (set), one per flag, in order. Files are png, jpg, gif or webp, up to 10 MiB, 12 to an item.
- `coord item tree` prints one line per item, children indented:
  `I-90 [epic] active  ME  Release 2  7/10` (id, type, stage, assignee, title, children done over
  children). Use it in a standup.
- `coord note <I|T> "<text>"` keeps an informal remark (300 characters, id `N-n`). A note never
  blocks the item. `coord ask` and `coord decide` also take a `T-n`.
- `coord action add ... --who` gives a to-do to the owner (the default), a thread, or
  `lead:<key>`, the lead of another project, who is woken. A to-do blocks an item only when it is
  linked as a blocker.
- Rich text fields (description, goal, repro and the like) follow one rule: bullets and images, at
  most one short lead line, about 500 characters of text and 6 images. A lone paragraph is refused.
  Write a description as a few bullets.

## 4. The blocker rule (SKILL.md, `thread-verbs.md`, and the working standard)

This is a breaking change. `coord item block I-n --on "<free text>"` now fails with a sentence that
says what to do. The forms are:

```
coord item block I-n --on <id>                       an existing record of this board
coord item block I-n --todo "<what>" --who owner|<thread>|lead:<key>
coord item unblock I-n [--from <id>]
```

What to use instead of text:

| The wait is | Use |
|---|---|
| a test the owner must run | `--on T-n` |
| a go, a permission or a decision from the owner | `--todo "<what>" --who owner`, or `coord ask` (it blocks by itself) |
| another lead's work | `coord dep ask <key> "<what>" --item I-n` (not `--on`; an id with a project key is refused) |
| the lead's own later step, such as the daily sync | `--todo "<what>" --who <the lead's own thread id>` |
| another item, question, to-do or decision of this board | `--on <id>` |

Settled links (done, dropped, passed, answered) never count as blocking.

One-off migration of the old texts: each lead runs this once on its own board after the deploy.

```
coord item migrate-blockers --dry-run     prints one line per change, writes nothing
coord item migrate-blockers               applies them
```

Rules of the migration: an id of this board becomes a link; a ledger `D-n` is dropped since the
dependency already blocks; `external: owner ...` becomes a to-do for the owner; any other text
becomes a to-do for the item's thread; text on a done or dropped item is cleared. Tell the leads to
run it after the owner's go.

The working standard line in every project `CLAUDE.md` (the section "Never wait", the bullet
"A lead is idle only when its board is empty") now reads:

> Every open item is either being worked now (a running agent or your own step) or carries a
> recorded blocker: a dependency (`coord dep`), an owner ask, or a named wait, recorded with
> `coord item block I-n --on <id>` or `coord item block I-n --todo "<what>" --who owner|<thread>|lead:<key>`
> (`unblock` when it ends).

The old text was `coord item block I-n --on "D-n | Q-n | external: who, what"`. Replace it
wherever it appears, including the Overlord's own board-cleaning notes.

## 5. `coordinator.md` and `coordinator-board-admin.md`

- When to make each type. An epic is a goal that takes several stories (a release, a rework). A
  story is a user need, "As a ..., I want ..., so that ...", that several tasks serve. A task is
  one piece of work. A bug has a severity, steps and the expected and actual result. A debt is
  work we choose to do for the code, not for a user, with an area, a cost and a risk. A test plan
  is a set of test cases run together on a bundle.
- Parents. Put a story under an epic, and a task, bug, debt or test plan under a story or an epic.
  Depth is at most three. A parent is never blocked by its children; its done rule reads them.
- Done rules. A task and a debt `warn` until the criteria are checked. An epic is refused while a
  child is open. A story is refused until its criteria are checked, its children are settled and
  every test that verifies it passes. A bug is refused until a linked test passes (or the owner
  closed it himself). A test plan is refused until its last run is complete and every case passes.
  `--force "<reason>"` passes a refusal and the reason is kept as a decision on the item. Use it
  rarely and say why.
- Retyping an item (`coord item set I-n --type t`) is allowed when its parent and children stay
  valid for the new type.
- Board cleaning. An open item with neither a worker nor a recorded blocker is still the thing to
  find. Blockers are now ids, so `coord item list --open` plus the `blocked` list the board shows
  is the whole check. Run `coord item migrate-blockers` once per board (section 4).

## 6. `coordinator-wakes.md`

Add these reasons. The first column is the exact reason string a wake line carries.

| Reason | Class | Sent to | Means | What to do |
|---|---|---|---|---|
| `item-assigned` | wake | the new assignee | an item was given to you by someone else | read it and start or answer |
| `item-unassigned` | passive | the old assignee | an item was taken from you | note it, no action |
| `todo-assigned` | wake | the thread, or the lead of another project | a to-do was given to you | do it or answer |
| `item-status` | passive | the thread that works it | someone else moved the stage of your item | read it |
| `item-ask` | passive | the thread that works it | another thread asked or wants a decision on your item | answer when you reach it |
| `item-unblocked` | wake | the assignee | the last blocker of your item settled | continue the item |
| `children-done` | wake | the parent's assignee | every child of your epic or story is done or dropped | review and close the parent |
| `plan-run-done` | wake | the plan's thread | a run of your test plan ended, with the counts | read the failures |
| `todo-quiet` | passive | the lead assigned | a to-do of yours that blocks an item stayed pending for a day | do it or say why not |

`item-assigned` already existed for owner posts; it now also covers a change by another thread or
a lead. A change made by the thread that holds the item, or an item given to oneself, wakes nobody. The
owner does not get these as wakes: his are toasts and the "Needs you" count (an item or to-do
assigned to him, a bug he reported ready to verify, a plan ready to run).

## 7. `tests.md`

Add the test plan flow.

```
coord test plan add "<title>" [--parent I] [--bundle <tag>] [--case T]...
coord test plan cases <plan I> --add T [--before T] | --remove T
coord test plan start <plan I> [--bundle <tag>]
coord test plan end <plan I>
coord test plan show <plan I>
coord test add <I> "<title>" ... [--plan <plan I>]
coord test result <T> <pass|fail|partial|blocked|skip> "<note>" [--proof <img>] [--run <plan I>]
```

- A plan is an item of type `test-plan`. Its cases are `T-n` tests, in order. A test may be in
  many plans. A test still verifies one item through its `item` field.
- A plan reaches `ready` with at least one case. `start` opens a run and moves the plan to
  `running`. `end` closes it and moves the plan to `reported`. One run is open at a time.
- `coord test result ... --run <plan I>` records the result in the open run and on the test
  itself. `skip` exists only in the run. A `fail` still needs a note.
- A plan is `passed` only when its last run is complete and every case passed.
- The owner runs a plan from the pane runner in the page; a lead records results with the CLI.

## 8. `portfolio.md`

- Cross-board work stays `coord dep ask`. A parent link and a `--on` link are for the same board
  only, and an id with a project key is refused with that hint.
- A to-do may go to another lead: `coord action add "<step>" --item I-n --who lead:<key>`. That
  lead is woken with `todo-assigned`. It is a request for a small step, not for a work item.
- The Overlord's board cleaning also reads the new `blocked` causes. A blocked item has a named
  cause; an open item with none and no worker is the thing to chase.

## 9. `coordinator-standup.md`

- Use `coord item tree` for the shape of the work, with `--open` to leave out settled items.
- Use `coord item list --assignee owner` for what waits on the owner, and `--assignee me` for the
  thread's own.
- A long chain of dependencies shows as `trail N` on the card in the page; ask the lead of the
  waiting end first.

## 10. Hooks

No hook changes. The hooks call only `coord item status`, `move` and `add` without a type, and
those keep their behaviour and output.

## 11. Order of work for you

1. Wait for the Argus lead to say the rework is deployed locally.
2. Apply sections 2 to 9 and section 12 to the skill, and the blocker line of section 4 to every project
   `CLAUDE.md` working standard.
3. Re-render every project, and tell the leads.
4. After the owner's go on the preview, tell each lead to run `coord item migrate-blockers
   --dry-run`, then without the flag, and `coord build migrate-links --dry-run`, then without the
   flag (section 12).

## 12. Builds and worktrees live with their item (I-78)

Add this to `SKILL.md` and to the build and worktree blocks of `thread-verbs.md`.

```
coord launcher set <name> --cmd "<command>" --for I-n [flags as before]
coord launcher set <name> --master [--cmd "<command>"]
coord launcher link <name> I-n
coord build add|set <tag> --cmd "<command>" [--for I-n]
coord build migrate-links [--dry-run]
coord worktree link <id|path> I-n
coord worktree unlink <id|path>
coord agent start <name> "<task>" ... --item I-n
```

- A build serves one work item and closes with it. `--for` takes an `I-n` of the same board whose
  type is epic, story, task, bug, debt or test-plan, and which is not done, dropped or in the trash.
  A new build without `--for` is refused, except the master build.
- A board has one master build, the main checkout's app: mandatory, no item needed, never closed
  by an item. `coord launcher set <name> --master` hands the role to another build; the build
  that had it is marked `relink` and waits for `coord launcher link`.
- `coord build add <tag> --cmd ...` with no `--for` serves the one test plan whose `bundle` field
  names the tag. With none or several it is refused: pass `--for`.
- When an item reaches done or dropped, or goes to the trash, its builds close in the same write
  and its Play definition is dropped. Reopening or restoring the item brings those builds back.
  The CLI prints `I-7: closed with it: build bundle-7`.
- A worktree is linked to an item too. Start sub-agents with `coord agent start --item I-n` and
  Argus links the worktree they use. `coord worktree link` fixes one by hand. When the item closes
  the worktree is removed after a short grace, with a backup written first, so a lead links a
  worktree it still needs to another open item. A worktree with no item an hour after Argus saw it
  produces a passive `worktree-unlinked` notice.
- Each lead runs `coord build migrate-links --dry-run`, then without the flag, once. It flags the
  master build, links a build whose bundle one test plan names (or whose tests all verify one item)
  and marks the rest `relink`. List them with `coord launcher list` and answer each with
  `coord launcher link <name> I-n`, or retire it.
- The hooks call none of these, so no hook changes.

## 13. Agents are linked records (I-85 P2)

- `coord agent start <name> "<task>" --model m --why "..." --item I-n [--item I-m ...]
  (--worktree <path|auto> | --master) [--blocked-by id ...]` is refused without an item or without a
  place, with the fix in one line. `--worktree auto` means the `agent-<claudeId>` folder the session makes
  for the sub-agent (Argus finds it once the scan sees it); `--master` means the main checkout. Put
  `--master` after the name and the task. `--item` also takes `I-1,I-2`.
- `coord agent set A-n` gains `--item` (replaces the list, `""` clears), `--add-item I-n`,
  `--worktree <path|auto>`, `--master` and `--blocked-by id ...` (replaces, `""` clears). A blocker is
  an I-, T-, H-, Q-, D- or A- id of the board, never text.
- The record carries `items` (`item` stays the first), `master` and `blockedBy`. The hooks still make
  their records without them; start (or `agent set`) fills them in.
- Argus leaves a passive `agent-unassigned` notice for an agent with no item 10 minutes after it
  started. Skill text: start every sub-agent with `--item` and `--worktree auto` (or `--master`).

## 14. Test runs replace bundles (I-99)

A request from the Argus lead to the Overlord, item I-99. A test plan always holds test cases and is
run from them. A run is a `test-run` item that carries its build. The bundle is gone. Apply this
after the Argus build with I-99 is deployed locally. The skill source is
`X:/ai-config/shared/skills/coord/*`. Every file with a bundle mention is listed here.

### New and changed verbs

- `coord test run list|hand|unhand|ready|draft|end|reopen|drop <run I>`. `list` replaces
  `coord build list`. `hand` assigns a ready run to the owner and `unhand` takes it back. `ready`
  and `draft` move a run between draft and ready (ready needs a build that exists and is live).
  `end` finishes it (passed when every case passed, else reported). `reopen` brings back an
  ended run. `drop` abandons it.
- `coord test plan add "<title>" --case T [--case T]... [--build <launcher>]`. A plan needs at
  least one case. Removing the last case is refused. `--bundle` is gone.
- `coord test plan run <plan I> --build <launcher>` starts a run on that build (it replaces
  `plan start --bundle`).
- `coord test add ... --mode ai|manual`. Stored values stay `auto` and `manual`; `ai` is an alias of
  `auto`. The page shows AI or manual on every case.
- `coord test migrate-runs [--dry-run]`: each lead runs it once on its board. It turns every
  plan run and every bundle tag into a run, keeps the data and marks the board. Run it with
  `--dry-run` first.
- `coord build ...` stays for one release as aliases that print the new verb. `coord build add` and
  `coord build set` are gone: make the build with `coord launcher set <name> --for I-n`, then put
  it on the run with `--build`. A build made for a run retires when the run is passed or
  abandoned.
- A test case has no `build` or `bundle` field any more: `--build` and `--clear-build` on
  `coord test add|set` are refused. The build belongs to the run.

### Files and what to change

1. `coordinator-builds.md` (35 mentions, rewrite). Title and intro: hand the owner a run, not a
   bundle. Replace steps 1 to 5 (`build add|set|rm`, `ready|draft`, `hand|unhand`, `list`,
   `close|reopen`) with the `coord test run` verbs above. Drop the "bundle record" text (title,
   note, thread) and the `--cmd`, `--cwd`, `--window` and `--launcher` flags of `build add`. Keep
   the launcher link text: the run's build is a launcher. The automatic ping now reads "a run is
   ready to test" and fires when a ready run is assigned to the owner. Line 66 names `coord build
   hand`.
2. `coordinator-launchers.md` (16 mentions). Lines 47 to 55: "Without a live master `coord build
   hand`" becomes `coord test run hand`. Rule 2 "A launcher made for a bundle goes with it" becomes
   "for a run": a build made with `--for <run>` retires when the run is passed or abandoned and
   `reopen` restores it. Delete the `coord build add|set <tag> --cmd` and "for bundle <tag>" text.
   Line 60: "a bundle's launcher takes none" becomes "a run's build". Lines 75 to 83: drop "Link each
   bundle to its launcher: `coord build set`" and "Relink or close the bundles"; the
   manager-of-the-bundle sentence names the run.
3. `tests.md` (8 mentions). Line 25: the `build` field is a launcher name, not "bundle-1". Lines
   72 to 81: "I was handed xyz" now means a run handed by `coord test run hand`; the examples use a
   launcher name and `coord test run list`, not `coord build list`. Add the `mode` row (AI or
   manual) and the "plan needs a case" rule. Replace `--bundle` and `plan start` in the plan text.
4. `SKILL.md` lines 132 and 133: "Bundles and builds" becomes "Test runs and builds", and "The lead
   hands builds to the owner (`coord build hand`)" becomes `coord test run hand`.
5. `coordinator.md` line 150: the `coordinator-builds.md` row reads "handing the owner a test run".
6. `coordinator-wakes.md` line 50: "the pass that leaves no manual test of a bundle pending" becomes
   the `plan-run-done` wake: a run of the thread's plan ended (once, with the counts). The reason
   `bundle-complete` goes; there is no `run-complete` (D-63).
7. `portfolio.md` line 151: "when a lead hands a bundle (`<Project>: <bundle> is ready to test
   (N tests)`)" becomes "when a lead hands a run (`<Project>: test run I-n (title) is ready to run
   on <build>`)".
8. `coordinator-worktrees.md` lines 79 and 81: a git bundle file in the worktree backups. Not a
   test bundle: leave it.
9. `thread-verbs.md` and `coordinator-board-admin.md`: no bundle mention today. Add the test plan
   rule (a plan keeps at least one case) where the plan verbs are shown, if they are shown.
10. Standing rule for all text: the unit the owner tests is a test run, a plan is what it is run from,
    and a build is the launcher the run carries.

## 15. A story or a task needs a parent (I-102)

Owner rule, 2026-11-04: a story or a task is never created without an epic above it. Apply this
after the Argus build with I-102 is deployed locally.

- `coord item add` for a story or a task now needs `--parent I-n`. A story's parent is an epic. A
  task's parent is an epic or a story. A bare `coord item add "<title>"` is a task, so it is
  refused the same way. The refusal lists the open epics (id and title, at most five).
- `coord item set <I-n> --parent none` is refused for a story or a task. A bug, a debt and a test
  plan keep an optional parent.
- Existing stories and tasks with no parent stay readable and editable in their other fields. The
  kanban card shows the warning "no epic" on each.
- In `SKILL.md` and `thread-verbs.md`, show `--parent` in every `item add` example for a story or a
  task, and add the rule "a story or a task needs an epic parent" next to the type list.

## 16. Worktree hygiene: attached, approve, report, exception (2026-11-06)

Owner rule, 2026-11-06: the link between an agent and its worktree is mandatory. A worktree,
never a main checkout, with nothing attached is removed, or its lead approves the removal, or the
Overlord deletes it or keeps it as an exception. Apply this after the Argus build with the new
hygiene is deployed locally. The server side is done (`docs/worktree-hygiene.md`); the CLI text
and the verbs below are the Overlord's.

### CLI

1. `crates/argus-coord/src/cli/usage.rs` line 37, the keep line, becomes:

   ```
   coord worktree keep <id|path> --note "why" [--until <date>] [--asked-by <thread>] [--clear] [--project p] | done <request id> --note "..."    (the Overlord only: the exception list; a worktree on it is never removed or reported; a lead asks with coord lead portfolio "keep worktree <id> of <p>: <why>")
   ```

   `keep.rs` (`verbs/remote/worktree/keep.rs`) must stop writing the board `worktreeKeeps` (board
   op `handle_keep` in `argus-board/src/ops/worktree.rs`) and POST
   `/api/p/<p>/worktrees/keep {id, note, until, askedBy, clear, by, byProject}` instead (`byProject`
   is the project of the board the calling thread is registered on, `portfolio` for the Overlord;
   the server refuses a keep without it, because every lead's thread is also named `coordinator`; I-130), then print the
   answer's text. `until` is an ISO date or milliseconds. `--days n` goes; a thread's keep is
   refused by the server with a message that names the Overlord.
2. Line 38, the link line, keeps its verbs and changes its note to:

   ```
   (a worktree goes when nothing is attached to it: no running agent, live thread, run, launcher or process; clean ones go after 1 minute, ones with work ask their lead)
   ```

3. `verbs/remote/worktree/link.rs` line 60: the sentence "It goes with {item}: when {item} is
   done or dropped, Argus removes it after a short grace (the backup is written first). Link it to
   another open item to keep it." becomes "A link names its item; the worktree stays while an
   agent, a live thread working on the item, a run or a process is attached."
4. A new verb, `coord worktree approve <id|path>... [--project p] [--reason "..."]`. It takes
   several ids (I-129): with one it POSTs `/api/p/<p>/worktrees/approve {id, reason, by}` as before;
   with several it POSTs `{ids: [...], reason, by}`. The answer of `ids` is
   `{queued: {id, status, text, request} | null, refused: [{id, code, text}]}`: one removal request
   for every id that passed (it counts once against the queue's cap of 20) and one line per id that
   did not (not listed, attached, kept, locked); print `queued.text` and each refused id with its
   text. Through the existing `file_request(call, id, "approve", ...)` for the single form. Its
   usage line:

   ```
   coord worktree approve <id|path>... [--project p] [--reason "..."]    (the owning lead or the Overlord approves removing unattached worktrees that hold work, one or several ids in one request: full backup first, then removed with what is left; its branch stays)
   ```

5. `coord worktree release` (doc comment in `verbs/remote/worktree/remove.rs` line 225, and a
   usage line if one is added): no grace any more. "Removed at once when nothing is attached and
   nothing would be lost; work left needs coord worktree approve."

### Skill text

In `SKILL.md`, `thread-verbs.md`, `coordinator-worktrees.md` and `coordinator-wakes.md`:

- A worktree must stay attached while it is needed: a running agent, a live thread working in it,
  a run, a launcher or a process. An open pull request does not attach it.
- Unattached and clean, it is removed after 1 minute (its merged branch too) and the lead gets
  one inbox note of what went.
- Unattached with work left (changes, commits no ref keeps, a lock, a failed removal): one
  `worktree-approve` wake and an inbox note. Approve with `coord worktree approve <id>`, or ask
  the Overlord for an exception with `coord lead portfolio "keep worktree <id> of <p>: <why>"`.
  A lock still stops the approval: unlock first.
- Unanswered for 30 minutes, the Overlord gets one `worktree-report` wake and a note on his board.
  He deletes with `coord worktree approve <id> --project <p>` or keeps with
  `coord worktree keep <id> --project <p> --note "why" [--until <date>]`.
- Only the Overlord keeps. A thread's keep is refused. Board keeps written before 2026-11-06 no
  longer hold a worktree; they only snooze `worktree-stale`.
- `coordinator-wakes.md`: add `worktree-approve` (lead) and `worktree-report` (Overlord) to the
  wake list; `worktree-review` is no longer sent.
- Section 12 above: replace "When the item closes the worktree is removed after a short grace,
  with a backup written first, so a lead links a worktree it still needs to another open item."
  with "The worktree stays while something is attached to it; once nothing is, it goes as this
  section 16 says."
