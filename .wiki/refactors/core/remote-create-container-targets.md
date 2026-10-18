---
type: Plan
title: "Harden container creation on remote Docker hosts"
description: "Add preflight, rollback, cleanup, and live validation to SSH-backed remote container creation."
difficulty: high
impact: high
package: [acorn-cli, acorn-lib]
tags: [feature, cli, create, docker, ssh, reliability, rollback]
status: in-progress
execution: phased
phase_budget_hours: 5
---

# Project code boundaries

- **Update:** `crates/acorn-cli/src/commands/create/`, CLI argument/execution models, Docker/SSH helpers, tests, and operations documentation.
- **Create:** explicit remote target, preflight, rollback journal, and fake-runner/live-test fixtures.
- **Delete:** disposable test containers/volumes on only the explicitly targeted host; remove legacy long-signature orchestration after typed requests migrate.

# Dependencies

Use the normalized request boundary from [CLI composition](cli-composition-boundary.md). Host process/network implementations ultimately belong in `acorn-host`, while target policy remains in the interface/service boundary.

# Parallel execution

Preflight, rollback, and fake-runner work confined to `commands/create/` can run with portability and non-CLI lanes. Any change to shared CLI parser types, startup, generated help, or host abstraction contracts is a separate integration slice and requires the corresponding lock.

# Goal

Harden bot and GitLab runner creation on an SSH-accessible Docker host with side-effect-free preflight, ownership-aware cleanup, per-runner rollback, and live validation without changing the user's active Docker context:

```text
acorn create bot <PROJECT_ID> --host ssh://some-domain
acorn create runner --group <GROUP_ID> --description <NAME> --host ssh://user@some-domain
```

Bot and runner container creation run detached. The host option selects the Docker daemon; it does not change container lifecycle behavior.

# Current evidence

- Remote bot and runner creation already target Docker through per-command `--host` arguments without changing Docker context or process-wide environment.
- Configured runners can declare `target.host`; `acorn create runner` discovers the configuration, resolves every target/runtime pair before GitLab API activity, and lets an explicit CLI host or context override configured targets.
- Runner container creation is always detached, so config-only creation can continue to the registration step without a compatibility `--detach` flag.
- The runner workflow can acquire a GitLab runner, remote container, configuration volume, and temporary GPU template without a single ownership record or compensating rollback boundary.
- No disposable SSH Docker endpoint is available in the development environment, so remote daemon behavior and cleanup guarantees still lack live validation.

# Remaining work

Harden the workflow before live validation:

1. Add a side-effect-free preflight phase before the GitLab create-runner request:
   - Verify that the local Docker client exists.
   - Verify SSH connectivity, remote Docker socket access, and daemon responsiveness with the selected `--host` endpoint.
   - Check required GPU capability when GPU execution is requested.
   - Check the requested bot container name for a conflict.
   - Return preflight failures without creating a GitLab runner, container, or volume.
2. After the GitLab API assigns a runner identifier but before Docker mutation, check the derived runner container name and configuration volume for conflicts. Track the resources acquired by each runner attempt, including the GitLab runner identifier, remote container name, configuration volume, and temporary GPU template.
3. Add per-runner compensating rollback when any step after GitLab runner creation fails:
   - Remove a container created by the current attempt.
   - Delete the GitLab runner created by the current attempt.
   - Preserve the original failure as the primary error and append any rollback failures instead of masking it.
   - Keep rollback isolated per runner when configuration-driven runners are created concurrently.
4. Add ownership-aware automatic cleanup:
   - Always attempt to remove temporary GPU-template files created by the invocation.
   - Remove failed-attempt containers only when their identity matches the resource created by the invocation.
   - Remove a named volume only when ACORN proved before creation that it did not already exist and the current attempt created it.
   - Never automatically delete a user-supplied bot state volume or any pre-existing container or volume.
   - Report retained resources and explicit cleanup commands when safe automatic cleanup is not possible.
5. Add failure-path tests for preflight rejection, failure after each acquired resource, rollback failure reporting, concurrent per-runner isolation, and preservation of pre-existing or user-supplied resources.

Then perform a manual smoke test against a disposable SSH-accessible Docker host:

1. Create a bot remotely and confirm it is absent from local `docker ps` and present in `docker --host <URI> ps`.
2. Verify its published port, restart policy, and named state volume on the remote host.
3. Create and register a runner remotely, then verify its configuration volume and Docker socket mount.
4. Repeat at least one operation with a username and non-default SSH port; if a custom socket path is supported by the disposable host, exercise that form too.
5. Exercise GPU configuration and verify the copied in-container template and successful registration command.
6. Force a failure after GitLab runner creation and confirm that the runner and current-attempt container are rolled back without deleting pre-existing resources.
7. Force cleanup failures and confirm the original error remains primary while retained resource identifiers and cleanup instructions are reported.
8. Remove any remaining disposable containers and volumes from the explicitly targeted remote host.

# Acceptance criteria

1. Remote daemon connectivity and required capabilities are checked before the GitLab create-runner API call, and preflight failure leaves no new GitLab or Docker resources.
2. A failure after GitLab runner creation triggers per-runner rollback of resources created by that attempt; batch rollback does not affect successful or unrelated runners.
3. The original operation failure remains primary when rollback or cleanup also fails, and the error reports every resource that may require manual cleanup.
4. Automatic cleanup never removes a pre-existing container or volume, and never removes a user-supplied bot state volume.
5. A runner with `target.host` can be created through configuration discovery with only `acorn create runner`; invalid SSH targets and incompatible executors fail before GitLab API activity, while CLI targets override configured targets.

# Validation

- Configuration tests cover string and object target forms, SSH alias normalization, invalid targets, unknown target keys, serialization, and runner snapshots.
- Runner command tests cover configured-target selection, CLI override precedence, runtime compatibility, detached container creation, remote volumes, and remote registration.
- Verified with `cargo clippy -p acorn-lib -p acorn-cli --all-targets --no-deps -- -D warnings`; focused configuration, snapshot, CLI target, and runner tests pass. A full `make test` rerun stopped after 746 passes on the unrelated missing OpenAPI import snapshot baseline on 2026-09-19; the previous full run passed 1,796 tests with 14 skipped.

# Related plans

- [CLI composition boundary](cli-composition-boundary.md)
- [CLI coverage](../../testing/cli-coverage.md)

# Execution phases

## RCC-01 — Model attempt ownership

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-cli/src/commands/create/runner/mod.rs`, `crates/acorn-cli/src/commands/create/runner/tests.rs`
- Primary integration lock: create-runner workflow
- Outcome: Track the runner ID, container, volume, and temporary template acquired by each attempt.
- Focused validation: Test empty, partial, complete, and concurrent ownership records.
- Handoff: Remove RCC-01 and use the record in preflight and rollback.

## RCC-02 — Preflight Docker and SSH connectivity

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RCC-01
- Exact write set: `crates/acorn-cli/src/commands/create/runner/mod.rs`, `crates/acorn-cli/src/commands/create/runner/tests.rs`
- Primary integration lock: create-runner workflow
- Outcome: Check local client, SSH target, remote socket, and daemon before GitLab mutation.
- Focused validation: Test missing client, unreachable host, invalid socket, and responsive daemon with no acquired resources.
- Handoff: Remove RCC-02 and expose a side-effect-free preflight result.

## RCC-03 — Preflight capabilities and names

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RCC-02
- Exact write set: `crates/acorn-cli/src/commands/create/runner/mod.rs`, `crates/acorn-cli/src/commands/create/runner/tests.rs`
- Primary integration lock: create-runner workflow
- Outcome: Reject missing GPU capability and conflicting bot/runner names before mutation.
- Focused validation: Test CPU/GPU targets, name conflicts, and CLI-over-configuration precedence.
- Handoff: Remove RCC-03 and freeze preflight diagnostics.

## RCC-04 — Record post-registration resource conflicts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RCC-01, RCC-03
- Exact write set: `crates/acorn-cli/src/commands/create/runner/mod.rs`, `crates/acorn-cli/src/commands/create/runner/tests.rs`
- Primary integration lock: create-runner workflow
- Outcome: Check derived container/volume conflicts and record each successfully acquired resource.
- Focused validation: Inject failures after every acquisition boundary.
- Handoff: Remove RCC-04 and pass ownership records to rollback.

## RCC-05 — Add compensating runner/container rollback

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RCC-04
- Exact write set: `crates/acorn-cli/src/commands/create/runner/mod.rs`, `crates/acorn-cli/src/commands/create/runner/tests.rs`
- Primary integration lock: create-runner workflow
- Outcome: Remove only the current-attempt container and GitLab runner while preserving the original failure.
- Focused validation: Test each failure point, rollback failures, and concurrent attempt isolation.
- Handoff: Remove RCC-05 and record retained-resource diagnostics.

## RCC-06 — Add ownership-aware volume and template cleanup

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RCC-04
- Exact write set: `crates/acorn-cli/src/commands/create/runner/mod.rs`, `crates/acorn-cli/src/commands/create/runner/tests.rs`
- Primary integration lock: create-runner workflow
- Outcome: Remove temporary templates and newly created volumes without deleting pre-existing or user-supplied state.
- Focused validation: Test created/pre-existing/user-supplied resources and cleanup failure reporting.
- Handoff: Remove RCC-06 and freeze cleanup-command guidance.

## RCC-07 — Complete the failure-path matrix

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RCC-05, RCC-06
- Exact write set: `crates/acorn-cli/src/commands/create/runner/tests.rs`, `crates/acorn-cli/src/commands/create/tests/mod.rs`
- Primary integration lock: create-runner workflow
- Outcome: Cover preflight rejection, every acquisition failure, rollback aggregation, and batch isolation.
- Focused validation: Run focused configuration, snapshot, target, runner, and failure tests.
- Handoff: Remove RCC-07 and publish the disposable-host test script.

## RCC-08 — Smoke-test remote bot creation

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: RCC-07 and a disposable SSH Docker host
- Exact write set: disposable remote Docker host, `docs/src/cli-reference/remote-container-validation.md`
- Primary integration lock: remote Docker validation
- Outcome: Create a bot remotely and verify ports, restart policy, state volume, username, port, and host isolation.
- Focused validation: Compare local and remote `docker ps` plus inspected remote resources.
- Handoff: Remove RCC-08 and retain the host/daemon evidence.

## RCC-09 — Smoke-test runner rollback and GPU setup

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RCC-08
- Exact write set: disposable GitLab runner and remote Docker host, `docs/src/cli-reference/remote-container-validation.md`
- Primary integration lock: remote Docker validation
- Outcome: Register a remote runner, exercise GPU/template behavior, forced rollback, and cleanup failures safely.
- Focused validation: Inspect GitLab registration, mounts, retained resources, original errors, and manual cleanup instructions.
- Handoff: Remove RCC-09 and clean every disposable resource explicitly.

## RCC-10 — Document and retire remote creation hardening

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: RCC-09
- Exact write set: `docs/src/cli-reference/commands/create.md`, `.wiki/index.md`, `.wiki/refactors/core/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/refactors/core/remote-create-container-targets.md`
- Primary integration lock: container documentation
- Outcome: Publish preflight/rollback/cleanup guarantees, run final gates, and remove the plan.
- Focused validation: Run focused tests, clippy, `make test`, docs, and wiki validation.
- Handoff: Delete this plan and every index, network, and lane entry.
