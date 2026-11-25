UPDATE runs SET state = ?2, finished_at = ?3, exit_code = ?4, report_path = ?5, detail = ?6
WHERE id = ?1 AND state = 'running'
