UPDATE runs SET state = 'running', started_at = ?2 WHERE id = ?1 AND state = 'queued'
