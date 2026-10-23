CREATE TABLE IF NOT EXISTS logbook_entries (
    {id_column},
    research_activity_iid TEXT NOT NULL,
    entry_identifier TEXT NOT NULL,
    entry_json TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    archived INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (research_activity_iid, entry_identifier)
)
