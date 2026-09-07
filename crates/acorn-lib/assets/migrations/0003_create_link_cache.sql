CREATE TABLE IF NOT EXISTS link_cache (
    {id_column},
    url TEXT NOT NULL UNIQUE,
    status_code INTEGER NOT NULL,
    is_reachable INTEGER NOT NULL,
    checked_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
)
