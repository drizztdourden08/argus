# Kanban config

Each board may carry its own Kanban columns and rules. The config lives in `board.json` as the
top-level `kanban` object. This piece only stores and checks it. The write-time rules and the
sweep that read it come later (I-206 pieces P6 and P7).

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
