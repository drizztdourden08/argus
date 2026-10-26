CREATE TABLE kv (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE docs (scope TEXT NOT NULL, coll TEXT NOT NULL, id TEXT NOT NULL, data TEXT NOT NULL,
  hash TEXT NOT NULL, seq INTEGER NOT NULL, updated_at TEXT NOT NULL,
  PRIMARY KEY (scope, coll, id)) WITHOUT ROWID;
CREATE INDEX docs_scope_seq ON docs(scope, seq);
CREATE TABLE changes (seq INTEGER PRIMARY KEY AUTOINCREMENT, scope TEXT NOT NULL, coll TEXT NOT NULL,
  id TEXT NOT NULL, op TEXT NOT NULL CHECK (op IN ('set','delete')), at TEXT NOT NULL);
CREATE INDEX changes_scope ON changes(scope, seq);
CREATE TABLE posts (id TEXT PRIMARY KEY, scope TEXT NOT NULL, kind TEXT NOT NULL,
  source TEXT NOT NULL CHECK (source IN ('page','mcp','cli')), body TEXT NOT NULL, status TEXT NOT NULL,
  outcome TEXT, received_at TEXT NOT NULL, done_at TEXT);
CREATE INDEX posts_received ON posts(status) WHERE status = 'received';
CREATE TABLE launch_defs (scope TEXT NOT NULL, name TEXT NOT NULL, def TEXT NOT NULL, hash TEXT NOT NULL,
  updated_at TEXT NOT NULL, PRIMARY KEY (scope, name)) WITHOUT ROWID;
CREATE TABLE runs (id TEXT PRIMARY KEY,
  scope TEXT NOT NULL, rid TEXT NOT NULL, name TEXT NOT NULL, kind TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('queued','running','exited','failed','stopped')),
  exit_code INTEGER, signal TEXT, pid INTEGER, cmd TEXT NOT NULL, cwd TEXT NOT NULL,
  visible INTEGER NOT NULL DEFAULT 0, origin TEXT NOT NULL, by TEXT NOT NULL, agent TEXT, reason TEXT,
  queued_at TEXT NOT NULL, started_at TEXT, ended_at TEXT,
  log_bytes INTEGER NOT NULL DEFAULT 0, log_dropped INTEGER NOT NULL DEFAULT 0, log_seq INTEGER NOT NULL DEFAULT 0,
  orphan INTEGER NOT NULL DEFAULT 0, error TEXT, last_line TEXT, instance TEXT NOT NULL,
  UNIQUE (scope, rid));
CREATE INDEX runs_scope_time ON runs(scope, queued_at DESC);
CREATE INDEX runs_live ON runs(status) WHERE status IN ('queued','running');
CREATE TABLE run_logs (run_id TEXT NOT NULL, seq INTEGER NOT NULL, stream TEXT NOT NULL CHECK (stream IN ('out','err','sys')),
  at TEXT NOT NULL, text TEXT NOT NULL, head INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (run_id, seq)) WITHOUT ROWID;
CREATE TABLE attachments (id TEXT PRIMARY KEY, scope TEXT NOT NULL,
  owner_kind TEXT NOT NULL CHECK (owner_kind IN ('test','run')), owner_id TEXT NOT NULL, n INTEGER,
  path TEXT NOT NULL, mime TEXT NOT NULL, bytes INTEGER NOT NULL, sha256 TEXT NOT NULL,
  caption TEXT, created_by TEXT NOT NULL, created_at TEXT NOT NULL);
CREATE INDEX attachments_owner ON attachments(scope, owner_kind, owner_id);
