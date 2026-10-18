CREATE TABLE IF NOT EXISTS gitlab_event_cursors (
    source TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    event_id TEXT NOT NULL
);
