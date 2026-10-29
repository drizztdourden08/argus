//! The usage text of `coord`, verbatim from `coord.mjs` of the Node CLI (printed by `help`, with no verb,
//! and after an unknown verb).

/// The usage text, without a trailing newline.
pub const USAGE: &str = r#"Usage: coord <verb> ... [--project <key>] [--as <thread>]

Thread side (every thread; read everything, change only what is yours):
  coord thread register <name> [--subject <s>] [--session "<title>"] [--link <claude://...>] [--note "..."]
  coord thread note "<what I am doing now>"      coord thread status <active|blocked|waiting|idle|done>
  coord thread set [--subject <s>] [--note "..."] [--status <s>]   coord thread flag <needs-human|blocked> [--clear]
  coord thread list [--all] | show [name] | done | pause [name] | resume [name]
  coord thread archive [name] | unarchive [name]  (session gone: off the list, work stays visible; --as <name> works too)
  coord agent start <name> "<task>" --model <haiku|sonnet|opus> --why "<justification>" [--checkin 15..30] [--kind build|test|research|chore] [--item I] [--branch b] [--worktree w]   (prints its A-id; --checkin 15..30: minutes between check-in wakes for an opus-level agent, pick by assumed complexity (15 for risky or sprawling work, 30 for a well-bounded task); required for opus level, ignored for haiku/sonnet)
  coord agent note <A|name> "<text>" | set <A|name> [--task t] [--model m --why "..."] [--kind k] [--item I] [--branch b] [--worktree w] [--status waiting|working] [--checkin 15..30]
  coord agent done <A|name> ["<result>"] | fail <A|name> "<reason>" | stop <A|name> | archive|unarchive|rm|restore <A|name>...
  coord agent list [--all] [--thread T] [--status s] [--json] | show <A|name>      (a name is your newest running agent with it)
  coord item add "<title>" [--desc "..."] [--ac "..."]... [--subject <s>] [--status <s>] [--before|--after <I>]
  coord item set <I> [--title t] [--desc d] [--subject s] [--status s] [--if-rev n]    coord item status <I> <s>
  coord item move <I> [--status s] [--before <I>|--after <I>|--top|--bottom]    archive|unarchive|rm|restore <I> [--reason "..."]
  coord item ac <I> add "<t>" | set <n> "<t>" | done <n> [--undo] | rm <n>[-<m>] | move <n> <pos>
  coord item show <I> | list [--mine] [--status s] [--subject s] [--archived] [--deleted] [--json]
  coord test add <I> "<title>" [--mode auto|manual] [--build <tag>] [--file spec.json] [--ac N]...    coord test show <T> | list [--item I] [--mine] [--build <tag>]
  coord test step|setup|expect|proof <T> ... (add, set <n>, rm <n>)    coord test set <T> ... [--build <tag> | --clear-build]
  coord build list [--json]      (each bundle: build tag, its tests, pass, fail, to run, launch command; which one is handed)
  coord build add <tag> [--title t] [--note n] [--launcher <name> | --cmd "<launch command>" [--cwd d]] --thread <you>    build set <tag> [same flags; "" clears]    build rm <tag> [--force]
  coord build close <tag> [--note "why"] [--force] | reopen <tag>    (a finished, landed bundle leaves the owner's Tests tab until "Show closed"; --cmd sets the launcher named after the tag and links it)
  coord launcher set <name> --cmd "<command>" [--title t] [--cwd d] [--branch b] [--commit c] [--note n] [--kind app|dev|other] [--current|--not-current] [--window|--no-window] [--use-every 12h|3d|2w|1mo]
  coord launcher keep <name> [--days n] [--note "why"] | retire <name> [--note "why"] | list [--all] [--json] | show <name> [--json]    (the Builds tab: what the owner can start; master = the main checkout, mandatory, never retired; a standalone launcher needs --use-every and goes stale when unused past it; one made for a bundle is retired when the bundle closes)
  coord worktree rule add <parent folder, no wildcards> [--kind folder|repo] [--include "<name-glob>"]... [--exclude "<name-glob>"]... [--thread t] [--stale-days n] [--note "..."] | rule rm <id> | rules [--json] | list [--json]
  coord worktree keep <id|path> --note "why" [--days n | --clear] | done <request id> --note "..."    (the Worktrees tab: tell Argus where the worktrees live; it looks, and wakes you when one is stale)
  coord worktree remove <id|path> [--branch] [--discard --owner-entry <E-id>] [--reason "..."] | prune [<id|path>]    (Argus removes it behind its guards: merged and clean needs nothing, discarding work needs the owner's entry naming it; a backup bundle is written first)
  coord worktree leftovers [--json] | remove --all-leftovers [--older-than 3d] [--reason "..."]    (folders git no longer lists, only files left: listed with size and age; deleted in the background by Argus when nothing touched them for 3 days; no backup is possible; remove <id|path> works on one, without --branch/--discard)
  coord worktree request <id|path>... | --finished [--idle 6h] | --all-leftovers [--older-than 3d] [--branch] [--reason "..."]  |  requests [--json]    (ASKS Argus to remove, for a session whose own safety check refuses remove: deletes nothing itself; Argus removes behind the same guards, one at a time, and one note comes to your inbox; --finished = merged, or stale and clean; --idle D adds clean worktrees with nothing ahead untouched for D, 1h to 30d, 6h by default)
  coord test result <T> <pass|fail|partial|blocked> "<note>" [--proof <img>]    move <T> --item <I2> | archive|unarchive|rm|restore <T>
  coord ask <I> "<question>" --context "<c>" [--option "<recommended>" --option "<other>"]... [--decide]    coord decide <I> "<decision>" --context "<c>"   (context and, for a decision, 2 to 5 options are required)  [--image [N=]<png> "<caption>"]... on either: pictures to open and zoom, one quoted caption each after the text, N= ties it to option N from 1
  coord notify "<text>" [--item I] [--level info|warn|bad] [--context "<c>"]    coord message "<text for the owner>" [--item I] [--context "<c>"]
  coord entry list [--open] [--kind k] [--mine] [--archived] | show <E> | pick <E> | done <E>
  coord entry set <E> ["<text>"] [--option "..."]... [--context "<c>"] | move <E> --item <I> | archive|unarchive|rm|restore <E>
  coord lock claim <resource> [--desc "..."] | release <resource> | drop <resource> | list
  coord action (alias todo) add "<manual step for the human>" [--item <I>] [--before|--after <H>] | done <H> [--undo] | show <H> | list
  coord action set <H> [--text "..."] [--item <I>] | move <H> [--before|--after <H>] | archive|unarchive|rm|restore <H>
  coord log "<what just happened>" [--subject <s>] [--item <I>]   coord log list [--subject <s>] | rm <subject> <seq>
  coord inbox [--ack] | rm <m-id>

Coordinator side (the same verbs work on every thread's records; --for <T> creates or lists for T, --to <T> reassigns):
  coord board init [--name <Name>] | name <Name> | prefixes [<Product>...] [--clear] | show [--json] | open | path
  coord board shortcut [--thread <id>] [--root <dir>]   (adds "<Name> board" to .claude/launch.json: the Browser pane's new tab lists it, Play opens the board, nothing is started; idempotent; the Overlord gets two rows, "Argus" and "Overlord")
  coord thread add <name> --subject <s> --brief "<text>" [--worktree <w>]   (coord dispatch is the same)
  coord thread set <name> [--subject <s>] [--brief "<text>"] [--note "..."] [--status <s>] [--before|--after <T>] [--if-rev n]
  coord thread archive|unarchive|pause|resume <name> [--reason "..."]
  coord item add ... --for <T>     coord item move <I> --to <T>     coord item list --for <T>     coord test list|move ... --for <T>
  coord lock free <resource> [--to T] | rm <resource>    coord agent <note|set|done|fail|stop|...> <A|name> --for <T>
  coord inbox send <thread> "<text>" | list --for <T> | rm <m-id>|--all --for <T>    coord answer <entry-id> "<text>" [--decision]   (inbox send: a thread of THIS board)
  coord build hand <tag> | unhand                (the build the owner was handed; the page marks its tests; a draft bundle cannot be handed); build add|set|rm need no --thread
  coord build ready <tag> | draft <tag>          (a new bundle is a draft: built and testable = ready, which shows Play and allows hand)
  coord tick                                     coord tick woke <thread>...      (manual check, never scheduled: wake + attention, JSON)
  coord check [--stale <minutes>] [--json]
  coord check [<check>] [--commit <sha>] [--pr <PR-n>] [--job gate|upgrade|all] [--trigger push|pr|manual|release]     (a check run: starts brock ci on the run engine in the project root and prints R-n and C-n; see coord run logs R-n)
  coord pr open "<title>" --head <branch> [--base main] [--item <I>] [--body "..."] [--commit <sha>]     coord pr list [--status s] [--json] | merge <PR-n> | close <PR-n>     (local pull requests, the records PR-n of the local remote; merge runs brock pr merge --local and records the merge commit it prints)
  coord sync import <dir>                        (replay saved owner entries, offline)
  coord run define <name> --cmd "<command>" [--cwd d] [--kind build|test|launch|task] [--hand] [--visible] [--timeout s] [--parallel n] [--env K=V]...
  coord run undefine <name> | defs [--json]      coord run start <name> [--agent A-n] [--reason "..."] [--wait] [--visible]
  coord run stop <R-n> | status [<R-n>] [--json] | list [--status s] [--name n] [--limit n] [--json] | logs <R-n> [--follow] [--tail n]

Ping the owner (any registered thread, the Overlord too; an OS notification on his screen and a line in Argus > Notifications):
  coord ping "<text, max 240>" [--title "<t, max 60>"] [--level info|ping|urgent] [--project k]
              (ping only when the owner is needed: a build handed and ready to test, a blocking decision only he can answer, something
              broken that only he can fix; never for progress or a "done" with nothing to look at. info = recorded only; urgent = stays until
              dismissed, only when he must act now. At most 3 toasts a minute per thread; more are recorded without a toast.)

Coordinator only (hard delete; rm on a record is a soft delete into the trash):
  coord purge thread <T>... | --keep <T>... | --status done,archived,placeholder,dispatched | --idle <days> | --trash [--older <days>]
              [--reason "..."] [--confirm <project-key>]   (no --confirm: dry run, counts and the exact confirm command)
  coord thread rm <T>...   (= purge thread)
  coord restore --from <backup.json> [--thread T]... --confirm <project-key>
  coord trash [--for T] [--json]

Lead to lead (another project's coordinator; it needs no thread on that board, max 240 chars; wakes it when it runs wake-stream):
  coord lead <project-key> "<text>" [--kind note|decision]      e.g. coord lead brock "rule X fires on Y, I think it is wrong"

Argus (the overview of every project; its verb is "coord portfolio", used by the Overlord):
  coord portfolio init | add <key> --name "<Name>" [--root <dir>] | remove <key> | show [--json]
  coord portfolio send <key|all> "<text>" [--decision]   coord portfolio tick

Dashboard (the local server for every board page, MCP and runs):
  coord dashboard start | stop | status | open [<key>] | logs | resync [--project k] [--hard] | mcp | install-mcp | autostart
  coord dashboard wake-stream --project <key> --thread <you|*>     (run it under the Monitor tool: one line per wake)
  coord wake-fault list [--all] [--json]         (wakes that failed to reach a thread; the pages show a warning on the Registered strip)
  coord wake-fault clear <project>/<thread> --note "<what was wrong and what you did>"     (the Overlord only)
  coord proc list [--rule r] [--json] | rules      (the process monitor: Argus > Processes; processes.md in the coord skill)
  coord proc rule set <name> [--name-match re] [--cmd-match re] [--path-match re] [--ignore re]... [--max-mem MB] [--max-group-mem MB]
             [--max-age 6h] [--idle 30m] [--orphan|--no-orphan] [--hung|--no-hung] [--killable|--not-killable] [--note "..."] | rule rm <name>
  coord proc kill <pid> [--tree] --reason "..." | kill-rule <rule> [--older-than 6h] [--idle-only]     (rule and kill: the Overlord only)
  coord activity [--limit n] [--kind k] [--level info|warn|error] [--json]     (what the dashboard noticed and did: Argus > Activity)

Hooks (rendered by ai-config, not typed): coord hook session-start|prompt|tool|guard|stop"#;
