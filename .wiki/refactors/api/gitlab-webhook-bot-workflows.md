---
type: Plan
title: "Generalize GitLab repository automation and finish citation workflows"
description: "Use shared JSON-RPC and webhook services to ingest, process, and publish typed GitLab repository files."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, api, gitlab, bot, webhook, json-rpc, workflow, repository, flat-file, citation]
status: in-progress
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/gitlab/{bot,intake,mod,review,service,webhook}.rs`, `src/io/workflow/mod.rs`, `src/analyzer/service.rs`, embedded GitLab endpoint metadata, bot command modules in `crates/acorn-cli`, fixtures, and mocked API/store tests.
- **Create:** `crates/acorn-lib/src/io/api/gitlab/{adapter,automation}.rs` for typed provider operations and GitLab-specific repository automation; `crates/acorn-lib/src/io/workflow/{registry,repository}.rs` for registered workflow selection and provider-neutral repository requests.
- **Delete:** `crates/acorn-lib/src/io/api/gitlab/database.rs` after its lifecycle behavior moves to `io/database/webhooks.rs`; remove GitLab-local delivery, retry, worker, operation-dispatch, citation-only routing, and the existing bot queue tables at cutover because the bot has no compatibility obligation or current users.
- **Do not change:** schema parsers and renderers remain owned by their existing modules; GitLab authentication, REST/GraphQL wire DTOs, and pagination remain in the provider adapter; JSON-RPC framing remains in `io::api::json_rpc`.

# Dependencies

This plan is third in the [highest-priority RPC and automation lane](../../parallel-execution.md#highest-priority-lane-execution-order). It consumes the [JSON-RPC operation runtime](json-rpc-runtime.md) and [extensible webhook runtime](webhook-runtime.md). The [Protobuf/gRPC transport](protobuf-grpc-transport.md) is a later sibling adapter and is not required for the GitLab vertical slice.

# Goal

Turn the GitLab bot into a general repository automation service. Authenticated webhook events and direct commands select registered workflows that can read repository files at an exact commit, parse and process supported flat-file formats, and publish reviewed changes through bot branches and draft merge requests. `CITATION.cff` remains one workflow and one output; it no longer defines the bot architecture.

# Current evidence

- `io::api::webhooks` already owns verifier, delivery, handler, and safe callback primitives, but GitLab still owns Axum ingress, provider decode, durable queueing, retry, and worker orchestration.
- `io::api::gitlab` already accepts merge-request and note hooks, persists `bot_webhook_deliveries` and `bot_operations`, analyzes merge-request heads, posts report notes, and creates commit statuses.
- `analyzer::service` already has structured checks for CFF, RAD JSON/JSONC/YAML, text, and DOCX. Existing discovery and schema code should be composed through typed workflows instead of copied into the bot.
- The GitLab client can read merge-request details, diffs, repository files at an exact SHA, work items, notes, membership, hook state, and commit statuses. It still needs typed branch, commit, merge-request, label, and work-item mutation operations.
- `acorn check --merge-request` assumes a local checkout. A detached bot must fetch the selected files at the merge-request head SHA and reject stale publication if the source head changes.
- The current database tables encode GitLab and bot terminology. They should be replaced by the provider-neutral delivery, operation, effect, and callback schema defined by the webhook plan rather than maintained as a compatibility layer.

# Decisions

- Expose registered ACORN workflows, never an arbitrary GitLab API proxy. The initial operation names are `acorn.webhooks.gitlab.merge-request`, `acorn.webhooks.gitlab.note`, `acorn.workflows.process`, `acorn.operations.status`, and `acorn.operations.replay`.
- Use one deployment-owned YAML configuration to map events, commands, paths, formats, and allowed effects to workflow IDs. Repository content cannot expand server permissions.
- Support CFF, RAD/logbook JSON and JSONC, YAML, and Markdown in the first registry. Each format adapter owns typed parse, validation, normalization, and rendering behavior.
- The initial workflow actions are `validate`, `format`, `enrich`, and `link`. Each workflow declares its accepted formats and effects.
- Trigger workflows from merge-request activity and authorized slash commands. Retain `/acorn check`; add `/acorn process [workflow]` for explicit selection.
- Operate in the configured GitLab project first. Cross-project and fork write-back require a later policy decision.
- Read every source file at a recorded commit SHA. Write only through a bot-owned branch and draft merge request, with compare-and-swap checks before publication.
- A workflow may rewrite a source flat file and create derived artifacts. Every output must be declared in the workflow result and shown in the merge-request description.
- When processing a human merge request, publish corrections in a stacked bot merge request rather than mutating the contributor's branch.
- Require at least GitLab Developer access for mutating commands, retain an immutable actor/project/source-SHA audit record, and suppress events produced by the bot's own branches, notes, and merge requests.
- Keep outbound GitLab callbacks durable and idempotent. A retry may repeat a network request, so each effect uses a stable idempotency key and reconciles existing GitLab objects before creating new ones.

# Configuration contract

```yaml
workflows:
  repository-quality:
    paths:
      - "**/*.cff"
      - "**/*.{json,jsonc,yaml,yml,md}"
    formats: [cff, rad, json, jsonc, yaml, markdown]
    actions: [validate, format, enrich, link]
    outputs:
      mode: source-and-artifacts
    effects: [commit, draft-merge-request, note, commit-status]
```

Configuration is validated at startup. Unknown workflow IDs, formats, actions, effects, and unsafe paths are errors rather than ignored extensions.

# Repository workflow contract

```rust
/// A repository snapshot selected for workflow execution.
pub struct RepositoryInput {
    pub project: ProjectRef,
    pub revision: CommitSha,
    pub paths: Vec<SafePath>,
}

/// Declarative output from a registered repository workflow.
pub struct RepositoryChangeSet {
    pub base_revision: CommitSha,
    pub files: Vec<FileChange>,
    pub report: WorkflowReport,
}
```

The service executes a repository workflow in this order:

1. Normalize the webhook or direct request into a registered JSON-RPC operation.
2. Authorize the actor, project, workflow, requested paths, and effects.
3. Resolve the immutable source SHA and fetch only declared repository files.
4. Detect the format and deserialize through its existing typed entry point; use `eserde` for user-facing JSON where the schema already supports multi-error decoding.
5. Run pure validation and transformation steps before any external effect.
6. Produce a deterministic `RepositoryChangeSet` and effect plan.
7. Recheck the source head and reject stale writes.
8. Create or update the bot branch, commit, draft merge request, report note, and commit status through typed GitLab adapter methods.
9. Persist every effect result and enqueue durable callbacks before acknowledging completion.
10. On replay, reconcile the existing branch, merge request, note, and status rather than creating duplicates.

# Citation workflow

The existing citation intake becomes a registered workflow built on the same contracts:

- Accept candidates from work-item content and repository CFF files.
- Validate title and author data independently for each candidate.
- Write accepted records to `citations/work-items/<work-item-iid>/<creator-username>-<identifier-kind>-<identifier-hash>/CITATION.cff`.
- List incomplete candidates and missing fields in the work-item report and draft merge-request description without committing invalid CFF.
- Create or update one bot branch and draft citation merge request per work item; corrected candidates are promoted on rerun.
- When the bot-created merge request merges, apply `acorn::collected` and post its link exactly once while leaving the source work item open.

# Implementation order

| Order | Work | Verification gate |
|---:|---|---|
| 1 | Replace GitLab-local dispatch and bot tables with the shared webhook store, worker, JSON-RPC operation registry, and typed GitLab event adapter. | Existing merge-request and note-hook fixtures normalize to deterministic operations; duplicate receipt and restart converge on one operation. |
| 2 | Add the workflow and format registries plus immutable repository reads. | Fixture repositories exercise CFF, RAD, JSON/JSONC, YAML, and Markdown selection at an exact SHA without network effects. |
| 3 | Add typed branch, commit, merge-request, note, label, and status effects with bot-branch publication and stale-head protection. | Mock GitLab tests cover success, permission denial, stale heads, retry reconciliation, feedback-loop suppression, and stacked merge requests. |
| 4 | Port citation intake and merge completion to the generic workflow and finish its remaining repository writes. | Mixed valid/incomplete intake, zero-valid intake, correction promotion, stable slugs, and idempotent completion pass. |
| 5 | Add `/state`, liveness/readiness, graceful draining, deployment configuration, and staging documentation. | Container smoke, staging webhook resend, focused tests, generated help, formatting, and link checks pass. |

# Plan conflicts and resolutions

| Conflict | Resolution |
|---|---|
| GitLab currently owns HTTP ingress, queueing, retry, and workers. | Move lifecycle ownership to `io::api::webhooks`; retain only GitLab signature/token verification, payload decoding, and API calls in the adapter. |
| Existing bot tables contain useful data but encode the wrong boundary. | Replace them at cutover. There are no current users and no backward-compatibility requirement; tests verify clean initialization rather than migration. |
| `CITATION.cff` currently drives route and state design. | Register citation as one workflow. No generic service type may contain citation-specific fields. |
| ACP, MCP, webhooks, and gRPC also need callable operations. | Share the JSON-RPC operation registry and typed service handlers. Transport-specific sessions, authorization, streaming, and errors remain in their adapters. |
| Repository parsing is spread across analyzer, schema, and I/O modules. | Format adapters compose existing entry points; they do not duplicate schemas or move parsing into GitLab code. |
| A merge-request bot could overwrite contributor work. | Use immutable reads and stacked bot branches, then fail stale publications before any commit. |
| Generic API access could turn the bot into a privileged proxy. | Expose an allowlisted workflow/effect vocabulary and typed GitLab methods only. |
| Webhook acknowledgement is fast but GitLab callbacks can fail later. | Persist callback intents transactionally and retry them at least once with stable idempotency keys. |

# Validation

1. Contract-test GitLab merge-request and note fixtures through verification, normalization, JSON-RPC dispatch, persistence, and handler selection.
2. Run the same workflow handler directly and through JSON-RPC; assert identical typed results and error codes.
3. Process CFF, RAD, JSON/JSONC, YAML, and Markdown fixture repositories at fixed SHAs; assert deterministic ordering and byte-stable output.
4. Exercise duplicate deliveries, slash-command replay, worker restart, stale claims, callback failure, and process restart; verify one logical operation and reconciled GitLab effects.
5. Verify unauthorized users, undeclared paths, unknown workflows, disallowed effects, oversized bodies, invalid signatures, stale source heads, and bot-authored events fail without repository mutation.
6. Verify human merge-request corrections use a stacked draft merge request and never push to the source branch.
7. Verify citation mixed intake, zero-valid intake, correction promotion, merge completion, and loop suppression.
8. Run focused Cargo tests, applicable feature checks, `make help`, formatting, wiki link validation, and `git diff --check`.

# Acceptance criteria

- GitLab events and direct requests invoke only registered workflows through the shared operation registry.
- The bot ingests, validates, processes, and writes supported repository flat files; CFF is one supported format rather than the output boundary.
- Every repository read is pinned to an exact SHA and every write uses a bot-owned branch and draft merge request with stale-head protection.
- Duplicate delivery, replay, polling overlap, retry, and restart cannot duplicate an operation, branch, merge request, note, label, status, or completion comment.
- Provider-neutral webhook tables and workers replace the GitLab-local lifecycle without a compatibility migration.
- Citation intake retains deterministic paths, exclusion reporting, correction promotion, and idempotent `acorn::collected` completion.
- Health, readiness, queue state, failures, last success/error, and deduplication counts are observable.

# Next plan

After this plan's acceptance criteria pass, continue with the [Protobuf/gRPC transport](protobuf-grpc-transport.md). Its equivalence suite should use the operation and policy contracts proven by the GitLab vertical slice.

# Citations

[1] [GitLab webhook events](https://docs.gitlab.com/user/project/integrations/webhook_events/) - Merge-request, comment, and work-item payload behavior.

[2] [GitLab project webhooks API](https://docs.gitlab.com/api/project_webhooks/) - Hook registration, history, tests, and resend operations.

[3] [GitLab repository files API](https://docs.gitlab.com/api/repository_files/) - Exact-ref reads and repository file mutations.

[4] [GitLab commits API](https://docs.gitlab.com/api/commits/) - Atomic multi-file repository commits and commit statuses.

[5] [GitLab merge requests API](https://docs.gitlab.com/api/merge_requests/) - Draft merge-request creation, lookup, and update behavior.

[6] [GitLab project members API](https://docs.gitlab.com/api/project_members/) - Effective project membership and access levels.
