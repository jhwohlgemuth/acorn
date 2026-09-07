CREATE TABLE IF NOT EXISTS activity (
    {id_column},
    command TEXT NOT NULL,
    executed_at TEXT NOT NULL,
    user_path TEXT,
    success INTEGER NOT NULL DEFAULT 1
)
