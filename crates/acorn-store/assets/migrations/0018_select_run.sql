SELECT id, job_id, trigger, state, queued_at, started_at, finished_at, exit_code, report_path, detail FROM runs WHERE id = ?1
