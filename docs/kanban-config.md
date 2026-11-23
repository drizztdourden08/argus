# Kanban config

Each board may carry its own Kanban columns and rules. The config lives in `board.json` as the
top-level `kanban` object. The write-time rules read it when an item is written (see Write-time
rules). The sweep runs the alert rules every tick (see The sweep).

## Shape

```json
{
  "columns": [
    {
      "id": "doing",
      "name": "Doing",
      "accent": "blue",
      "match": { "lanes": ["doing"], "stages": [] },
      "limit": { "max": 3, "mode": "warn" },
      "rules": ["stale-doing"]
    }
  ],
  "rules": [
    {
      "id": "stale-doing",
      "kind": "dwell",
      "params": { "ms": 14400000 },
      "alert": { "to": "item-thread", "wake": true, "repeatMs": 43200000 }
    }
  ]
}
```

- A column has an `id` (unique), a `name`, an optional `accent`, a `match`, an optional `limit` and
  the ids of the `rules` that apply to it.
- `match.lanes` are item statuses (`todo`, `doing`, `review`, `done`, `dropped`). `match.stages`
  narrows to named stages; empty means every stage of the lanes.
- `limit.max` is 1 or more; `limit.mode` is `warn`, `refuse` or `alert`.
- A rule has an `id` (unique), a `kind`, `params`, and either a `mode` (`warn` or `refuse`, for the
  write-time kinds) or an `alert` (for the sweep kinds). `alert.to` is `item-thread`, `assignee`,
  `lead`, `owner` or a thread id; `alert.wake` says whether it wakes or only lands in the inbox;
  `alert.repeatMs` is the wait before the same break is told again.

### Rule kinds

| Kind | When | Params |
|---|---|---|
| `require-parent` | write | `types`: the parent types allowed |
| `require-assignee` | write | `kinds` (optional): the assignee kinds allowed |
| `limit` | write | none |
| `require-field` | write | `field`: the field that must be set |
| `dwell` | sweep | `ms`: the time in the lane before the break |
| `assignee-gone` | sweep | none |
| `over-limit` | sweep | none |
| `unassigned` | sweep | `ms`: the time without an assignee |

## Defaults

A board with no `kanban` runs the default, which is today's page:

| Column | Lanes | Quiet rule |
|---|---|---|
| To do | `todo` | `stale-todo`: dwell 24 h |
| Doing | `doing` | `stale-doing`: dwell 4 h |
| Review | `review` | `stale-review`: dwell 6 h |
| Done | `done`, `dropped` | none |

Each stale rule alerts `item-thread`, with a wake, every 12 h, as the stale sweep does today. A
stored config that no longer passes the checks falls back to the default.

## The sweep

Every tick (5 minutes) the wake service sweeps each board with the config in use: the board's own, or
the default when it has none or a damaged one. The default is built from the `items` timing of
`config.json` (`staleTodoMs`, `staleDoingMs`, `staleReviewMs`, `repeatMs`), so a board with no
config gets the stale reminders it always had, with the same wording and the same single wake per
thread, worst offenders first.

For each live item, each column that holds it (its lane, and its stage when the column names stages)
runs the sweep rules the column lists:

| Kind | A break is |
|---|---|
| `dwell` | the item sat in the column longer than `params.ms`. The clock starts at `laneAt` (then `startedAt`, then `updatedAt`) and restarts when anything touches the item (tests, to-dos, sub-agents, entries). An item that waits on someone else is skipped, as the stale reminders skip it |
| `assignee-gone` | the assignee is an agent that no longer runs, or a thread that is missing or ended. `owner` and no assignee are never a break |
| `unassigned` | the item had no assignee for longer than `params.ms` since it entered the column |
| `over-limit` | the column holds more live items than `limit.max`. A column whose `limit.mode` is `alert` fires too, to the lead, with a wake, every 12 h, when it has no `over-limit` rule |

Who hears of a break is `alert.to`: `item-thread` (the item's thread while it is live, else the
lead), `assignee` (the assignee thread, an agent's thread, or the owner; the item thread when none),
`lead`, `owner`, or a thread id (that thread while live, else the lead). `over-limit` is about a
column, not one item, so `item-thread` and `assignee` mean the lead there. A paused thread hears
nothing until it resumes.

`alert.wake` picks the reason of the row in the wakes table. With `wake` on, a `dwell` keeps
`kanban-stale` and the other kinds use `kanban-alert` (a wake). With `wake` off the row is a
`kanban-notice`, which only lands in the inbox and the board notices. One line is sent per project,
recipient and kind, listing the worst items first.

`alert.repeatMs` is the wait before the same break is told again to the same recipient. For a thread
the wakes table remembers it, so a restart does not repeat a line. An alert to the `owner` goes
through the owner notice path (`OwnerNotice`, topic `kanban`: a toast and a line in the
notifications); it has no wakes row, so only the running process remembers its pace and a restart
may tell it once more.

## Who may change it

The owner and the board's lead, through the `kanban-config` op (`null` clears it). Every change
writes one audit line. A thread is refused.

```text
coord board kanban show
coord board kanban set --file kanban.json
```

`show` prints `{custom, kanban}`: whether the board has its own config, and the one in use.

## Export and `laneAt`

The page gets the config in use as the meta doc `meta/kanban`, the default when the board sets none.

Every item has a `laneAt` stamp (an ISO time): when it entered its current lane. It is set on
create and on every change of `status` (move, edit, a dependency drop or reopen). The `dwell`
rule reads it, falling back to `startedAt`, then `updatedAt`, for older items.

## Write-time rules

When a `create`, `edit` or `move` op writes an item (a bulk runs its ops one by one), the column
the item lands in judges it: the first column whose `match` takes its status and stage. The rules
of that column run on the board as the write leaves it.

- `require-parent`: the item has a parent whose type is in `params.types` (an item with no type
  reads as `task`).
- `require-assignee`: the item has an `assignee`; with `params.kinds`, of one of those kinds:
  `thread`, `owner`, `agent` (`agent:A-n`) or `lead` (`lead:key`).
- `require-field`: `params.field` is set on the item or among its type `fields`.
- `limit`: the column's `limit.max` is not passed by the live items in it (not in the trash, not
  archived), counting the item just written. The column's own `limit` acts the same with its mode
  when the column names no `limit` rule; mode `alert` does nothing at write time (the sweep
  handles it).

A rule with `mode` `refuse` refuses the op: nothing is written and the outcome is
`rule-refused: I-5 cannot enter Doing: <rule id> needs a parent of type epic.` A rule with mode
`warn` (also when no mode is set) lets the op apply, and the outcome ends with `Warning: ...` and
lists the lines in `warnings`, as the stage warnings of typed items do. Only items the op created
or changed in status, stage, parent, assignee, thread, type or fields are judged. A board with no
config runs the default, which has no write-time rule, so nothing changes for it.

## Error keys

A refusal starts with its key, then a colon and a sentence.

| Key | Meaning |
|---|---|
| `bad-shape` | not the shape above (a wrong type, a missing or unknown field) |
| `no-columns` | no column |
| `bad-column` | a column with no id or name |
| `duplicate-column` | two columns share an id |
| `unknown-lane` | a column names a lane that does not exist, or none |
| `unknown-stage` | a column names a stage no item type has |
| `bad-limit` | `max` below 1, or a mode other than `warn`, `refuse`, `alert` |
| `unknown-rule` | a column names a rule the config does not define |
| `duplicate-rule` | two rules share an id |
| `unknown-rule-kind` | a rule kind not in the table |
| `bad-rule-mode` | a rule mode other than `warn` or `refuse` |
| `bad-params` | the params of the kind are missing or of the wrong type |
| `bad-alert` | an alert with no recipient or a `repeatMs` below 1 |
| `rule-refused` | a write-time rule with mode `refuse` refused an item op |
