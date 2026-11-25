DELETE FROM runs WHERE state IN ('succeeded', 'failed', 'interrupted') AND queued_at < ?1
