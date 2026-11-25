INSERT INTO runs (job_id, trigger, state, queued_at)
SELECT jobs.id, 'startup', 'queued', ?2 FROM jobs JOIN active_jobs ON active_jobs.id = jobs.id
WHERE jobs.id = ?1 AND jobs.paused = 0
