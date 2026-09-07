CREATE TABLE IF NOT EXISTS research_activity_revisions (
    {id_column},
    iid TEXT NOT NULL UNIQUE,
    research_activity_iid TEXT NOT NULL,
    rad_json TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL
)
