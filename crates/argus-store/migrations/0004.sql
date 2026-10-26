CREATE TABLE wake_faults (id INTEGER PRIMARY KEY AUTOINCREMENT, scope TEXT NOT NULL, thread TEXT NOT NULL,
  since TEXT NOT NULL, reason TEXT NOT NULL, wake_id TEXT, detail TEXT,
  cleared_at TEXT, cleared_by TEXT, note TEXT);
CREATE UNIQUE INDEX wake_faults_open ON wake_faults(scope, thread) WHERE cleared_at IS NULL;
