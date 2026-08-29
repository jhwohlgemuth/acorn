---
type: Plan
title: "Finish GitLab citation repository and bot operations"
description: "Write validated citations from GitLab work items, handle collection completion, and finish bot operations."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, api, gitlab, bot, webhook, citation]
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/gitlab/`, `src/analyzer/service.rs`, bot command modules in `crates/acorn-cli`, fixtures, and mocked API/store tests.
- **Create:** protocol/client pieces in `acorn-api`, operation persistence in `acorn-store`, workflow orchestration in `acorn-service`, and composition in CLI/host after extraction.
- **Delete:** GitLab workflow orchestration from provider client modules once service use cases own it; keep protocol compatibility paths through the facade.

# Dependencies

Requires [claimed-operation capabilities](gitlab-claimed-operation-capabilities.md), stable API request/response/error contracts, and the analyzer service boundary. It is downstream of the portability crate split, not a prerequisite for `acorn-core`/`acorn-schema`.

# Goal

Finish the authenticated GitLab citation workflow by writing valid CFF 1.2.0 records to draft merge requests, handling merged-citation completion idempotently, and completing bot observability, shutdown, deployment, and operating documentation.

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
- Include every candidate that produces valid CFF title and author data in the draft citation merge request. Exclude incomplete candidates from repository writes and clearly list each exclusion and its missing `Identifier`, `Title`, or `Authors` fields in both the work-item report and draft merge request description. If no candidate is valid, post the checklist without creating a branch or merge request.
- Store each accepted artifact at `citations/work-items/<work-item-iid>/<creator-username>-<identifier-kind>-<identifier-hash>/CITATION.cff`.
- Derive `<creator-username>` from the user who created the work item, not the user who invokes `/acorn check`. Normalize it to lowercase kebab case, fall back to `user-<gitlab-user-id>` when empty, and persist the first observed creator slug so later account renames do not move existing records.
- Derive `<identifier-hash>` from the first 12 lowercase hexadecimal characters of SHA-256 over the normalized primary identifier.
- Create or update one bot branch and draft citation merge request per work item. Repository commits are the citation system of record; the bot database stores processing state and GitLab object references only.
- When a bot-created citation merge request is merged, idempotently apply the `acorn::collected` label to the source work item and post a comment linking the merged merge request. The label means at least one citation from the intake was merged; excluded candidates remain listed for follow-up. Leave the work item open for the scientist or reviewer to close.
- Prevent bot-authored notes and the bot's citation merge requests from creating feedback loops or duplicate intake.

# GitLab workflows

## 1. Operate the bot

Expose distinct readiness and liveness state, queue and operation counters, graceful worker draining, and actionable deployment guidance. Validate the published endpoint and persistent state with a container smoke test and staging webhook deliveries.

## 2. Propose valid citations and exclude incomplete candidates

The bot validates each candidate independently. Valid candidates become creator-prefixed `CITATION.cff` files in one work-item branch and draft merge request. Incomplete candidates are not committed and are clearly listed as excluded, with missing fields and correction guidance, in both the work-item report and draft merge request description. A later authorized `/acorn check` updates the draft: corrected candidates can move from excluded to included, while unchanged valid files remain stable. If every candidate is incomplete, the bot posts only the correction checklist.

## 3. Record collection without closing scientific review

When the bot-created citation merge request merges, the bot applies `acorn::collected` to the source work item and posts a link to the merged merge request exactly once. The source item stays open so the scientist or reviewer controls closure. Any excluded candidates remain visible; later corrections create the documented update or follow-up merge-request path.

## 4. Recover safely

Webhook redelivery, duplicate commands, polling overlap, worker retry, and process restart converge on the same canonical operations. Existing report notes, branches, merge requests, labels, and completion comments are updated or reused instead of duplicated. Polling remains best-effort reconciliation because GitLab's Events API cannot reproduce every webhook delivery.

# Public interfaces and data contracts

- Add GitLab client operations for branch and commit creation, merge request creation and lookup, work-item label updates, and Task mutation.
- Add `CitationMergeCompleted { project_id, merge_request_iid, work_item_iid, merge_commit_sha }` handling and apply `acorn::collected` through the Issue update API or `workItemUpdate`, according to the persisted source type and global ID.
- Add structured readiness, liveness, queue, running, failed, deduplication, last-success, and last-error state.

# Implementation steps

Steps are ordered by impact, then by estimated difficulty within the same impact level. Each step must meet its verification gate before the next dependent step begins.

| Order | Impact | Difficulty | Step | Verification gate |
|---:|---|---|---|---|
| 1 | high | high | Implement citation repository writes and completion handling. Generate creator-prefixed artifact slugs, validate each included CFF with ACORN schema and link checks, create or update one work-item branch and draft merge request, and link both the source work item and creator. Commit valid candidates while omitting and listing incomplete candidates in the work-item report and merge request description. Repeated commands update the open draft; unchanged merged intake links the merged merge request; materially changed intake creates a content-hash follow-up branch. On citation-MR merge, apply `acorn::collected`, post the merged-MR link once, and leave the work item open. | Mock API tests cover deterministic paths, creator-versus-command-author selection, username changes, slug collisions, mixed valid/incomplete intake, zero-valid intake, correction promotion, existing branch/MR updates, duplicate commands, merged intake, idempotent label/comment handling, open work-item state, follow-up content, and permission errors. |
| 2 | medium | medium | Validate webhook deployment against a reachable container and staging GitLab project. | A container smoke test reaches `/health`; staging merge request, Issue, Task, and resend deliveries are visible in GitLab history without duplicate output. |
| 3 | medium | low | Add operational state and documentation. Separate readiness from liveness, expose queue/running/failed/deduplication counters and the last success/error through `/state`, drain workers gracefully, document token permissions, reverse-proxy TLS, webhook history/resend, scientist intake, correction templates, creator-prefixed paths, and staging validation, then regenerate CLI help. | State and shutdown tests pass; `make help`, focused crate tests, applicable feature checks, `cargo fmt --check`, and `git diff --check` pass. |

# Planned files

| Area | Change |
|---|---|
| `crates/acorn-lib/src/io/api/gitlab/` | Add citation repository writes, completion handling, operational state, and focused fixtures/tests. |
| `crates/acorn-cli` bot command modules | Add graceful shutdown and expose remaining operational state. |
| Embedded GitLab endpoint metadata | Add only the GitLab branch, commit, merge request, label, and Task operations still required. |
| Bot deployment and user documentation | Document citation writes, correction, completion, security, and staging workflows. |

# Validation

1. Start the bot in a container and verify liveness, readiness, persistent state, and authenticated webhook receipt through the published endpoint.
2. Register a staging hook and test merge request open/update, Issue `/acorn check`, and Task `/acorn check` deliveries.
3. Resend identical deliveries and restart the bot before retrying; verify repository mutations are not duplicated.
4. Verify mixed intake commits only valid CFF files, lists incomplete candidates as excluded in both reports, and promotes a corrected exclusion on rerun; verify zero-valid intake creates no branch, commit, or merge request.
5. Verify corrected multi-artifact intake creates valid CFF files under the work-item bundle and one draft merge request.
6. Merge a staging citation MR and verify one `acorn::collected` label and one merged-MR link appear while the work item remains open.
7. Verify bot-authored notes and the bot's own citation merge request do not create feedback loops or duplicate citation intake.
8. Run generated-help, formatting, focused Cargo checks/tests, applicable feature checks, link validation, and whitespace validation.

# Acceptance criteria

- Duplicate delivery, webhook resend, polling overlap, command replay, and process restart cannot duplicate a citation file, branch, commit, merge request, label, or completion comment.
- Candidates with valid CFF title and authors enter the draft merge request immediately; incomplete candidates are omitted from commits and clearly listed as excluded until corrected.
- Each accepted artifact is stored as valid CFF 1.2.0 beneath `citations/work-items/<iid>/`, and its artifact slug begins with the stable normalized username of the work-item creator.
- Multiple artifacts from one work item produce distinct collision-resistant directories in one draft citation merge request.
- Merging a bot-created citation merge request applies `acorn::collected` and posts its link exactly once without closing the source work item, including when the intake report still lists excluded candidates.
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
