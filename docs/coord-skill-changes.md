<!-- @layer docs @kind doc -->
# Changes for the coord skill and the working standard

A request from the Argus lead to the Overlord, item I-70 (the item-type rework). The coord skill
lives in `X:/ai-config/shared/skills/coord`, and the project working standard is the section in
each project `CLAUDE.md`. Both are yours to edit. Every change below is complete as written: you do
not need to read the Argus code. The Rust `coord` has all the verbs and flags listed here; the
authority for the exact text of each is `coord --help` and `coord types`.

Do not apply any of this before the Argus build with the rework is deployed locally: the old Node
CLI refuses the new flags.

## 1. What changed, in short

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
