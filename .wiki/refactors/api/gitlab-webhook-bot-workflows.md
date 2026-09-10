---
type: Plan
title: "Finish GitLab automation effects and citation lifecycle"
description: "Complete effect reconciliation, citation correction and merge handling, deployment configuration, and operational readiness."
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, api, gitlab, bot, webhook, workflow, citation, operations]
status: in-progress
priority: next
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/gitlab/{automation,bot,intake,mod,webhook}.rs`, provider-neutral webhook effect/state persistence, embedded GitLab endpoint metadata, bot composition roots, deployment documentation, fixtures, and mocked API/store tests.
- **Create:** Deployment-owned workflow configuration and typed label, status, completion-comment, and effect-reconciliation helpers where the existing typed GitLab adapter cannot express them.
- **Delete:** Citation-specific lifecycle state superseded by provider-neutral effect records, if any remains after reconciliation is implemented.
- **Do not change:** `io::api::json_rpc` framing, the provider-neutral repository workflow contracts, or the completed exact-SHA branch/commit/draft-MR publication path.

# Dependencies

This is second in the [highest-priority RPC and automation lane](../../parallel-execution.md#highest-priority-lane-execution-order). It follows the remaining readiness and graceful-drain work in the [webhook runtime](webhook-runtime.md). The implemented JSON-RPC registry, webhook lifecycle, flat-file workflow registry, and GitLab publication path are foundations rather than open plan scope.

# Goal

Finish the operational edges of GitLab repository automation: deployment-controlled workflow policy, durable reconciliation for every external effect, corrected citation promotion, idempotent merge-completion feedback, and shared readiness/drain integration.

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

Validate configuration at startup. Unknown workflow IDs, formats, actions, effects, and unsafe paths are errors. Repository content cannot expand deployment permissions. Cross-project or fork writes remain disabled until configuration names an explicit destination and policy.

# Remaining citation lifecycle

- Promote a previously incomplete candidate when corrected work-item content becomes valid, updating the existing bot branch and draft citation merge request rather than creating another logical operation.
- When a bot-created citation merge request merges, apply `acorn::collected` and post its link exactly once while leaving the source work item open.
- Persist label, status, and completion-comment effects under stable idempotency keys and reconcile existing GitLab objects before retrying creation.
- Suppress webhook loops caused by the bot's own labels, comments, branches, and merge requests.

# Implementation order

| Order | Work | Verification gate |
|---:|---|---|
| 1 | Add deployment workflow configuration, explicit cross-project denial/policy, per-effect persistence, and typed reconciliation for labels, statuses, and comments. | Startup rejects invalid policy; retry tests converge on one external effect after ambiguous failures. |
| 2 | Add citation correction promotion, merge-completion handling, and shared readiness/drain exposure. | Corrected candidates update one draft MR; merge processing applies one label and one comment; readiness and shutdown use webhook runtime state. |

# Plan conflicts and resolutions

| Conflict | Resolution |
|---|---|
| API health work and webhook runtime both expose readiness. | Consume the shared webhook state; GitLab adds provider details without defining another lifecycle model. |
| Citation retries can duplicate labels, comments, statuses, or merge requests. | Persist a stable effect identity and reconcile existing GitLab objects before each mutation. |
| Cross-project writes require broader credentials and trust policy. | Fail closed until deployment configuration explicitly names the destination and allowed effect set. |
| Repository configuration could attempt to widen bot authority. | Deployment configuration is authoritative; repository content may only narrow the allowed workflow. |

# Validation

1. Exercise ambiguous failures and retries for branch, commit, merge request, label, status, and completion-comment effects; assert one logical external effect.
2. Verify invalid deployment configuration, undeclared paths, unknown workflows, disallowed effects, fork sources, and unauthorized destinations fail without repository mutation.
3. Verify a corrected citation candidate is promoted on the existing bot branch and draft merge request.
4. Verify merge completion applies `acorn::collected` and posts the merge-request link exactly once while leaving the source work item open.
5. Verify bot-authored completion events do not enqueue feedback-loop operations.
6. Verify readiness and bounded shutdown follow the shared webhook runtime state.
7. Run focused GitLab and webhook tests, applicable feature checks, generated help, formatting, wiki link validation, and `git diff --check`.

# Acceptance criteria

- [ ] Branch, commit, merge-request, label, status, and completion-comment effects reconcile idempotently after retries and ambiguous failures.
- [ ] Corrected citation candidates update the existing draft merge request, and merge completion applies `acorn::collected` plus one link comment.
- [ ] Deployment configuration constrains workflows and effects; cross-project and fork writes fail closed unless explicitly authorized.
- [ ] GitLab readiness and shutdown expose the shared webhook runtime state without a second lifecycle implementation.

# Next plan

After this plan's acceptance criteria pass, continue with the [Protobuf/gRPC transport](protobuf-grpc-transport.md). Its equivalence suite should use the operation and policy contracts proven by the implemented GitLab vertical slice.

# Citations

- [GitLab commits API](https://docs.gitlab.com/api/commits/) - Atomic commits and commit-status reconciliation.
- [GitLab merge requests API](https://docs.gitlab.com/api/merge_requests/) - Draft merge-request lookup and update behavior.
- [GitLab notes API](https://docs.gitlab.com/api/notes/) - Work-item and merge-request completion comments.
- [GitLab project members API](https://docs.gitlab.com/api/project_members/) - Effective project access levels.
