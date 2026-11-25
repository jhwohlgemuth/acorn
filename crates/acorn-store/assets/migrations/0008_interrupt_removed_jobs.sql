UPDATE runs SET state = 'interrupted', finished_at = ?1, detail = 'job removed from current configuration'
WHERE state = 'queued' AND job_id NOT IN (SELECT id FROM active_jobs)
