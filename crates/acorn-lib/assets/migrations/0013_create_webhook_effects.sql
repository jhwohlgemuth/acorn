CREATE TABLE IF NOT EXISTS webhook_effects (
    effect_key TEXT PRIMARY KEY,
    effect_kind TEXT NOT NULL,
    state TEXT NOT NULL,
    result_json TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
