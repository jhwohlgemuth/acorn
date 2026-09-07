CREATE TABLE IF NOT EXISTS providers (
    {id_column},
    provider_id TEXT NOT NULL UNIQUE,
    name TEXT,
    description TEXT,
    endpoint TEXT,
    documentation TEXT,
    authentication TEXT,
    env TEXT,
    npm TEXT,
    url TEXT,
    established_date TEXT,
    last_updated TEXT,
    models TEXT
)
