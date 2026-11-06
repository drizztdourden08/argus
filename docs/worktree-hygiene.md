<!-- @layer docs @kind doc -->
# Worktree hygiene

The link between an agent and its worktree is mandatory (the owner's rule of 2026-11-06). A
worktree, never a main checkout, that nothing is attached to is removed at once, or its lead
approves its removal, or the Overlord deletes it or keeps it as an exception. No owner click is
needed anywhere on this path. This is an Argus feature: the Node dashboard has none.

The goal: the Worktrees list of a project is about one to one with the agents that are working.

The code is `crates/argus-worktrees/src/hygiene`.

## Attached

A worktree is attached while any of these holds, whatever its age.

1. A running agent owns it. An agent owns it when the folder is `.claude/worktrees/agent-<claudeId>`
   and a board agent record with that `claudeId` exists (board agent records usually have
   `worktree: null`, so the match is on the id), or when the record's `worktree` names it (an
   absolute path, or the folder name). A record that is `working`, `waiting` or `stop-requested`
   is running. An agent that ended after the last scan counts as attached until a fresh scan.
2. A live thread works in it: an item in progress (`doing`) of a live thread names its folder or
   branch. A feature thread's session is seen through its agent record or such an item; the board
   has no folder per thread.
3. A launcher that is not retired, a bundle that is not closed, or a queued or running run (check
   runs among them) is in it.
4. A live process uses it (the same process check the removal makes).

A list Argus cannot read (the runs, the processes) counts as attached. An open pull request does
not attach a worktree: its branch outlives the folder.

The hygiene leaves alone a main checkout, a folder its repository does not list, and a folder
outside its rule.

## The states

The Worktrees tab shows one state on each row. The 3 day "touched" status of the staleness check
(`active`) is gone from the label: staleness only explains a row that waits on its lead or the
Overlord.

| State | Tag | What it means |
|---|---|---|
| `attached` | attached | Something above is attached. Argus leaves it. |
| `removing` | removing, goes 14:05 | Nothing is attached and nothing would be lost. It goes once the debounce ends. |
| `awaiting-lead` | awaiting lead, lead asked | Nothing is attached but it holds work: uncommitted changes, commits no branch or ref keeps, a lock other than Brock's, or a removal that failed. Its lead is asked. |
| `reported` | with the Overlord | The lead did not answer in 30 minutes: the Overlord was told. |
| `exception` | exception | On the Overlord's exception list. Never removed, never reported. |

A leftover folder with no file in it (nothing git lists, 0 files) is `removing` the same way and
goes through the existing leftover removal. The tab lists the last 20 removals under the table.

## The flow

Argus judges every worktree right after every scan and on every live tick (`liveMs`, 20 seconds).

- **Nothing attached, nothing left.** Once nothing has been attached for `detachDebounceMs` (1
  minute), the clean worktrees go as one request of mode `detached` per owning lead. The request
  worker scans afresh and judges each worktree again right before removing it: an agent that runs
  again, a file made dirty or a run that starts stops it. Then it goes through `Service::remove`
  unchanged: links removed as links first, Brock's lock, the branch deleted only when merged.
  Nothing may be lost on this path. The lead gets one inbox note of what went.
- **Nothing attached, something left.** After the same debounce the owning lead is asked ONCE: an
  inbox note with each worktree, what is left, the approve command and the way to ask the Overlord
  for an exception, and a `worktree-approve` wake. The owning lead is the thread of the worktree's
  last agent while it lives, else the thread of the item it belongs to, else the thread of its
  rule, else the board's lead.
- **The approval.** `coord worktree approve <id>` files a request of mode `approved`: a full
  backup (a patch of the changes, the untracked files, a bundle of commits no ref keeps), then the
  removal with what is left. Its branch stays. A lock still stops it: the ask says to unlock first.
  A backup that would be partial (untracked files or a nested repository over the copy limits)
  refuses it too, and the worktree waits on its lead with the reason.
- **30 minutes unanswered.** `reportAfterMs` after the ask, still unattached and unanswered, the
  worktree is reported ONCE to the Overlord (the lead of the `portfolio` board): an inbox note on
  his board with the worktree, what is left, the backup path and the one delete command
  (`coord worktree approve <id> --project <p>`), and a `worktree-report` wake. With no live
  Overlord the report waits for him.
- **Attached again.** An agent, a run or a process that comes back stops all of this. A later
  detach starts over: a new debounce, a new ask.
- **No loop.** What the removal refused is not tried again while the worktree stays as it was
  (same HEAD, same dirty count); it waits on its lead with the reason, and the lead is not asked
  twice.

## The exception list

Only the Overlord keeps a worktree: `coord worktree keep <id> --project <p> --note "why"
[--until <date>] [--asked-by <thread>]`. A lead asks him with
`coord lead portfolio "keep worktree <id> of <p>: <why>"`. A thread or the owner's page that calls
keep is refused and told so. An exception ends at its `until`, when the Overlord clears it
(`--clear`), or when its worktree is gone.

The keeps threads wrote on the board before 2026-11-06 (`worktreeKeeps`) do not hold a worktree
any more. They still snooze the `worktree-stale` signal until their date.

## How the scan is triggered

A scan runs git in every worktree, which costs with dozens of them, so the scan is event driven.

- **The look.** Every live tick (`liveMs`, 20 seconds) Argus reads file times and sizes, with no
  git process and without the turn that scans and removals take. Each project has its own turn
  and its own deletion worker, so a long removal in one project never holds up another, and a
  removal that waits for its turn more than 30 seconds names in its step who holds it and since
  when. A worktree's fingerprint is: the
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
- **No removal on a cached state.** The hygiene only marks, asks and reports. The removal it hands
  to the request worker begins with a forced scan, then the look again (`still_free`, with the
  board, the runs and the processes read afresh), then `Service::remove`, which reads the folder
  with git once more right before it deletes (HEAD and dirty count must still match the row).

## The counter

The board header and each overview card show `worktrees N / running agents M`. N is the listed
worktrees that are not main checkouts (and not being deleted), M is the running agent records of
the board. The tag turns amber (Tessera warning) when N is more than M + 2.

The numbers are in the worktrees meta of the project (`listed`, `runningAgents`, doc
`meta/worktrees`) and, for the overview cards, in the portfolio doc `meta/worktrees` under
`counts.<project>` (`listed`, `running`).

## The verbs

- `coord worktree release <id|path> [--reason "..."]`: a thread no longer needs it. After a fresh
  look at that one worktree (git runs in it on the spot, with no scan of the project, so it never
  waits for a running removal) Argus checks that nothing is attached and that removing it loses
  nothing, and answers at once with a removal request of the thread (`R-n`); the request worker
  looks again, removes it (the branch too when merged) and the result comes to the thread's inbox.
  Anything that holds work is refused: its lead approves that one. Route:
  `POST /api/p/:key/worktrees/release {id, reason?, by}`.
- `coord worktree approve <id|path> [--project p] [--reason "..."]`: the owning lead, a thread its
  rule allows, or the Overlord approves removing a worktree that holds work. Refused while anything
  is attached or while it is an exception. The result comes to the approver's inbox, on the
  Overlord's own board when he approved from outside the project. Route:
  `POST /api/p/:key/worktrees/approve {id, reason?, by}`.
- `coord worktree keep <id|path> --note "why" [--until <date>] [--asked-by <thread>] [--clear]`:
  the Overlord only. Route:
  `POST /api/p/:key/worktrees/keep {id, note, until?, askedBy?, clear?, by}` (`until` is an ISO
  date or milliseconds).

## The config

In `config.json`, under `worktrees`:

| Key | Default | What it does |
|---|---|---|
| `hygiene` | `true` | `false` turns the hygiene off (the old `release: false` still does). Shadow mode always has it off. |
| `detachDebounceMs` | `60000` | How long nothing must be attached before Argus removes a clean worktree or asks the lead. |
| `reportAfterMs` | `1800000` | How long an ask stays unanswered before the Overlord is told. |
| `scanMs` | `1800000` | The safety scan: how often every worktree is read with git whatever the look says. |

## The state kept

Each worktree's mark (state, owning lead, since, asked, reported, request), what the removal
refused, the exception list and the last 20 removals live in the key-value table under
`worktrees:hygiene:<project>`. A restart keeps every time. The states written before 2026-11-06
(`in-use`, `released`, `closing`, `review`, `kept`) are read as their new names.
