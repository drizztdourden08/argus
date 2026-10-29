<!-- @layer docs @kind doc -->
# Local names

Argus answers on port 80 of `127.0.0.1` for `<key>.localhost`, with no port and nothing to
configure: `http://argus.localhost/` is the overview, `http://rotp.localhost/` the board of
`rotp` (the lead), `http://argus-app.localhost/` the board of the `argus` project and
`http://<thread>.<key>.localhost/` a thread. This is the way to open Argus. The older `.argus`
names below are opt-in and need browser setup.

## The localhost door

- **Why.** Browsers resolve every `*.localhost` name to loopback without DNS, so nothing on the
  machine has to be set up. Claude's built-in browser pane loads the HTML of `http://<key>.argus/`
  but blocks its scripts and styles (`net::ERR_BLOCKED_BY_CLIENT`), even when the site is in its
  allow list, while `<key>.localhost` hosts load fully. Sessions therefore open
  `http://<key>.localhost/?thread=<id>`. Firefox also skips the Windows DNS rules that `.argus`
  needs.
- **What it is.** A second listener on `127.0.0.1:80` that serves the same app and routes by
  `Host` exactly as the main listener does on `:47615`: `http://<key>.localhost/` redirects to
  `/p/<key>/?thread=coordinator`, `http://<thread>.<key>.localhost/` to
  `/p/<key>/?thread=<thread>`, `argus.localhost` is the overview. The Host and Origin checks
  accept `<x>.localhost` with no port on this door and refuse every other host with 421. The
  existing `<key>.localhost:47615` hosts keep working.
- **The update feed host.** `http://releases.localhost/<project>/<channel>/<path>` is the
  read-only release store, the same as `/releases/<project>/<channel>/<path>` on the main
  listener (see `docs/local-remote.md`). A project called `releases` cannot use that host for its
  board.
- **On by default**, whether or not `localNames` is on. `"localhostDoor": false` in `config.json`
  turns it off. It needs a restart of Argus to change.
- **If port 80 cannot be bound** (taken, or owned by `http.sys`), the server logs one line
  (`localhost door: cannot bind 127.0.0.1:80, it stays off`) and carries on; nothing else changes.
- **Where it is reported.** `/api/health` has `"localhostDoor": 80` (or `null`), and
  `argus dns status` prints a `localhost door:` line.
- **The contract the coord hook reads.** While the door is bound, the JSON of the pid file
  (`server.pid` in the dashboard home, `~/.coord/dashboard/server.pid`) has `"localhostDoor": 80`
  beside `pid`, `port`, `startedAt` and `instance`. When the door is not bound the key is left
  out. The hook reads the file only, with no HTTP request, and builds
  `http://<key>.localhost/?thread=<id>` from it when the key is there. A stop removes the file.
- **Tests** never bind port 80: `ServeOptions::localhost_door_port` is `None` by default and
  tests set it to `Some(0)`. Only `argus serve` sets 80.

## The `.argus` names (opt-in)

Argus can have a name suffix of its own with real DNS: `http://argus.argus/` is the overview and
`http://rotp.argus/` the board of `rotp`, with no port. It is off until the owner turns it on, and
nothing about `.localhost:<port>` changes while it is off. It needs browser setup: Windows must
send `.argus` to the responder (Chrome and Edge honour that, Firefox does not use the Windows
DNS rules), and Claude's built-in browser pane blocks the page assets of these names.

### How it works

- **A loopback address of its own.** The default is `127.65.82.71`. It is inside `127.0.0.0/8`,
  so it is local to this computer, and it is never `127.0.0.1`, so no other program binds it.
  Ports 53 and 80 on it can never conflict with other software.
- **A tiny DNS responder** (crate `argus-dns`) on `<address>:53`, UDP and TCP. It answers an A
  query for the suffix and for anything under it with the address. AAAA and every other type
  of those names get an empty answer without an error, so a browser does not wait. Every other
  name is refused. There is no recursion and no forwarding, and the responder talks only on its
  own socket.
- **A front door.** The server also listens on `<address>:80` and routes by `Host` exactly as it
  does for `.localhost`: `argus.<suffix>` (and the bare suffix) is the overview, `<key>.<suffix>`
  a board, `<thread>.<key>.<suffix>` a thread. Any other `Host` is refused with 421, as before.
- **If port 80 is taken** (`http.sys` can own port 80 on every address), the server logs one line
  and listens on `<address>:<config port>` instead. The names then carry the port:
  `http://rotp.argus:47615/`. `argus dns status` says which one is in use.
- **One setting of the system, made by the owner.** Windows must send `.argus` to the responder.
  That is one NRPT rule, printed by `argus dns install --print`. Argus never runs it and never
  changes a system setting.

### Turn it on

1. In `config.json` of the dashboard home add:

   ```json
   "localNames": { "enabled": true, "address": "127.65.82.71", "suffix": "argus" }
   ```

   `address` and `suffix` are optional and show their defaults. The address must be in
   `127.0.0.0/8` and not `127.0.0.1`. The suffix is lowercase letters, digits and dashes, with
   dots between labels, and not `localhost`.
2. Restart Argus: `argus stop`, then `argus start`.
3. Print the setup and run it yourself in a PowerShell opened as administrator, once:

   ```text
   argus dns install --print
   ```

   It prints `Add-DnsClientNrptRule -Namespace ".argus" -NameServers "127.65.82.71"`. Chrome and
   Edge honour NRPT rules.
4. Check: `argus dns status` shows whether the names are on, whether the DNS answers (a real
   query to its socket), the front door port and whether Windows already resolves
   `argus.argus`.
5. Open `http://argus.argus/`. Type the first visit with `http://` so the browser does not
   search for the name.

Toast links follow the one link rule: `http://<key>.localhost/` while the localhost door is
bound, which the server knows itself. Only when the door is not bound and the names are on and
the responder is bound do toast links use `http://<key>.<suffix>/` (with the port only when the
front door could not take port 80). Otherwise they use `http://127.0.0.1:<port>`. A refused
`localNames` value (a bad address or suffix) is logged and leaves the names off.

### Turn it off

Set `"enabled": false` (or remove the key), restart Argus, and remove the rule with
`argus dns remove --print`, run as administrator.

### Health answer, Host check and tests

- The health answer carries a `localNames` object (address, suffix, door port, DNS socket) only
  while the names are up. `argus dns status` and `argus open` read it.
- The `Host` check accepts the port only when it is not 80: a browser sends `Host: rotp.argus`
  for port 80 and `Host: rotp.argus:47615` otherwise.
- Tests never bind port 53 or 80: the responder and the door take port 0 through
  `ServeOptions::local_names`.
