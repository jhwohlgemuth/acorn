INSERT INTO runs (job_id, trigger, state, queued_at)
SELECT jobs.id, ?2, 'queued', ?3 FROM jobs JOIN active_jobs ON active_jobs.id = jobs.id
WHERE jobs.id = ?1 AND jobs.paused = 0
