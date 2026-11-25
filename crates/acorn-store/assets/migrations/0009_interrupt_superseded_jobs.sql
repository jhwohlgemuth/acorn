UPDATE runs SET state = 'interrupted', finished_at = ?2, detail = 'job changed during reload'
WHERE job_id = ?1 AND state = 'queued'
