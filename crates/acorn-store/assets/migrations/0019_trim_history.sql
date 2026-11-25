DELETE FROM runs WHERE state IN ('succeeded', 'failed', 'interrupted') AND job_id = (SELECT job_id FROM runs WHERE id = ?1)
AND (queued_at < ?2 OR id IN (
SELECT id FROM runs WHERE job_id = (SELECT job_id FROM runs WHERE id = ?1)
AND state IN ('succeeded', 'failed', 'interrupted') ORDER BY id DESC LIMIT -1 OFFSET ?3
))
