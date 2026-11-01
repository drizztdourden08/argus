<!-- @layer docs @kind doc -->
# Worktree hygiene

A worktree nobody works in goes by itself. Argus flags it to the thread that owns it, waits a
grace, and removes it through the same removal and guards a thread's own `coord worktree remove`
goes through. No owner click is needed anywhere on this path. This is an Argus feature: the Node
dashboard has none.

The goal: the Worktrees list of a project is about one to one with the agents that are working.

The code is `crates/argus-worktrees/src/hygiene`.

## The rule

A worktree is released when every condition holds, whatever its age.

1. No running agent owns it. An agent owns it when the folder is `.claude/worktrees/agent-<claudeId>`
   and a board agent record with that `claudeId` exists (board agent records usually have
   `worktree: null`, so the match is on the id), or when the record's `worktree` names it (an
   absolute path, or the folder name). A record that is `working`, `waiting` or `stop-requested`
   is running and holds it. No record at all, or only `done`, `stopped` or `failed` records, is no
   running owner. An agent that ended after the last scan waits for a fresh scan first (shown in
   review for a moment).
2. Nothing holds it. No unexpired keep (`coord worktree keep`). No open pull request of its
   branch. No launcher that is not retired and no bundle that is not closed whose folder is in it
   or whose branch is its branch. No queued or running run, check runs among them, whose folder is
   in it. No live process uses it (the same process check the removal makes). An open board item
   that names its folder or branch holds it only when the item is in progress (`doing`) and
   assigned to a thread that is live; a todo, review, done or orphaned item does not.
3. Safe. Not a main checkout, a linked worktree its repository lists, inside its rule, not
   locked (Brock's own lock, `cloned node_modules ...`, aside: the removal lifts it), clean, and
   its HEAD kept by a branch or a ref, or merged (0 ahead counts).

A leftover folder with no file in it (nothing git lists, 0 files) is released the same way and goes
through the existing leftover removal, with no age limit because there is nothing to lose.

## The states

The Worktrees tab shows one state on each row. The 3 day "touched" status of the staleness check
(`active`) is gone from the label: staleness only explains a row in review.

| State | What it means |
|---|---|
| in use | A running agent owns it, or something holds it (a process, a run, a launcher, a pull request, work in progress). Argus leaves it. |
| released | Every condition holds. The row says when it goes once the thread was told. |
| review | Argus will not remove it by itself and the thread decides: uncommitted work, commits no branch or ref keeps, a lock, a removal the guards refused. The row says what. |
| kept | The thread kept it (`coord worktree keep <id> --note "..."`). |
| leftover | No `.git`, no repository lists it: only files are left (shown by the status; an empty one is released). |
| removed | The hygiene removed it. The tab lists the last 20 under the table. |

Never automatic: uncommitted work, commits no branch or ref keeps, a lock other than Brock's, a
folder in use, a main checkout (the hygiene does not judge it; the row carries the `main` tag),
anything outside a rule. Those stay in review or in use, and the thread still hears of them through
the `worktree-stale` wake when they grow stale.

## The flow

- Argus judges every worktree right after every scan, so once right after the first scan at startup
  (the first sweep flags every released worktree of every project and removes them after the
  grace), after the scan an agent that ended asks for (a board agent change from running to done,
  stopped or failed), and on every live tick (`liveMs`, 20 seconds).
- A newly released worktree is told to its thread with ONE `worktree-review` wake per thread,
  once nothing new was released for that thread for `releaseDebounceMs`. A burst of agents
  ending makes one wake. The thread is the agent's own thread when it may remove the worktree,
  else the rule's thread, else the project's lead.
- `releaseGraceMs` after the wake, the released worktrees of a thread go as one removal request
  of mode `released`, by that thread. The request worker scans afresh and judges each worktree
  again right before removing it: an agent that runs again, a file made dirty, a keep, a new link
  or a process during the grace stops it. Then it goes through `Service::remove` unchanged: links
  removed as links first, the backups, Brock's lock, the branch deleted only when merged.
- The thread gets one inbox note for the batch: what went, and what was refused and why.
- What the removal itself refused is not released again while the worktree stays as it was (same
  HEAD, same dirty count), so a refusal never loops.

## How the scan is triggered

A scan runs git in every worktree, which costs with dozens of them, so the scan is event driven.

- **The look.** Every live tick (`liveMs`, 20 seconds) Argus reads file times and sizes, with no
  git process and without the turn that scans and removals take. A worktree's fingerprint is: the
  folder is there, its `.git`, its own `HEAD` and `index` (for a linked worktree the files under
  `<main>/.git/worktrees/<name>`, found through its `.git` file), the loose ref of its branch, the
  loose ref of the branch the main checkout has, and `packed-refs`. The shape of the rules is
  also looked at: the candidate folders of a folder rule, and the registration folder
  (`.git/worktrees`) of a repo rule, so a folder or a worktree that comes or goes shows.
- **The scan that follows.** It runs only when the look saw a change. It reads with git only the
  worktrees whose fingerprint changed (or are new); every other worktree keeps the facts of the
  last scan, except the registration facts (main, locked, prunable), which come from one
  `git worktree list` for each repository.
- **The safety scan.** An edited tracked file, a fetch or a push changes none of those files, so
  every `scanMs` (30 minutes by default, `SAFETY_SCAN_MS`) one scan reads everything with git
  again. The dirty count of a row can be that old; nothing is ever removed on it (below).
- **A forced scan** reads everything and trusts no fingerprint: the first scan after the start, a
  changed rule (after the debounce), the rescan after a removal, a prune or an agent that ended,
  the unmeasured leftovers, and the request worker before it starts a request.
- **No removal on a cached state.** The hygiene only flags and tells. The removal it hands to the
  request worker begins with a forced scan, then the look again (`still_released`, `blocked_why`
  with fresh process data), then `Service::remove`, which reads the folder with git once more
  right before it deletes (HEAD and dirty count must still match the row).

## The counter

The board header and each overview card show `worktrees N / running agents M`. N is the listed
worktrees that are not main checkouts (and not being deleted), M is the running agent records of
the board. The tag turns amber (Tessera warning) when N is more than M + 2.

The numbers are in the worktrees meta of the project (`listed`, `runningAgents`, doc
`meta/worktrees`) and, for the overview cards, in the portfolio doc `meta/worktrees` under
`counts.<project>` (`listed`, `running`).

## The thread's verbs

- `coord worktree keep <id|path> --note "why" [--days n]` keeps it (7 days by default).
- `coord worktree keep <id|path> --clear` ends the keep.
- `coord worktree release <id|path> [--reason "..."]` says the thread no longer needs it. After a
  fresh look at that one worktree (git runs in it on the spot, with no scan of the project, so
  it never waits for a running removal) Argus checks that nothing holds it and that removing it is
  safe (not the keep) and answers at once with a removal request of the thread (`R-n`); the project's request worker
  looks again, removes it through the same removal (the branch too when merged) and the result
  comes to the thread's inbox. Anything that is not safe is refused with the reason. Route: `POST /api/p/:key/worktrees/release {id, reason?, by}`.

## The config

In `config.json`, under `worktrees`:

| Key | Default | What it does |
|---|---|---|
| `release` | `true` | `false` turns the hygiene off. Shadow mode always has it off. |
| `releaseGraceMs` | `3600000` | How long a released worktree stays after its thread was told. |
| `releaseDebounceMs` | `30000` | How long nothing new must be released before the thread is told. |
| `scanMs` | `1800000` | The safety scan: how often every worktree is read with git whatever the look says. |

## The state kept

What each worktree is, when its thread was told, its request, what the removal refused, and the
last 20 removals live in the key-value table under `worktrees:hygiene:<project>`. A restart keeps
the time a worktree goes.
