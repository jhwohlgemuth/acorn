---
type: Plan
title: "Capability-brokered Rhai workflow hooks"
description: "Evaluate typed, explicitly trusted Rhai workflow hooks through declared capabilities and transactional ACORN boundaries."
difficulty: high
impact: critical
package: [workspace, acorn-lib, acorn-cli]
tags: [feature, rhai, scripting, workflow, capabilities, security]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/script/{mod,result}.rs`, `crates/acorn-lib/src/io/workflow/{mod,repository}.rs`, workspace and `acorn-lib` manifests, focused CLI configuration, dedicated tests, documentation, and generated help. Transport and TUI files remain outside the first implementation slice.
- **Create:** `crates/acorn-lib/src/script/{capability,manifest,hooks}.rs` plus dedicated hook policy and lifecycle fixtures. Reconcile these paths with the implemented script and workflow layout before work begins.
- **Delete:** None.

# Dependencies

This plan is not an implementation candidate until all of the following are true:

1. [Configured Rhai exec](rhai-exec.md) has landed and real use demonstrates a need beyond configuration-authorized, pure workflow quality gates.
2. Workflow plan/apply and analyzer result boundaries are stable.
3. Operation effects can express filesystem-read, filesystem-write, network-read, process, credentials, inference, and mutation separately.
4. A security review chooses in-process capability brokering or process isolation for effectful and untrusted execution.
5. The highest-priority RPC and repository automation lane releases any shared integration locks.

# Goal

Permit a small set of explicitly configured Rhai hooks to propose typed workflow changes without exposing raw ACORN modules, ambient host authority, or generic pipeline replacement.

# Current evidence

- Repository workflows already separate typed planning from application and deployment automation uses allowlisted actions and effects.
- `SafePath` is a useful output-path validator but is not a complete filesystem sandbox or capability broker.
- JSON-RPC's mutation flag does not express the finer-grained authority required by arbitrary scripts.
- CLI, TUI, workflow service, and repository automation do not all share one automatic hook path today.

# Decisions required before implementation

- Define a deployment-owned manifest containing script ID, ABI version, entry point, content digest, allowed hooks, capabilities, resource limits, and approved input/output roots.
- Project configuration may authorize the pure quality gates defined by the prerequisite plan. Effectful automation additionally requires a deployment-owned allowlist entry whose digest matches; repository or XDG discovery alone never establishes that authority.
- Define every hook with dedicated input and output DTOs. Reuse `ExecutionResult<T>` only with a named output type; do not use a generic `Map`, `Value`, `false`, or exceptions for skip/replace/fail control flow.
- Hooks may propose immutable changes during planning. Existing validators run over the complete proposal before the host applies any effect.
- Specify hook ordering, conflict resolution, error policy, idempotency, rollback, cancellation, re-entrancy, state lifetime, compile caching, provenance, and audit records.
- Grant effects through narrow host functions backed by an invocation capability object. Never inject raw `acorn::io`, analyzer traversal, environment, process, keyring, cache, archive, database, or provider modules.
- Filesystem access, if approved, uses distinct read and write roots and a directory-capability design that addresses symlink and time-of-check/time-of-use behavior. The invocation directory is not an automatic grant.
- Remote `script.execute`, MCP, JSON-RPC, webhook, GitLab, and TUI integration remain separate final slices after local typed hooks and policy tests pass.

# Implementation sequence

1. Split `OperationEffects` before adding any effectful script hook. Replace coarse read-only/open-world/mutation flags with explicit filesystem-read, filesystem-write, network-read, process, credentials, inference, mutation, and destructive effects. Filesystem-write, mutation, and destructive effects require mutation approval. Network-read is denied in offline mode. Process, credentials, and inference remain unavailable until a later policy explicitly enables them.
2. Implement filesystem-read and filesystem-write as the first effectful Rhai hook slice. Add typed capabilities and host-brokered functions such as `read_text(path)`, `exists(path)`, and `write_text(path, content)`. Read operations are limited to declared read roots. Write operations are limited to declared write roots and propose host-owned file changes instead of writing directly.
3. Add network-read immediately after the `OperationEffects` split and its offline/allowlist tests pass. Network access must use an allowlist, respect offline mode, enforce response-size and timeout bounds, avoid credential forwarding, and reject local/private/link-local targets unless a later policy explicitly allows them.
4. Keep the first implementation local-CLI only. Do not expose remote JSON-RPC, MCP, webhook, GitLab, or TUI script execution in this slice.
5. Add local validation commands for manifests and hooks after the filesystem hook path exists, such as `acorn script manifest check <MANIFEST>` and `acorn script hook check <SCRIPT> --manifest <MANIFEST> --hook <HOOK> --input <FILE>`.

# Validation

- Threat-model untrusted repositories, merge requests, local configuration, symlink races, SSRF, credential access, process execution, cache leakage, recursive hooks, replay, and partial application.
- Contract-test every hook DTO, explicit `OperationEffects` policy, filesystem-read denial, filesystem-write denial, manifest/digest mismatch, path traversal and symlink escape denial, deterministic order, conflicting proposals, validator rejection before writes, cancellation, timeout, audit provenance, rollback, and absence of partial effects.
- Before enabling network-read hooks, prove offline denial, network allowlist denial and allow, response-size bounds, timeout behavior, redirect policy, credential isolation, and localhost/private/link-local rejection.
- Prove ordinary CLI, TUI, RPC, webhook, and GitLab operations do not execute unconfigured scripts.
- Run per-capability feature checks, policy tests, lint, full tests, audit, and portable graph checks before exposing any transport integration.

# Acceptance criteria

- Every implemented hook has a documented, versioned input type and typed `ExecutionResult<T>` output.
- Hooks propose changes through workflow planning; only the host applies a fully validated proposal.
- No script executes because it merely exists in a repository or user configuration directory.
- Every effect is authorized by a deployment-owned manifest and invocation capability; unspecified effects are unavailable.
- Script identity, digest, capabilities, inputs, outputs, findings, and application result are auditable without persisting secrets.
- Remote or automated execution is absent until its transport-specific policy and negative tests pass.
- The final implementation write set and integration locks are made exact before work begins.
