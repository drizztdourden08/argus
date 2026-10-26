CREATE TABLE wakes (id TEXT PRIMARY KEY, n INTEGER NOT NULL, scope TEXT NOT NULL, thread TEXT NOT NULL,
  reason TEXT NOT NULL, subject TEXT, text TEXT NOT NULL, at TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('queued','delivered','coalesced')),
  delivered_at TEXT, listeners INTEGER NOT NULL DEFAULT 0,
  lines TEXT, count INTEGER NOT NULL DEFAULT 1, merged_into TEXT);
CREATE INDEX wakes_scope_n ON wakes(scope, n);
CREATE INDEX wakes_queued ON wakes(status, scope, thread) WHERE status = 'queued';
