CREATE TABLE IF NOT EXISTS webhook_intake_admissions (
    operation_key TEXT PRIMARY KEY,
    raw TEXT NOT NULL,
    digest TEXT NOT NULL,
    admission_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS webhook_intake_records (
    source TEXT NOT NULL,
    commit_sha TEXT NOT NULL,
    line BIGINT NOT NULL,
    digest TEXT NOT NULL,
    record_json TEXT NOT NULL,
    PRIMARY KEY (source, commit_sha, line, digest)
);
