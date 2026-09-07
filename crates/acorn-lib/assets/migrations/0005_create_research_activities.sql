CREATE TABLE IF NOT EXISTS research_activities (
    {id_column},
    iid TEXT NOT NULL UNIQUE,
    rad_json TEXT NOT NULL,
    identity_keys_json TEXT NOT NULL,
    provenance_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)
