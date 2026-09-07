CREATE TABLE IF NOT EXISTS validation_history (
    {id_column},
    path TEXT NOT NULL,
    check_type TEXT NOT NULL,
    success INTEGER NOT NULL,
    message TEXT NOT NULL,
    checked_at TEXT NOT NULL
)
