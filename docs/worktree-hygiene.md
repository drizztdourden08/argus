<!-- @layer docs @kind doc -->
# Worktree hygiene

The worktree of a sub-agent that ended goes by itself. Argus flags it to the thread that owns it,
waits a grace, and removes it through the same removal and guards a thread's own `coord worktree
remove` goes through. No owner click is needed anywhere on this path. This is an Argus feature:
the Node dashboard has none.

The code is `crates/argus-worktrees/src/hygiene`.

## The rule

A worktree is released when every condition holds.

1. Not needed. A sub-agent record owns it: the record's `worktree` names it (an absolute path,
   or the folder name), or it is the agent's own folder `.claude/worktrees/agent-<claudeId>`.
   Every record that owns it is `done` or `stopped`, and the last scan read the folder after the
   agent ended. An agent still `working`, `waiting` or `stop-requested` holds it, and the hygiene
   says nothing. A `failed` agent leaves it in review.
2. Nothing is linked. No unexpired keep (`coord worktree keep`). No open item (todo, doing,
   review) whose title or description names its folder or branch. No open pull request of its
   branch. No launcher that is not retired and no bundle that is not closed whose folder is in it
   or whose branch is its branch. No queued or running run, check runs among them, whose folder is
   in it. No live process uses it (the same process check the removal makes).
3. Safe. Not a main checkout, a linked worktree its repository lists, inside its rule, not
   locked (Brock's own lock, `cloned node_modules ...`, aside: the removal lifts it), clean, and
   its HEAD kept by a branch or a ref, or merged.

## The states

The Worktrees tab shows the state on each row of an ended agent's worktree.

| State | What it means |
|---|---|
| released | Every condition holds. The row says when it goes once the thread was told. |
| review | Its agent ended, but something keeps Argus from removing it. The row says what. |
| kept | The thread kept it (`coord worktree keep <id> --note "..."`). |
| removed | The hygiene removed it. The tab lists the last 20 under the table. |

Never automatic: uncommitted work, commits no branch or ref keeps, a lock other than Brock's, a
folder in use, a main checkout, anything outside a rule. Those stay in review, and the thread still
hears of them through the `worktree-stale` wake when they grow stale.

## The flow

- After every scan and on every live tick (`liveMs`, 20 seconds) Argus judges every worktree.
- A newly released worktree is told to its thread with ONE `worktree-review` wake per thread,
  once nothing new was released for that thread for `releaseDebounceMs`. A burst of agents
  ending makes one wake. The thread is the agent's own thread when it may remove the worktree,
  else the rule's thread, else the project's lead.
- `releaseGraceMs` after the wake, the released worktrees of a thread go as one removal request
  of mode `released`, by that thread. The request worker scans afresh and judges each worktree
  again right before removing it: a file made dirty, a keep, a new link or a process during the
  grace stops it. Then it goes through `Service::remove` unchanged: links removed as links
  first, the backups and age rules, Brock's lock, the branch deleted only when merged.
- The thread gets one inbox note for the batch: what went, and what was refused and why.
- What the removal itself refused is not released again while the worktree stays as it was (same
  HEAD, same dirty count), so a refusal never loops.

## The thread's verbs

- `coord worktree keep <id|path> --note "why" [--days n]` keeps it (7 days by default).
- `coord worktree keep <id|path> --clear` ends the keep.
- `coord worktree release <id|path> [--reason "..."]` says the thread no longer needs it. After a
  fresh scan Argus checks conditions 2 and 3 (not the keep) and removes it at once through the
  same removal, the branch too when merged. Anything that is not safe is refused with the reason.
  Route: `POST /api/p/:key/worktrees/release {id, reason?, by}`.

## The config

In `config.json`, under `worktrees`:

| Key | Default | What it does |
|---|---|---|
| `release` | `true` | `false` turns the hygiene off. Shadow mode always has it off. |
| `releaseGraceMs` | `3600000` | How long a released worktree stays after its thread was told. |
| `releaseDebounceMs` | `30000` | How long nothing new must be released before the thread is told. |

## The state kept

What each worktree is, when its thread was told, its request, what the removal refused, and the
last 20 removals live in the key-value table under `worktrees:hygiene:<project>`. A restart keeps
the time a worktree goes.
