CREATE TABLE notifications (n INTEGER PRIMARY KEY AUTOINCREMENT, at TEXT NOT NULL,
  level TEXT NOT NULL CHECK (level IN ('info','ping','urgent')), title TEXT NOT NULL, text TEXT NOT NULL,
  project TEXT, thread TEXT, by TEXT NOT NULL CHECK (by IN ('thread','overlord','auto','page')),
  toast TEXT NOT NULL CHECK (toast IN ('shown','skipped','failed','limited')), url TEXT, why TEXT);
