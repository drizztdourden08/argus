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
  coord agent start <name> "<task>" --model <haiku|sonnet|opus> --why "<justification>" [--checkin 15..30] [--kind build|test|research|chore] --item I [--item I ...] (I-n, or the letters of its type: B-12) (--worktree <path|auto> | --master) [--blocked-by id ...] [--branch b]   (prints its A-id; an agent works on an item and runs somewhere, so --item and a place are required; --checkin 15..30: minutes between check-in wakes for an opus-level agent, pick by assumed complexity (15 for risky or sprawling work, 30 for a well-bounded task); required for opus level, ignored for haiku/sonnet)
  coord agent note <A|name> "<text>" | set <A|name> [--task t] [--model m --why "..."] [--kind k] [--item I ...] [--add-item I] [--branch b] [--worktree w|auto | --master] [--blocked-by id ...] [--status waiting|working] [--checkin 15..30]
  coord agent done <A|name> ["<result>"] | fail <A|name> "<reason>" | stop <A|name> | archive|unarchive|rm|restore <A|name>...
  coord agent list [--all] [--thread T] [--status s] [--json] | show <A|name>      (a name is your newest running agent with it)
  coord item add "<title>" [--desc "..."] [--ac "..."]... [--status <s>] [--before|--after <I>]
                 [--type <t>] [--parent <I>] [--assignee <thread|owner>] [--stage <s>] [--field k=v]... [--image <file> "<caption>"]... [--force "<reason>"]   (no --type is a task; a story or a task needs --parent <I>, a story's parent is an epic and a task's an epic or a story, and the server refuses it without one; --parent none is refused for them; coord types lists the types, their stages and fields; each field also has a flag, e.g. --severity S2)
  coord item set <I> [--title t] [--desc d] [--status s] [--type t] [--parent <I>|none] [--assignee who|none] [--stage s] [--field k=v]... [--image <file> "<caption>"]... [--force "<reason>"] [--if-rev n]
  coord item status <I> <lane|stage> [--force "<reason>"]    (a lane is todo|doing|review|done|dropped; --force "<reason>" passes a done rule that refuses and is kept as a decision)
  coord item block <I> --on <I|T|H|Q|D-n|"external: who, what"> | --todo "<what>" --who <owner|thread|lead:key>    coord item unblock <I> [--from <id>]    (a blocker is an id of this board or an external: text; another board's work is coord dep ask)
  coord item move <I> [--status s] [--before <I>|--after <I>|--top|--bottom]    archive|unarchive|rm|restore <I> [--reason "..."]
  coord item ac <I> add "<t>" | set <n> "<t>" | done <n> [--undo] | rm <n>[-<m>] | move <n> <pos>
  coord item show <I> | list [--mine] [--status s] [--type t] [--assignee who|me] [--open] [--archived] [--deleted] [--json]
  coord item tree [<I>] [--type t] [--open] [--assignee who|me]    coord item migrate-blockers [--dry-run]    coord types [<type>]
  coord test add <I> "<title>" [--mode ai|manual] [--file spec.json] [--ac N]...    coord test show <T> | list [--item I] [--mine]   (a case is run by AI or by hand; ai is auto)
  coord test step|setup|expect|proof <T> ... (add, set <n>, rm <n>)    coord test set <T> ...
  coord test run list [--plan I] [--json]      (each run: run id, plan, build, stage, pass, fail, to run)
  coord test run hand <run> | unhand <run> | ready <run> | draft <run>      (hand = ready and assigned to the owner; needs a master launcher and a live build; a run is made by coord test plan run)
  coord test run end <run> | reopen <run> | drop <run>      (end: a running run reports, a reported one passes; a passed or dropped run retires the build made for it)
  coord build list|hand|unhand|ready|draft|close|reopen|rm <run>      (the old bundle verbs, kept one more release: each prints the new coord test run verb and does it; build add and set are gone)
  coord build migrate-links [--dry-run]   (once per board, the lead: the master build gets its flag; a build whose bundle one test plan names, or whose tests verify one item, gets that item; a live build with no clear match is marked and waits for coord launcher link)
  coord launcher set <name> --cmd "<command>" --for I-n | --master [--title t] [--cwd d] [--branch b] [--commit c] [--note n] [--kind app|dev|other] [--current|--not-current] [--window|--no-window] [--use-every 12h|3d|2w|1mo]
  coord launcher link <name> I-n | unlink <name>    (a build serves one work item and closes with it; unlink is for the master build only, which needs no item; --master hands the master role over: one master build per board, mandatory, needs no item)
  coord launcher keep <name> [--days n] [--note "why"] | retire <name> [--note "why"] | list [--all] [--json] | show <name> [--json]    (the Builds tab: what the owner can start; master = the main checkout, mandatory, never retired; every other build needs --for I-n and closes with its item; an older standalone launcher needs --use-every and goes stale when unused past it; one made for a run is retired when the run passes or is dropped)
  coord worktree rule add <parent folder, no wildcards> [--kind folder|repo] [--include "<name-glob>"]... [--exclude "<name-glob>"]... [--thread t] [--stale-days n] [--note "..."] | rule rm <id> | rules [--json] | list [--json]
  coord worktree keep <id|path> --note "why" [--until <date>] [--asked-by <thread>] [--clear] [--project p] | done <request id> --note "..."    (the exception list: the Overlord writes it; a thread files a pending keep that holds the worktree for at most 24 h until he confirms or declines it, --clear withdraws it; a worktree on the list is never removed or reported, and a removal request for it is refused until --clear; --until <date> keeps it through that day, local time, then one hour of grace)
  coord worktree link <id|path> I-n | unlink <id|path>    (a worktree goes when nothing is attached to it: no running agent, live thread, run, launcher or process; clean ones go after 1 minute, ones with work ask their lead)
  coord worktree remove <id|path> [--branch] [--discard --owner-entry <E-id>] [--reason "..."] | prune [<id|path>]    (Argus removes it behind its guards: merged and clean needs nothing, discarding work needs the owner's entry naming it; a backup bundle is written first)
  coord worktree approve <id|path>... [--project p] [--reason "..."]    (the owning lead or the Overlord approves removing an unattached worktree that holds work: full backup first, then removed with what is left; its branch stays)
  coord worktree leftovers [--json] | remove --all-leftovers [--older-than 3d] [--reason "..."]    (folders git no longer lists, only files left: listed with size and age; deleted in the background by Argus when nothing touched them for 3 days; no backup is possible; remove <id|path> works on one, without --branch/--discard)
  coord worktree request <id|path>... | --finished [--idle 6h] | --all-leftovers [--older-than 3d] [--branch] [--reason "..."]  |  requests [--json]    (ASKS Argus to remove, for a session whose own safety check refuses remove: deletes nothing itself; Argus removes behind the same guards, one at a time, and one note comes to your inbox; --finished = merged, or stale and clean; --idle D adds clean worktrees with nothing ahead untouched for D, 1h to 30d, 6h by default)
  coord test result <T> <pass|fail|partial|blocked> "<note>" [--proof <img>] [--run <plan I|run I>]    move <T> --item <I2> | archive|unarchive|rm|restore <T>    coord test add <I> "<title>" ... [--plan <plan I>]    (test plans: coord test plan add "<title>" --case T [--case T]... [--parent I] [--build <launcher>] | cases <plan I> --add T [--before T] | --remove T (the last case stays) | run <plan I> [--build <launcher>] | end <plan I> | show <plan I> (lists its runs); a plan always holds cases; record each case of a run with coord test result <T> <status> "<note>" --run <plan I|run I>, skip is a status too, the result is also written to the test)
  coord test migrate-runs [--dry-run]   (once per board, the lead: each plan run and each bundle becomes a test run, the legacy fields stay; a bundle tag with no plan gets a plan holding its tests)
  coord ask <I> "<question>" --context "<c>" [--option "<recommended>" --option "<other>"]... [--decide] [--image [N=]<file> "<caption>"]...    coord decide <I> "<decision>" --context "<c>" [--image <file> "<caption>"]...   (context and, for a decision, 2 to 5 options are required; --image is png/jpg/jpeg/gif/webp, 1 B to 10 MiB, one caption of 1 to 120 chars per image after the text in flag order, N= ties it to option N on ask only)
  coord notify "<text>" [--item I] [--level info|warn|bad] [--context "<c>"]    coord message "<text for the owner>" [--item I] [--context "<c>"]    coord note <I|T> "<text>"   (an informal remark on an item or a test, up to 300 chars, never blocks it; ask and decide also take a T-n)
  coord entry list [--open] [--kind k] [--mine] [--archived] | show <E> | pick <E> | done <E>
  coord entry set <E> ["<text>"] [--option "..."]... [--context "<c>"] | move <E> --item <I> | archive|unarchive|rm|restore <E>
  coord lock claim <resource> [--desc "..."] [--agent A-n] | release <resource> [--agent A-n] | drop <resource> | list    (a sub-agent passes its own --agent: it shares the lead's identity)
  coord action (alias todo) add "<manual step for the human>" [--item <I|T>] [--who owner|<thread>|lead:<key>] [--before|--after <H>] | done <H> [--undo] | show <H> | list [--who <w>]
  coord action set <H> [--text "..."] [--item <I|T>] [--who <w>] | move <H> [--before|--after <H>] | archive|unarchive|rm|restore <H>
  coord log "<what just happened>" [--subject <s>] [--item <I>]   coord log list [--subject <s>] | rm <subject> <seq>
  coord inbox [--ack] | rm <m-id>

Coordinator side (the same verbs work on every thread's records; --for <T> creates or lists for T, --to <T> reassigns):
  coord board init [--name <Name>] | name <Name> | id | prefixes [<Product>...] [--clear] | show [--json] | open | path
  coord board shortcut [--thread <id>] [--root <dir>]   (adds "<Name> board" to .claude/launch.json: the Browser pane's new tab lists it, Play opens the board, nothing is started; idempotent; the Overlord gets two rows, "Argus" and "Overlord")
  coord thread add <name> --subject <s> --brief "<text>" [--worktree <w>]   (coord dispatch is the same)
  coord thread set <name> [--subject <s>] [--brief "<text>"] [--note "..."] [--status <s>] [--before|--after <T>] [--if-rev n]
  coord thread archive|unarchive|pause|resume <name> [--reason "..."]
  coord item add ... --for <T>     coord item move <I> --to <T>     coord item list --for <T>     coord test list|move ... --for <T>
  coord lock free <resource> [--to T] | rm <resource>    coord agent <note|set|done|fail|stop|...> <A|name> --for <T>
  coord inbox send <thread> "<text>" | list --for <T> | rm <m-id>|--all --for <T>    coord answer <entry-id> "<text>" [--decision]   (inbox send: a thread of THIS board)
  coord tick                                     coord tick woke <thread>...      (manual check, never scheduled: wake + attention, JSON)
  coord check [--stale <minutes>] [--json]
  coord check [<check>] [--commit <sha>] [--pr <PR-n>] [--job gate|upgrade|all] [--trigger push|pr|manual|release]     (with a word or any option: starts a check run (brock ci) on the dashboard's run engine; prints: R-5 C-9 check started; without them it is the board checker above)
  coord pr open "<title>" --head <branch> [--base main] [--item <I-n>] [--body "<text>"] [--commit <sha>]    coord pr list [--status open|merged|closed] [--json]    coord pr merge <PR-n>    coord pr close <PR-n>     (local pull requests, the board's PR-n records; merge runs: brock pr merge --local)
  coord sync import <dir>                        (replay saved owner entries, offline)
  coord run define <name> --cmd "<command>" [--cwd d] [--kind build|test|launch|task] [--hand] [--visible] [--timeout s] [--parallel n] [--env K=V]...
  coord run undefine <name> | defs [--json]      coord run start <name> [--agent A-n] [--reason "..."] [--wait] [--visible]
  coord run stop <R-n> | status [<R-n>] [--json] | list [--status s] [--name n] [--limit n] [--json] | logs <R-n> [--follow] [--tail n]

Ping the owner (any registered thread, the Overlord too; an OS notification on his screen and a line in Argus > Notifications):
  coord remote ci-root <project> <dir> | <project> --clear      (the integration checkout coord check and coord pr merge run Brock in, for a project whose main checkout lags; kept in ~/.coord/remote.json)
  coord remote synced [--note "<what went out>"]      (the lead's daily full remote sync is done; recorded per project in ~/.coord/remote.json, Argus wakes the thread after 24 h without one)
  coord remote [status|local|remote]      (the GitHub switch, ~/.coord/remote.json: local = nothing calls gh, the GitHub API or registry and nothing is pushed (the default); remote = the owner's go, set only when he says so)
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
  coord wake due [--json]                        (threads that hold a queued wake and have no listener: board, thread, count, oldest age, local_ session id or -)
  coord wake-fault list [--all] [--json]        (wakes that failed to reach a thread; the pages show a warning on the Registered strip)
  coord wake-fault clear <project>/<thread> --note "<what was wrong and what you did>"     (the Overlord only)
  coord stall list [--project k]                 (what Argus found stalled: an unread wake, an unstarted request, an idle lead, a silent item)
  coord stall ack <project> --note "<what you did>"     (the Overlord only; stops the resends, the stall stays listed until the cause clears)
  coord proc list [--rule r] [--json] | rules      (the process monitor: Argus > Processes; processes.md in the coord skill)
  coord proc rule set <name> [--name-match re] [--cmd-match re] [--path-match re] [--ignore re]... [--max-mem MB] [--max-group-mem MB]
             [--max-age 6h] [--idle 30m] [--orphan|--no-orphan] [--hung|--no-hung] [--killable|--not-killable] [--note "..."] | rule rm <name>
  coord proc kill <pid> [--tree] --reason "..." | kill-rule <rule> [--older-than 6h] [--idle-only]     (rule and kill: the Overlord only)
  coord activity [--limit n] [--kind k] [--level info|warn|error] [--json]     (what the dashboard noticed and did: Argus > Activity)

Hooks (rendered by ai-config, not typed): coord hook session-start|prompt|tool|guard|stop"#;
