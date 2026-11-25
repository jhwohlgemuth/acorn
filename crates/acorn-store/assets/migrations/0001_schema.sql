CREATE TABLE jobs (id TEXT PRIMARY KEY NOT NULL, paused INTEGER NOT NULL DEFAULT 0 CHECK (paused IN (0, 1)));
CREATE TABLE runs (
id INTEGER PRIMARY KEY NOT NULL,
job_id TEXT NOT NULL REFERENCES jobs(id),
trigger TEXT NOT NULL,
state TEXT NOT NULL CHECK (state IN ('queued', 'running', 'succeeded', 'failed', 'interrupted')),
queued_at INTEGER NOT NULL,
started_at INTEGER,
finished_at INTEGER,
exit_code INTEGER,
report_path TEXT,
detail TEXT
);
CREATE INDEX runs_job_id ON runs(job_id, id DESC);
PRAGMA user_version = 1;
