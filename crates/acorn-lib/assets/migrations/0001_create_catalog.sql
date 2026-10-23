CREATE TABLE IF NOT EXISTS catalog (
    {id_column},
    bucket_name TEXT NOT NULL,
    bucket_url TEXT NOT NULL,
    identifier TEXT NOT NULL,
    title TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (bucket_name, identifier)
)
