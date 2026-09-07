CREATE TABLE IF NOT EXISTS programming_languages (
    {id_column},
    language_id INTEGER,
    name TEXT NOT NULL UNIQUE,
    language_type TEXT,
    color TEXT,
    group_name TEXT
)
