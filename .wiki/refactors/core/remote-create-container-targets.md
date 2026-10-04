---
type: Plan
title: "Harden container creation on remote Docker hosts"
description: "Add preflight, rollback, cleanup, and live validation to SSH-backed remote container creation."
difficulty: high
impact: high
package: [acorn-cli, acorn-lib]
tags: [feature, cli, create, docker, ssh, reliability, rollback]
status: in-progress
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
