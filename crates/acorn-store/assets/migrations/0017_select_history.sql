SELECT id, job_id, trigger, state, queued_at, started_at, finished_at, exit_code, report_path, detail FROM runs WHERE job_id = ?1 ORDER BY id DESC LIMIT ?2
