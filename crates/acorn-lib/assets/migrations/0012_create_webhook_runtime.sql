CREATE TABLE IF NOT EXISTS webhook_deliveries (
    delivery_id TEXT PRIMARY KEY,
    operation_key TEXT NOT NULL,
    event_json TEXT NOT NULL,
    received_at TEXT NOT NULL,
    deduplicated INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS webhook_operations (
    operation_key TEXT PRIMARY KEY,
    delivery_id TEXT NOT NULL,
    event_json TEXT NOT NULL,
    state TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    available_at TEXT NOT NULL,
    claimed_at TEXT,
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS webhook_callbacks (
    callback_key TEXT PRIMARY KEY,
    destination TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    state TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    available_at TEXT NOT NULL,
    claimed_at TEXT,
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS webhook_callback_attempts (
    callback_key TEXT NOT NULL,
    attempt INTEGER NOT NULL,
    status_code INTEGER,
    error TEXT,
    attempted_at TEXT NOT NULL,
    PRIMARY KEY (callback_key, attempt)
);
