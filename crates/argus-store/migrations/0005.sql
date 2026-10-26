CREATE TABLE activity (n INTEGER PRIMARY KEY AUTOINCREMENT, at TEXT NOT NULL, kind TEXT NOT NULL,
  level TEXT NOT NULL CHECK (level IN ('info','warn','error')), text TEXT NOT NULL, project TEXT, thread TEXT);
CREATE INDEX activity_kind ON activity(kind, n);
