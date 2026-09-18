---
type: Plan
title: "Automate PowerApps logbook intake and sponsor reporting"
description: "Turn signed PowerAutomate form submissions into reviewed bucket logbook changes and monthly sponsor PDFs delivered through GitLab and PowerAutomate."
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-cli, acorn-schema]
tags: [feature, automation, powerapps, powerautomate, gitlab, webhook, logbook, reporting, pdf]
status: planned
priority: next
---

# Goal

Deliver one reviewed, duplicate-safe path from Microsoft PowerApp form submission through a rolling GitLab merge request, selective logbook graduation, monthly PDF generation on GitLab runners, immutable GitLab package publication, and signed PowerAutomate notification for Teams and email delivery.

The MVP ends at a proven form-to-sponsor-artifact flow for one pilot project. Delivery receipts, broader graduation targets, and operational dashboards follow the MVP instead of delaying it.

# Current evidence

- `crates/acorn-lib/src/io/api/webhooks/` already provides Standard Webhooks verification, normalized deliveries, durable idempotent operations, reconciled effects, retryable callbacks, readiness, and graceful drain state.
- `crates/acorn-lib/src/io/api/gitlab/` already authenticates GitLab webhooks, authorizes human merge-request commands, reads repository files at exact revisions, and reconciles branch, commit, draft-MR, note, label, and commit-status effects.
- `crates/acorn-lib/src/io/workflow/` already turns immutable repository inputs into deterministic `RepositoryChangeSet` values.
- `crates/acorn-schema/src/research_activity/logbook.rs` already models validated logbook entries, proposes graduation candidates, applies accepted graduation, archives source entries, and selects updates from a lower timestamp bound.
- `crates/acorn-lib/src/io/database/logbook.rs` already persists complete activities and performs graduation with an immutable pre-change revision in one transaction.
- `crates/acorn-cli/src/commands/export/updates.rs` already renders since-filtered logbook updates to PDF, Markdown, and DOCX. It lacks an exclusive upper reporting bound and automated GitLab package publication.
- `acorn serve bot` already composes the GitLab webhook server and durable worker. There is no PowerAutomate provider module, project-folder intake map, rolling form-intake workflow, report package ledger, or scheduled monthly report command.

# Architecture decisions

- Keep every PowerAutomate-specific payload, header, signature, callback, endpoint, and configuration rule under `crates/acorn-lib/src/io/api/powerautomate/`. Other modules may register its router or consume normalized domain values but must not implement PowerAutomate communication.
- Reuse `WebhookProvider`, `Verifier`, `CallbackResolver`, `OperationQueue`, `OperationWorker`, `CallbackWorker`, GitLab automation, logbook graduation, and the existing updates renderer. Prefer implementations of those traits and inherent methods over new free functions.
- Treat the bucket GitLab repository as project truth. A deployment-owned map resolves a stable form `projectId` to a safe `<project-folder>/index.json` path and normalized submitter-email allowlist.
- Accept logbook additions only. Forms cannot submit repository paths, callback URLs, logbook identifiers, discovery provenance, archive state, or direct RAD patches.
- Maintain one active draft MR per project and reporting month. If it is already merged, open a deterministically numbered continuation MR. The bot never merges or bypasses approval rules.
- Require `/acorn graduate <entry-id>...` from an authorized human reviewer before selected candidates update canonical RAD fields. `/acorn graduate` without identifiers lists candidates without mutation.
- Run monthly reporting from the merged default-branch commit. The first successful report includes full history; later reports select entry timestamps in `[previous through, current through)`. Reject later form submissions whose entry timestamp predates the last successful cutoff.
- Publish the PDF and final `manifest.json` to a deterministic GitLab generic-package version. The final manifest is the reporting ledger; incomplete package versions do not advance the cutoff. A no-change period publishes only a manifest and notification.
- PowerAutomate owns Teams, email, sponsor recipients, and authenticated package download. ACORN sends signed metadata events and never embeds GitLab credentials or PDF bytes in them.

# Public interfaces

The new `io::api::powerautomate` module owns `PowerAutomateConfig`, `ProjectIntakeConfig`, `PowerAutomateProvider`, `PowerAutomateCallbackResolver`, `FormSubmission`, `FormReceipt`, `Submitter`, `PowerAutomateEvent`, and the post-MVP `DeliveryReceipt`.

Implement `WebhookProvider` for `PowerAutomateProvider`, `CallbackResolver` for `PowerAutomateCallbackResolver`, and `Validate` for configuration and wire types. Use inherent methods such as `PowerAutomateProvider::router`, `FormSubmission::entry`, `PowerAutomateConfig::project`, and `PowerAutomateEvent::enqueue` rather than provider-specific free functions.

`POST /webhooks/powerautomate/forms` accepts a strict version-one envelope containing `schemaVersion`, UUID `submissionId`, configured `projectId`, RFC3339 `submittedAt`, `submitter.email`, and an `entry` with timestamp, category, optional classification and event type, state, summary, and optional items. ACORN derives the stable logbook identifier and forces `origin=manual` and `archived=false`. New and duplicate deliveries return `202` with the canonical operation identifier and disposition.

Deployment configuration contains the bucket GitLab project, safe project-path mappings, normalized email allowlists, reporting timezone (UTC by default), named callback destination, separate inbound/outbound secret references, and package naming. URLs and secrets are resolved at runtime and never accepted from form data.

# Project code boundaries

- **Update:** the provider-neutral webhook signer/callback seam, GitLab automation and package provider, repository workflow registry, logbook reporting selection, updates renderer, bot/CLI composition, application configuration, generated CLI help, CI integration, and focused tests described in the numbered slices below.
- **Create:** `crates/acorn-lib/src/io/api/powerautomate/`, a transport-neutral logbook repository workflow leaf, a typed GitLab report-package leaf, a runner-facing report command, a reusable bucket-project GitLab CI template, dedicated fixtures, and workflow documentation.
- **Delete:** only rendering or publication branches made redundant by the shared implementation introduced by this plan; no existing public command or serialized contract is removed.

# Dependencies

This plan consumes the implemented JSON-RPC operation registry, webhook runtime, GitLab repository automation, canonical database/graduation contract, and updates PDF renderer. It precedes the remaining gRPC, ACP, and MCP work because those plans share the RPC, webhook, bot, manifest, and CLI integration locks.

[Database-backed artifact generation](database-backed-artifact-generation.md) is not an MVP prerequisite; it later reuses the bounded reporting-window and artifact-manifest contracts. [Research activity lifecycle, credit, license, and award fields](../refactors/core/research-activity-lifecycle-credit-fields.md) later expands deterministic graduation targets. [GitLab CI test runners](../refactors/ci-runner-pools.md) coordinates protected credentials and scheduled-report runner isolation.

# Implementation steps

## 1. Record the plan and priority

Add this concept to the feature index, root execution order, dependency network, and parallel-execution guide. Remove `priority: next` from gRPC, ACP, and MCP while preserving their relative order after this workflow.

- **Update:** `.wiki/index.md`, `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, and the gRPC, ACP, and MCP plan frontmatter.
- **Create:** this plan.
- **Delete:** None.
- **Lock:** planning-wiki indexes and dependency network.
- **Verify:** required OKF metadata is present, links resolve, and this plan is first in the wiki's strict highest-priority lane while lower lanes retain their independent candidates.

## 2. Establish the PowerAutomate module boundary

Add the module shell, public types, configuration, and module documentation. The API root only registers and re-exports it.

- **Update:** `crates/acorn-lib/src/io/api/mod.rs`, `crates/acorn-lib/src/io/config/application.rs`.
- **Create:** `crates/acorn-lib/src/io/api/powerautomate/{mod,config,form,event}.rs`, `crates/acorn-lib/src/io/api/powerautomate/tests/mod.rs`.
- **Delete:** None.
- **Lock:** module index and application configuration.
- **Verify:** the workspace compiles and a repository search confines PowerAutomate protocol identifiers to the module plus explicit composition points.

## 3. Implement strict form decoding and authorization

Implement immutable conversion from a form submission into a normal `LogbookEntry`. `FormSubmission::entry` validates the project, UUID, timestamps, normalized email, and form fields, then derives a stable kebab-case entry identifier.

- **Update:** `io/api/powerautomate/{config,form}.rs`, `io/api/powerautomate/tests/`.
- **Create:** focused form and configuration test leaves below `io/api/powerautomate/tests/`.
- **Delete:** None.
- **Lock:** PowerAutomate module only.
- **Verify:** reject unknown fields, invalid UUID/timestamps, unsafe paths, unauthorized emails, invalid logbook values, and caller-controlled effect data.

## 4. Add authenticated durable intake

Implement `WebhookProvider` for `PowerAutomateProvider` using `StandardWebhooksVerifier`. Classify one accepted form as `webhooks.powerautomate.form`, derive its idempotency key from schema version and submission UUID, enqueue it through the existing store, and return `202` immediately.

- **Update:** `io/api/powerautomate/mod.rs`, `io/api/powerautomate/tests/`, `crates/acorn-lib/src/io/api/gitlab/bot.rs`, `crates/acorn-cli/src/commands/serve/bot/mod.rs`.
- **Create:** `crates/acorn-lib/src/io/api/powerautomate/webhook.rs`.
- **Delete:** None.
- **Lock:** bot router, operation registry, and serve-bot composition.
- **Verify:** valid, invalid, missing, expired, replayed, and duplicate deliveries use the existing durable queue semantics.

## 5. Build a pure logbook repository proposal

Add a transport-neutral workflow that accepts normalized RAD content and a `LogbookEntry`, then returns a `RepositoryChangeSet`. Append missing entries, accept identical replay, reject identifier/content conflict, preserve unrelated fields, and emit one deterministic full-file replacement.

- **Update:** `crates/acorn-lib/src/io/workflow/mod.rs`, `crates/acorn-lib/src/io/workflow/registry.rs` only when registering the named workflow, and focused workflow tests.
- **Create:** `crates/acorn-lib/src/io/workflow/logbook.rs` and dedicated tests under `crates/acorn-lib/src/io/workflow/tests/`.
- **Delete:** None.
- **Lock:** workflow registry only during registration.
- **Verify:** pure tests cover append, replay, conflict, malformed RAD, order, and preservation of unrelated content.

## 6. Publish rolling intake merge requests

Reuse GitLab repository-file reads, reconciled effects, branch creation, commit publication, and draft-MR creation. Use exact branch heads and last-commit checks so concurrent submissions retry without losing changes. Reconcile one bot note with accepted submissions and current graduation candidates.

- **Update:** `crates/acorn-lib/src/io/api/gitlab/automation.rs`, `effects.rs` only if a new effect kind is necessary, and `crates/acorn-lib/src/io/api/gitlab/tests/{mod,bot}.rs`.
- **Create:** None.
- **Delete:** None.
- **Lock:** GitLab automation and durable effect reconciliation.
- **Verify:** fake-GitLab tests cover first MR, additional submission, continuation MR, stale-head retry, duplicate submission, and partial-effect recovery.

## 7. Add explicit MR graduation commands

Extend the existing GitLab note operation with candidate listing and selected graduation. Reuse the human-author and Developer-access checks, `ResearchActivity::graduate`, and `Database::graduate_logbook`, then commit the returned activity to the same bot branch and reconcile its status note.

- **Update:** `crates/acorn-lib/src/io/api/gitlab/{webhook,automation}.rs`, `crates/acorn-lib/src/io/api/gitlab/tests/bot.rs`, and `crates/acorn-lib/src/io/database/tests/logbook.rs` only for additional transactional assertions.
- **Create:** None.
- **Delete:** None.
- **Lock:** GitLab note-command dispatch and bot-branch publication.
- **Verify:** listing, selected graduation, unknown IDs, insufficient access, bot-authored notes, retries, rollback, and immutable backup revisions pass.

## 8. Add bounded reporting-window selection

Keep `ResearchActivity::updates_since` compatible. Add an immutable reporting-window type with an optional lower bound and exclusive upper bound. Share filtering and ordering through implementations on the activity and reporting types without introducing transport concerns.

- **Update:** `crates/acorn-schema/src/research_activity/{logbook,mod}.rs`, `crates/acorn-schema/src/research_activity/tests/mod.rs`.
- **Create:** None unless the reporting-window type merits its own adjacent schema leaf.
- **Delete:** None.
- **Lock:** research-activity logbook schema.
- **Verify:** full history, `[from, through)` boundaries, equality, ordering, invalid bounds, timezones, and daylight-saving transitions pass.

## 9. Reuse the existing PDF renderer

Generalize the updates renderer only enough to accept bounded report data. Keep one HTML template, Chromium path, PDF writer, and visual style; retain ordinary `export --target updates` behavior.

- **Update:** `crates/acorn-cli/src/commands/export/updates.rs`, `pdf.rs` only if it needs a reusable associated render method, and `crates/acorn-cli/src/commands/export/tests/mod.rs`.
- **Create:** None.
- **Delete:** any duplicate rendering branch introduced during the extraction.
- **Lock:** export/PDF integration.
- **Verify:** existing update exports remain compatible and representative monthly PDFs pass snapshot and visual inspection.

## 10. Add GitLab report-package support

Add typed package operations through existing endpoint, authentication, request, response, and error contracts. `ReportManifest` owns deterministic naming, source commit, reporting bounds, selected entry identifiers, pipeline/job IDs, checksum, authenticated URL, version, and `published` or `no-changes` status.

- **Update:** `crates/acorn-lib/src/io/api/gitlab/mod.rs`, `crates/acorn-lib/assets/constants/application.json`, and focused GitLab tests.
- **Create:** `crates/acorn-lib/src/io/api/gitlab/package.rs` and package tests under `crates/acorn-lib/src/io/api/gitlab/tests/`.
- **Delete:** None.
- **Lock:** GitLab provider module root and embedded endpoint definitions.
- **Verify:** latest-manifest lookup, incomplete package versions, upload order, checksum conflict, retry, and no-change advancement pass.

## 11. Add the runner-facing report command

Add one thin command that composes configuration, bounded selection, the existing renderer, and GitLab package publication. The scheduled job supplies a fixed `through` instant and default-branch commit. Process projects independently, but fail the job after completing successful projects when any project fails. Enforce late-entry rejection by comparing intake timestamps with the latest valid report manifest.

- **Update:** `crates/acorn-cli/src/commands/mod.rs`, `crates/acorn-cli/src/cli/{mod,arguments/mod}.rs`, `io/api/powerautomate/form.rs`, and `CommandLineHelp.md`.
- **Create:** `crates/acorn-cli/src/commands/report/{mod,tests}.rs`.
- **Delete:** None.
- **Lock:** CLI contract, generated help, export dispatch, and GitLab package API.
- **Verify:** missing CI context, explicit local fixtures, multiple projects, first full-history report, late rejection, partial failure, and machine-readable results pass.

## 12. Implement signed outbound PowerAutomate events

Add a provider-neutral Standard Webhooks signer beside the existing verifier and let a callback destination apply it to the exact serialized body. Keep event selection, payloads, destination names, and callback policy inside `io::api::powerautomate`. The MVP emits `mr-created`, `review-needed`, `artifact-ready`, `report-no-changes`, and `failed`.

- **Update:** `crates/acorn-lib/src/io/api/webhooks/{mod,store}.rs` only as required by generic signing, `crates/acorn-lib/src/io/api/webhooks/tests.rs`, and `io/api/powerautomate/{mod,event,tests/}`.
- **Create:** `crates/acorn-lib/src/io/api/powerautomate/callback.rs`.
- **Delete:** None.
- **Lock:** provider-neutral callback transport and callback store.
- **Verify:** exact-body signature, retry, destination allowlist, idempotency, and secret-redaction tests pass.

## 13. Wire the scheduled GitLab runner job

Provide a reusable bucket-project CI template. Run only for scheduled default-branch pipelines, capture one `through` value, invoke `acorn report`, retain failure diagnostics, and consume protected variables.

- **Update:** `.gitlab-ci.yml`, `docs/src/SUMMARY.md`.
- **Create:** `templates/gitlab/reporting.gitlab-ci.yml`, `docs/src/automation/powerautomate-reporting.md`.
- **Delete:** None.
- **Lock:** CI root and documentation index.
- **Verify:** GitLab CI lint passes and a test bucket schedule publishes a package from a fixed commit.

## 14. Prove the MVP end to end

Exercise signed form intake, durable processing, draft MR creation, reviewer graduation, human merge, scheduled report generation, package publication, and signed callback delivery for one pilot project.

- **Update:** existing GitLab fake-server fixtures only for new MR/package responses and PDF visual fixtures only for the new report.
- **Create:** `tests/fixtures/automation/powerautomate/` and end-to-end tests under `io/api/powerautomate/tests/`.
- **Delete:** None.
- **Lock:** shared HTTP fixtures and expected PDF snapshots.
- **Verify:** the flow is repeatable, duplicate-safe, and produces a visually accepted PDF. This is the MVP completion gate.

## 15. Add post-MVP delivery tracking and graduation breadth

After the pilot, add signed PowerAutomate delivery receipts, a durable `delivered` state, retention cleanup, additional graduation targets, and remaining project mappings.

- **Update:** `io/api/powerautomate/{event,webhook,tests/}`, `crates/acorn-schema/src/research_activity/`, and `crates/acorn-lib/src/io/database/logbook.rs` only if added targets require persistence changes.
- **Create:** a database migration only if delivery status cannot reuse the operation/callback tables.
- **Delete:** None.
- **Lock:** PowerAutomate event contract, schema graduation model, and database migrations.
- **Verify:** delivery receipts are idempotent and each new graduation target has deterministic merge/conflict tests. This work must not delay the MVP.

# Validation

- Run focused PowerAutomate, webhook, GitLab, database, schema, workflow, CLI, and export tests after each atomic slice.
- Run `make lint`, `make test`, GitLab CI lint, and PDF render inspection before the MVP pilot.
- Verify invalid signatures, replay, duplicate delivery, duplicate submission, concurrent branch updates, partial GitLab effects, callback retry, incomplete packages, and job retry never duplicate a durable external effect.
- Confirm configuration and error diagnostics redact secrets and never persist caller-supplied callback targets or GitLab credentials.
- Pilot one configured project for a complete reporting period before enabling the remaining project-folder mappings.

# Acceptance criteria

- PowerAutomate communication behavior is isolated to `io::api::powerautomate`; only explicit module, configuration, router, and operation composition points reference it elsewhere.
- Existing generic webhook, queue, callback, GitLab, graduation, and PDF implementations are reused rather than forked.
- A signed form creates or updates the correct project's draft MR exactly once.
- An authorized reviewer can inspect and selectively graduate entries through MR comments, while the bot cannot merge or bypass approvals.
- A scheduled runner generates the correct full-history or bounded monthly PDF from merged default-branch RAD.
- PDF and final manifest are published immutably to GitLab packages; incomplete publication does not advance the cutoff.
- No-change periods advance the cutoff without producing an empty PDF.
- Entries older than the last successful report cutoff are rejected for correction rather than silently omitted.
- PowerAutomate receives a signed, retryable artifact, no-change, review-needed, or failure event without embedded credentials or PDF bytes.
- Teams and email delivery remain owned by PowerAutomate.
