# Vergil operator runbook

Use the approved native ARM64 `acorn` binary, protected ACRE file, and service environment on Vergil. The protected environment supplies `CI_SERVER_HOST=code.ornl.gov`, a bot-scoped `GITLAB_TOKEN`, a persistent `ACORN_DATABASE_PATH`, and `POWER_AUTOMATE_INBOUND_SECRET` referenced by `variables.custom`; it supplies `POWER_AUTOMATE_CALLBACK_URL` when callbacks are enabled. Do not put their values in command arguments, source, or ordinary logs. Confirm `uname -m` is `aarch64` or `arm64` and record `acorn --version` with the pinned binary revision. Use one durable database for the bot and one-shot commands; stop the service before manually processing the same pilot inbox.

After BOT-01, PA-03 starts the managed service with a pilot-project ID and a first-boot ISO 8601 UTC cursor chosen before the first review event. Use the [ACRE example](configuration-examples.md) only after replacing its placeholder project, folders, and variable references with approved values; omit `options.powerAutomate.intake` for this first boot. PA-05 adds that source after the flow and branch are approved. On later restarts, retain the same database and omit `--after` so the durable event cursor controls the resume point. Keep the HTTP listener on Vergil loopback; the selected path uses GitLab polling and needs no public endpoint.

Run this foreground command in the service supervisor or a dedicated shell:

```sh
acorn serve bot 22464 --config "$ACORN_CONFIG" --event-source poll \
  --bind 127.0.0.1:3000 --poll-interval 10 --after "$INITIAL_AFTER"
```

From a second shell on Vergil, check the running service:

```sh
curl -fsS http://127.0.0.1:3000/health
curl -fsS http://127.0.0.1:3000/ready
curl -fsS http://127.0.0.1:3000/state
```

After at least one polling interval, `/state` must show `poll_count` above zero and `last_error` absent. `/health` and `/ready` establish local liveness and operation-runtime readiness; the successful state poll establishes a GitLab review-event read. Check supervised restart, the retained cursor, and redacted logs. When PA-05 has an approved pilot inbox, stop the bot before a one-shot intake check; `intake run --config` can admit records, remove exactly admitted inbox lines, and process review work including draft-MR creation, so use only the authorized isolated branch and inspect its JSON summary before restarting the service.

After stopping the first service instance, use this command for the restart check and the managed service definition; leave out the first-boot cursor:

```sh
acorn serve bot 22464 --config "$ACORN_CONFIG" --event-source poll \
  --bind 127.0.0.1:3000 --poll-interval 10
```

Run the second-shell `/state` check again and compare its cursor with the durable pre-restart value.

```sh
acorn intake run --config "$ACORN_CONFIG"
acorn intake export --config "$ACORN_CONFIG" \
  --submission-id "$SUBMISSION_ID" --output change.diff
acorn intake deliver --config "$ACORN_CONFIG"
acorn report --dry-run --config "$ACORN_CONFIG" --root "$REVIEWED_CHECKOUT" \
  --commit "$MERGED_SHA" --through "$REPORT_CUTOFF" \
  --project "$PROJECT_ID" --output "$REPORT_OUTPUT"
```

Run `intake export` against the draft merge request created during PA-05 and accepted by PA-06, `intake deliver` only after PA-04 accepts the callback endpoint, and report dry-run after PA-07 records the merged commit. These are operator instructions for future acceptance, not evidence that Vergil has run them. PA-09 preserves the verified sequence in the durable runbook.

