---
type: Plan
title: "Add GitLab bot analysis and citation intake workflows"
description: "Analyze merge requests and collect validated research artifact citations from GitLab work items."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, api, gitlab, bot, webhook, citation]
status: in-progress
priority: next
---

# Goal

Use authenticated GitLab webhooks to guide research artifact citation collection in one designated project. The bot automatically analyzes every opened, reopened, or updated merge request, analyzes work-item content when a human posts `/acorn check`, guides submitters through missing citation metadata, and proposes valid CFF 1.2.0 records in draft merge requests.

# Current evidence

- `crates/acorn-lib/src/io/api/gitlab/bot.rs` now accepts authenticated GitLab webhook deliveries, persists canonical operations before returning `202 Accepted`, and dispatches head-specific merge request analysis through a database-backed worker alongside the existing project-events poller.
- `crates/acorn-lib/src/io/api/gitlab/database.rs` provides durable delivery and operation records for SQLite and DuckDB, including deduplication, stale-work recovery, bounded retries, terminal failures, and queue counters without storing raw request bodies.
- `acorn check --merge-request` discovers changed paths from GitLab CI variables and assumes those files exist in a local checkout. The independently deployed bot has no merge request workspace, so it must fetch and materialize changed files at the merge request head SHA.
- `crates/acorn-lib/src/analyzer/service.rs` now exposes reusable structured checks for CFF, RAD JSON/JSONC/YAML, text, and DOCX inputs, while `discovery.rs` provides generic artifact grouping and deterministic hash helpers, PID normalization reuses `schema::pid`, and creator slugs live with the GitLab API boundary.
- The CFF schema requires a title and authors. An identifier alone can start intake, but the bot must request missing fields when deterministic enrichment cannot create a valid CFF record.
- The GitLab client now supports merge request details and paginated diffs, fork-aware repository file reads at an exact SHA, external commit statuses, work-item reads and paginated notes, effective project membership, bot-authored report-note upserts, instance-version discovery, and idempotent project-hook registration. It does not yet expose the branch, commit, merge request creation, label, or Task mutation operations required by citation repository writes and completion handling.
- Detached bot creation now publishes the selected port, mounts persistent bot state, binds the webhook server to `0.0.0.0`, forwards credential names without rendering values, and can register an externally reachable HTTPS webhook before serving.
- GitLab project webhook registration exposes merge request and note triggers but no separate `work_item_events` field. The `/acorn check` command arrives through a Note Hook.
- GitLab Standard Webhooks signing tokens start in GitLab 19.0; GitLab 17.2 compatibility requires the `X-Gitlab-Token` path.
- GitLab's Events API omits some merge request discussion notes, so polling cannot be treated as complete recovery for webhook delivery.

# Decisions

- Use one configured GitLab project as both the scientist intake project and citation repository.
- Add `POST /webhooks/gitlab` to the existing Axum server rather than creating a separate service.
- Make the database the durable operation queue. Persist an authenticated, normalized event and its canonical operation before returning `202 Accepted`; do not acknowledge work held only in memory.
- Keep raw-body authentication, payload decoding, event normalization, command recognition, analysis, report rendering, and GitLab mutations as separate boundaries. Keep deterministic parsing, grouping, slugging, and report construction pure.
- Automatically analyze merge requests on open, reopen, and head-SHA updates. Analyze supported changed files plus the merge request title and description, upsert one bot-authored report note per merge request, and publish an `acorn/check` commit status for the analyzed head SHA.
- Do not collect citations from ordinary merge request descriptions. Citation collection begins only from a work item whose human-authored comment contains an exact, trimmed `/acorn check` command line.
- Analyze the work-item title, description, and preceding human comments while excluding bot, system, and command comments.
- Allow multiple artifacts per work item. Extract DOI, ISBN, patent, ARK, RAiD, and public HTTP(S) artifact or repository URLs. Treat ORCID and ROR values as contributor or organization metadata, not artifact candidates.
- Group identifiers only when resolver metadata proves a shared canonical DOI or URL, or the enriched normalized title and authors match. Otherwise retain separate candidates.
- Accept `/acorn check` only when its author is the work-item creator or has an effective GitLab project access level of Maintainer or higher (`access_level >= 40`). Compare stable numeric user IDs, include inherited project membership, and deny safely when authorization cannot be established.
- Include every candidate that produces valid CFF title and author data in the draft citation merge request. Exclude incomplete candidates from repository writes and clearly list each exclusion and its missing `Identifier`, `Title`, or `Authors` fields in both the work-item report and draft merge request description. If no candidate is valid, post the checklist without creating a branch or merge request.
- Store each accepted artifact at `citations/work-items/<work-item-iid>/<creator-username>-<identifier-kind>-<identifier-hash>/CITATION.cff`.
- Derive `<creator-username>` from the user who created the work item, not the user who invokes `/acorn check`. Normalize it to lowercase kebab case, fall back to `user-<gitlab-user-id>` when empty, and persist the first observed creator slug so later account renames do not move existing records.
- Derive `<identifier-hash>` from the first 12 lowercase hexadecimal characters of SHA-256 over the normalized primary identifier.
- Create or update one bot branch and draft citation merge request per work item. Repository commits are the citation system of record; the bot database stores processing state and GitLab object references only.
- When a bot-created citation merge request is merged, idempotently apply the `acorn::collected` label to the source work item and post a comment linking the merged merge request. The label means at least one citation from the intake was merged; excluded candidates remain listed for follow-up. Leave the work item open for the scientist or reviewer to close.
- Support GitLab 17.2 and newer. Use `GITLAB_WEBHOOK_TOKEN` as the baseline inbound credential and optional `GITLAB_WEBHOOK_SIGNING_TOKEN` when the target supports Standard Webhooks. Keep both separate from outbound `GITLAB_TOKEN`.
- Register `merge_requests_events` and `note_events` only. Confidential-note intake, group epics, cross-project intake, required LLM inference, and automatic collection from ordinary merge requests are outside the MVP.
- Retain polling as an explicit legacy or best-effort reconciliation source. Reuse canonical operation keys where polling contains enough data, but do not promise complete parity with webhooks.

# GitLab workflows

## 1. Deploy and connect the bot

An operator starts the bot for one configured GitLab project with an outbound API token, an inbound webhook credential, persistent state, and an externally reachable HTTPS URL. The bot registers merge request and note triggers idempotently, authenticates each delivery before decoding it, persists a canonical operation before acknowledging it, and processes queued work with bounded retries. Health, readiness, queue state, and failures remain observable across restarts.

## 2. Analyze every merge request

An opened, reopened, or head-updated merge request queues one operation for its source head SHA. The bot sets the `acorn/check` commit status to `running`, fetches the paginated diff and supported non-deleted files at that SHA, and analyzes those files plus the merge request title and description. It then upserts one report note and changes the same status to `success` when analysis completes without failure-severity checks, or `failed` when a failure-severity check occurs, required supported content cannot be fetched or analyzed, or the workflow terminates in error. Skipped unsupported inputs are reported but do not fail the status. GitLab branch protection can require this status when the project wants analysis to block merging.

## 3. Start citation intake from a work item

A scientist creates an Issue or Task describing completed work or a research artifact. An exact, trimmed `/acorn check` command from the work-item creator or a project Maintainer starts intake; other authors receive an authorization response and cause no repository write. The bot analyzes the title, description, and preceding human comments, excluding bot, system, confidential, and command comments, then extracts, normalizes, enriches, and groups artifact candidates.

## 4. Propose valid citations and exclude incomplete candidates

The bot validates each candidate independently. Valid candidates become creator-prefixed `CITATION.cff` files in one work-item branch and draft merge request. Incomplete candidates are not committed and are clearly listed as excluded, with missing fields and correction guidance, in both the work-item report and draft merge request description. A later authorized `/acorn check` updates the draft: corrected candidates can move from excluded to included, while unchanged valid files remain stable. If every candidate is incomplete, the bot posts only the correction checklist.

## 5. Record collection without closing scientific review

When the bot-created citation merge request merges, the bot applies `acorn::collected` to the source work item and posts a link to the merged merge request exactly once. The source item stays open so the scientist or reviewer controls closure. Any excluded candidates remain visible; later corrections create the documented update or follow-up merge-request path.

## 6. Recover safely

Webhook redelivery, duplicate commands, polling overlap, worker retry, and process restart converge on the same canonical operations. Existing report notes, branches, merge requests, labels, and completion comments are updated or reused instead of duplicated. Polling remains best-effort reconciliation because GitLab's Events API cannot reproduce every webhook delivery.

# Public interfaces and data contracts

- Add normalized bot events equivalent to `MergeRequestChanged { project_id, iid, head_sha }`, `WorkItemCheckRequested { project_id, iid, note_id, command_author_id, creator }`, and `CitationMergeCompleted { project_id, merge_request_iid, work_item_iid, merge_commit_sha }`.
- Add durable delivery and operation records with queued, running, succeeded, retryable-failure, and terminal-failure states.
- Add structured merge request and work-item analysis reports so library code returns data without printing or exiting.
- Add GitLab client operations for merge request details and paginated diffs, repository file reads, commit-status publication, work-item reads and notes, effective project-member lookup, work-item label updates, branch and commit creation, merge request creation and lookup, note creation and update, instance version discovery, and project webhook registration.
- Resolve non-creator authorization with `GET /projects/:id/members/all/:user_id`; accept `access_level >= 40`, and treat not-found, lower access, and lookup failure as unauthorized. For completion, add `acorn::collected` through the Issue update API for Issues or `workItemUpdate` with the labels widget for Tasks, according to the persisted source type and global ID.
- Add `--event-source poll|webhook|hybrid`, retaining `poll` as the backward-compatible default. Add `--public-url` and `--register-webhook` to detached bot creation.
- In webhook mode, require at least one inbound webhook credential. Prefer HMAC whenever `webhook-signature` is present and never fall back after an invalid signature; otherwise use constant-time `X-Gitlab-Token` comparison.
- Select delivery identifiers in this order: `webhook-id`, `Idempotency-Key`, then `X-Gitlab-Webhook-UUID`.
- Use canonical operation keys `mr-check:<project>:<iid>:<head-sha>`, `work-item-check:<project>:<iid>:<note-id>`, and `citation-complete:<project>:<merge-request-iid>:<merge-commit-sha>`.

# Implementation steps

Steps are ordered by impact, then by estimated difficulty within the same impact level. Each step must meet its verification gate before the next dependent step begins.

| Order | Impact | Difficulty | Step | Verification gate |
|---:|---|---|---|---|
| 1 ✅ | high | medium | Add authenticated webhook ingress and immutable payload DTOs. Capture the exact raw body with a 1 MiB limit, validate a five-minute Standard Webhooks timestamp window, authenticate before deserialization, allow only the configured project, and normalize supported merge request and Note Hook payloads. | Router tests cover valid and invalid legacy tokens, valid and invalid HMAC signatures, stale timestamps, signature precedence, malformed JSON, oversized bodies, wrong projects, bot/system authors, and unsupported events without contacting GitLab. |
| 2 ✅ | high | medium | Add durable delivery and operation tables through the existing SQLite/DuckDB boundary. Transactionally record the normalized event and unique operation key before returning `202`; return `503` when persistence fails. Let workers claim database rows, recover stale running work after restart, retry five times with bounded backoff, and retain terminal failures. Do not persist raw webhook bodies. | Duplicate-delivery, duplicate-operation, database-failure, retry-exhaustion, and restart tests observe one operation and no lost or duplicated mutation. Database inspection confirms raw payloads are absent. |
| 3 ✅ | high | medium | Extract the CLI-private check collection path into a reusable library analysis service. Return structured `Check` results without printing or process exit. Classify CFF, JSON/JSONC, YAML, text, and DOCX inputs and add pure identifier normalization, candidate grouping, creator-slug generation, and report rendering helpers. | Existing CLI check tests remain unchanged in behavior; focused unit tests cover mixed standards, identifier normalization, grouping, username normalization/fallback, and deterministic hashes. |
| 4 ✅ | high | high | Implement automatic merge request analysis. Publish `acorn/check=running`, fetch paginated diffs, materialize non-deleted supported files at the head SHA in an isolated temporary workspace, analyze the title and description as text, and upsert one hidden-marker bot note containing checked files, errors/warnings, citation candidates, skipped inputs, and the analyzed SHA. Publish `success` only when analysis completes without failure-severity checks; publish `failed` for failure-severity checks, required supported-content failures, or terminal workflow errors. | Mock end-to-end tests cover open, reopen, update, duplicate SHA, forks, pagination, renamed/deleted/generated/oversized files, mixed standards, inaccessible content, terminal errors, status transitions, and updating rather than duplicating the report note. |
| 5 ✅ | high | high | Implement authorized `/acorn check` intake. Fetch the command author, stable work-item creator, effective project membership, and complete human-authored content; allow only the creator or `access_level >= 40`; then extract and normalize candidate identifiers, enrich DOI candidates through CiteAs, consume an existing repository CFF when available, and map complete candidates into CFF 1.2.0. Classify each candidate as included or excluded and render an updatable missing-metadata checklist for exclusions. | Issue and Task fixtures cover exact command matching, creator authorization, direct and inherited Maintainer authorization, Reporter denial, membership lookup failure, a different command author than creator, multiple artifacts, unresolved grouping, enrichment failure, and correction reruns. |
| 6 | high | high | Implement citation repository writes and completion handling. Generate creator-prefixed artifact slugs, validate each included CFF with ACORN schema and link checks, create or update one work-item branch and draft merge request, and link both the source work item and creator. Commit valid candidates while omitting and listing incomplete candidates in the work-item report and merge request description. Repeated commands update the open draft; unchanged merged intake links the completed merge request; materially changed intake creates a content-hash follow-up branch. On citation-MR merge, apply `acorn::collected`, post the merged-MR link once, and leave the work item open. | Mock API tests cover deterministic paths, creator-versus-command-author selection, username changes, slug collisions, mixed valid/incomplete intake, zero-valid intake, correction promotion, existing branch/MR updates, duplicate commands, merged intake, idempotent label/comment completion, open work-item state, follow-up content, and permission errors. |
| 7 | medium | medium | Make webhook deployment reachable and durable. Bind webhook containers to `0.0.0.0`, publish the selected port, mount a persistent named volume for bot state, require an externally reachable HTTPS public URL, and idempotently register matching merge request and note triggers. Use legacy tokens for GitLab 17.2-18.x and signing tokens when supported. | CLI/container tests prove port and volume arguments; mock registration tests cover both authentication eras, exact trigger fields, existing hooks, permission errors, reruns, and credential redaction; a container smoke test reaches `/health`. |
| 8 | medium | low | Add operational state and documentation. Separate readiness from liveness, expose queue/running/failed/deduplication counters and the last success/error through `/state`, drain workers gracefully, document token permissions, reverse-proxy TLS, webhook history/resend, scientist intake, correction templates, creator-prefixed paths, and staging validation, then regenerate CLI help. | State and shutdown tests pass; `make help`, focused crate tests, applicable feature checks, `cargo fmt --check`, and `git diff --check` pass; staging MR, Issue, Task, and resend deliveries are visible in GitLab history without duplicate output. |

# Planned files

| Area | Change |
|---|---|
| `crates/acorn-lib/src/io/api/gitlab/` | Add webhook ingress, GitLab DTOs/API operations, durable dispatch, MR/work-item workflows, repository writes, and focused fixtures/tests. |
| `crates/acorn-lib/src/analyzer/` and `crates/acorn-lib/src/io/database/` | Expose structured analysis and add durable delivery/operation state plus pure citation/slug helpers at the appropriate existing boundary. |
| `crates/acorn-cli/src/cli/arguments.rs` and bot command modules | Add event-source, public URL, registration, port publication, persistent volume, credential forwarding, and generated help changes. |
| Embedded GitLab endpoint metadata and workspace manifests | Add only the required GitLab operations and cryptographic dependencies. |
| Bot deployment and user documentation | Document the complete intake, correction, review, security, and staging workflows. |

# Validation

Completed non-Docker validation on 2026-07-24:

- `cargo nextest run --no-fail-fast`: 806 passed and 14 ignored.
- `cargo nextest run --run-ignored all --no-fail-fast`: 818 passed; the two failures were pre-existing live GitLab tests whose configured remote project returned no merge-request files or release.
- Focused SQLite and DuckDB queue tests, webhook router tests, hook registration tests, structured analysis and citation helper tests, container argument tests, CLI parsing tests, and existing check-command tests passed.
- `cargo check -p acorn-lib`, `cargo check -p acorn-lib --features duckdb`, and `cargo check -p acorn-lib --no-default-features` passed.
- Clippy passed with warnings denied for `acorn-lib`; `acorn-cli` passed with only its pre-existing unrelated test-only `unused_variables` warning allowed.
- `cargo fmt --all -- --check` and generated CLI help passed.
- The step 7 container `/health` smoke remains open because it requires a successful Docker image build and runtime.
- The 2026-07-25 Hunk refactor passed default, no-default-feature, and DuckDB `acorn-lib` checks, the `acorn-cli` check, and focused webhook pagination, credential refresh, awaited worker, PID normalization, discovery grouping, container argument, and CLI parsing tests.
- Merge request workflow tests now cover head-specific open/reopen/update operation keys, stale SHA suppression, default durable-worker dispatch, paginated diffs, fork source-project reads, renamed/deleted/generated/collapsed/oversized inputs, mixed supported standards, inaccessible content retries, `running` to `success`/`failed` status transitions, citation discovery, and bot-authored note creation/update. The full GitLab API test module passes, `cargo nextest run -p acorn-lib --no-fail-fast` passes 645 tests with 8 skipped, and `cargo clippy -p acorn-lib --all-targets --no-deps -- -D warnings` passes.
- Authorized work-item intake validation on 2026-08-03 covers exact command and fetched-author verification, creator and effective Maintainer authorization, safe Reporter and membership-failure denial, Issue and Task content, human-comment filtering, multiple unresolved artifacts, CiteAs mapping and failure retention, embedded and repository `CITATION.cff` reuse, included/excluded rendering, report creation/update, and durable worker dispatch. `cargo nextest run --package acorn-lib io::api::gitlab` passed 68 tests; `make test` passed 948 tests with 15 skipped; the default-feature Clippy gate and DuckDB feature check passed. The no-default-feature check remains blocked by the pre-existing unconditional `crate::io::ApiResult` import in `schema/hardware/memory.rs` while `io` is gated behind `std`.

1. Run focused pure helper, payload, router, database, worker, GitLab client, and workflow tests after each dependent step.
2. Run SQLite restart tests and the DuckDB feature check for the durable operation schema.
3. Start the bot in a container and verify liveness, readiness, persistent state, and authenticated webhook receipt through the published endpoint.
4. Register a staging hook and test merge request open/update, Issue `/acorn check`, and Task `/acorn check` deliveries.
5. Resend identical deliveries and restart the bot before retrying; verify one report note or repository mutation.
6. Verify every analyzed MR head receives `acorn/check` transitions from `running` to the expected `success` or `failed` state, including terminal worker errors.
7. Verify creator and effective Maintainer commands are accepted, lower roles and failed membership lookups are denied, and every accepted artifact slug still uses the creator username.
8. Verify mixed intake commits only valid CFF files, lists incomplete candidates as excluded in both reports, and promotes a corrected exclusion on rerun; verify zero-valid intake creates no branch, commit, or merge request.
9. Verify corrected multi-artifact intake creates valid CFF files under the work-item bundle and one draft merge request.
10. Merge a staging citation MR and verify one `acorn::collected` label and one merged-MR link appear while the work item remains open.
11. Verify bot-authored notes and the bot's own citation merge request do not create feedback loops or duplicate citation intake.
12. Run generated-help, formatting, focused Cargo checks/tests, applicable feature checks, link validation, and whitespace validation.

# Acceptance criteria

- Every opened, reopened, or head-updated merge request in the configured project receives one current ACORN analysis note and a final `acorn/check` pass/fail commit status for its head SHA.
- An exact human-authored `/acorn check` comment from the work-item creator or an effective project Maintainer on a project Issue or Task analyzes the work-item content and reports every detected artifact candidate; unauthorized commands make no repository changes.
- Invalid authentication, stale signed requests, oversized bodies, unsupported projects/events, and bot/system/confidential comments cannot start workflow work.
- Duplicate delivery, webhook resend, polling overlap, command replay, and process restart cannot duplicate a report note, citation file, branch, commit, or merge request.
- Identifier-only intake is accepted for analysis. Candidates with valid CFF title and authors enter the draft merge request immediately; incomplete candidates are omitted from commits and clearly listed as excluded until corrected.
- Each accepted artifact is stored as valid CFF 1.2.0 beneath `citations/work-items/<iid>/`, and its artifact slug begins with the stable normalized username of the work-item creator.
- Multiple artifacts from one work item produce distinct collision-resistant directories in one draft citation merge request.
- Merging a bot-created citation merge request applies `acorn::collected` and posts its link exactly once without closing the source work item, including when the intake report still lists excluded candidates.
- Detached webhook deployment has an explicit reachable HTTPS path, persistent operation state, idempotent hook registration, and no credential or sensitive-payload logging.
- Health, readiness, queue state, retry failures, last success/error, and deduplication counts are observable.
- Polling remains available for compatibility but is documented as incomplete reconciliation rather than an equivalent event source.

# Citations

[1] [GitLab webhook events](https://docs.gitlab.com/user/project/integrations/webhook_events/) - Merge request, comment, work-item payload, and group-epic behavior.

[2] [GitLab project webhooks API](https://docs.gitlab.com/api/project_webhooks/) - Project hook creation, available trigger fields, permissions, testing, history, and resend operations.

[3] [GitLab webhook signing tokens](https://docs.gitlab.com/user/project/integrations/webhooks/#signing-tokens) - Standard Webhooks HMAC verification, timestamp validation, delivery headers, and secret-token compatibility.

[4] [GitLab Events API](https://docs.gitlab.com/api/events/) - Polling limitations, including unsupported `DiscussionNote` merge request notes.

[5] [GitLab commit statuses API](https://docs.gitlab.com/api/commits/#set-commit-pipeline-status) - External commit status creation for a merge request source SHA.

[6] [GitLab project members API](https://docs.gitlab.com/api/project_members/) - Effective project membership and numeric access levels, including Maintainer level 40.

[7] [GitLab issues API](https://docs.gitlab.com/api/issues/#update-an-issue) - Adding labels while updating an Issue; Task implementations use the equivalent Work Item mutation.

[8] [GitLab Work Item API migration guide](https://docs.gitlab.com/api/graphql/epic_work_items_api_migration_guide/) - Updating a work item through `workItemUpdate` and widget inputs.
