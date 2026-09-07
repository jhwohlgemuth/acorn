CREATE TABLE IF NOT EXISTS discoveries (
    {id_column},
    discovered_at TEXT NOT NULL,
    identifier TEXT NOT NULL,
    identifier_type TEXT NOT NULL,
    metadata TEXT,
    resolution_status TEXT NOT NULL,
    source TEXT NOT NULL,
    source_format TEXT NOT NULL
)
