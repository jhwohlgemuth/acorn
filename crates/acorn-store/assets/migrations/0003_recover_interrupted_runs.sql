UPDATE runs SET state = 'interrupted', finished_at = ?1, detail = 'daemon stopped before run completed' WHERE state IN ('running', 'queued')
