CREATE TABLE IF NOT EXISTS licenses (
    {id_column},
    identifier TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_deprecated INTEGER NOT NULL,
    is_free_software INTEGER NOT NULL,
    is_open_source INTEGER NOT NULL
)
