SELECT runs.id FROM runs JOIN jobs ON jobs.id = runs.job_id JOIN active_jobs ON active_jobs.id = jobs.id
WHERE runs.state = 'queued' AND jobs.paused = 0
AND NOT EXISTS (SELECT 1 FROM runs WHERE state = 'running')
ORDER BY runs.id LIMIT 1
