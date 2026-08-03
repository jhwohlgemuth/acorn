---
type: Plan
title: "Create containers on remote Docker hosts"
description: "Add and harden an SSH-backed --remote target for container-producing create commands."
difficulty: high
impact: high
package: [acorn-cli, acorn-lib]
tags: [feature, cli, create, docker, ssh, reliability, rollback]
status: in-progress
---

# Goal

Allow ACORN to create bot and GitLab runner containers on an SSH-accessible Docker host without changing the user's active Docker context:

```text
acorn create bot <PROJECT_ID> --remote ssh://some-domain
acorn create runner --group <GROUP_ID> --description <NAME> --remote ssh://user@some-domain
```

Bot creation already runs detached. The remote option selects the Docker daemon; it does not change container lifecycle behavior.

# Current implementation

Implemented on 2026-07-26:

- `acorn-lib/src/io/mod.rs` defines a validated `Remote` target backed by `Location`. It accepts `ssh://[user@]host[:port][/socket]`, rejects embedded passwords and unsupported URI components, adds Docker `--host` arguments, creates GPU templates, and copies templates through the selected `Executor`.
- `Executor` exposes `is_available` and Strum's `is_docker` predicate. Remote creation rejects non-Docker runtimes and non-Docker runner configurations before the GitLab create-runner request.
- `acorn-cli/src/cli/arguments.rs` exposes `--remote <URI>` on `create bot` and `create runner`.
- Bot creation targets the selected daemon while preserving its existing detached mode, ports, restart policy, environment, naming, and volumes.
- Runner creation targets its `run`, `exec`, and GPU-template `cp` operations consistently through `Remote` methods. Remote runners use a named configuration volume, the remote Docker socket, and an in-container GPU template copied through the selected daemon.
- Local bot and runner behavior remains unchanged when `--remote` is absent.
- `CommandLineHelp.md` and the embedded ACORN skill document local Docker and SSH prerequisites and explain that ports and volumes belong to the remote host.
- `serve bot --detach` remains outside this plan's scope.

# Assessment

Assessment updated on 2026-08-03:

- The endpoint-scoping design is sound: bot `run` and runner `run`, GPU-template `cp`, and registration `exec` arguments are built with Docker's per-command `--host` option. The implementation does not mutate Docker context or process-wide environment.
- The plan is correctly `in-progress`. Static inspection and focused tests support the argument-construction and validation claims, but acceptance criteria 1-3 still require a real SSH-accessible Docker daemon.
- The implementation is a medium-to-large cross-cutting feature rather than a narrow refactor. Commit `26ad4f41` changed 16 files with 572 insertions and 156 deletions, including CLI parsing and dispatch, library types and command construction, runner workflow changes, tests, generated help, embedded guidance, and this plan.
- The original `medium` difficulty understated the integration risk across local CLI execution, SSH, the remote Docker daemon, and GitLab runner registration, so the estimate is now `high`.
- Remaining work is approximately 4-7 engineer-days: 0.5-1.5 days for preflight, 2-3 days for per-runner rollback, 1-2 days for ownership-aware cleanup, and the balance for live smoke testing and evidence capture. The work can be smaller if existing GitLab deletion APIs are directly reusable.

# Validation

Automated validation completed on 2026-07-26 and reassessed on 2026-08-03:

- CLI and library tests cover valid SSH targets with usernames, ports, and socket paths and reject invalid schemes, missing hosts, whitespace, queries, fragments, out-of-range ports, and embedded passwords.
- Bot tests verify explicit remote targeting, retained detached behavior, and rejection of a non-Docker runtime before container command execution.
- Runner tests verify remote mounts, registration `exec` targeting, unsupported runtime and executor rejection, and unchanged local arguments. A library test verifies GPU-template `cp` targeting through the shared `Remote::docker_args` transformation.
- Three focused `acorn-lib` Remote tests and nine focused `acorn-cli` parsing and create-command tests cover the remote feature and its local regression case. The focused filters run on 2026-08-03 passed all 12 tests.
- Formatting, diff whitespace checks, and workspace Clippy pass.
- Generated command help contains `--remote <URI>` for both supported create subcommands.
- The full workspace test run on 2026-08-03 passed all 939 executed tests, with 15 tests skipped.

# Remaining work

Harden the workflow before live validation:

1. Add a side-effect-free preflight phase before the GitLab create-runner request:
   - Validate runtime and configured runner compatibility.
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

No disposable SSH Docker endpoint was available in the development environment, so this plan remains `in-progress`.

# Acceptance criteria

1. `acorn create bot <PROJECT_ID> --remote ssh://some-domain` creates the detached bot container on `some-domain` and not on the local daemon.
2. `acorn create runner ... --remote ssh://some-domain` creates and registers a Docker runner on `some-domain`, including the GPU-template path when enabled.
3. Every Docker operation in one create workflow targets the same explicit endpoint without changing Docker context or process-wide environment.
4. Invalid SSH endpoints and unsupported runtime or executor combinations fail before container creation; runner incompatibilities fail before the GitLab create-runner API call.
5. Omitting `--remote` preserves current bot and runner behavior and argument construction.
6. Help text documents prerequisites, remote ownership of ports and volumes, and the fact that bot creation is already detached.
7. Remote daemon connectivity and required capabilities are checked before the GitLab create-runner API call, and preflight failure leaves no new GitLab or Docker resources.
8. A failure after GitLab runner creation triggers per-runner rollback of resources created by that attempt; batch rollback does not affect successful or unrelated runners.
9. The original operation failure remains primary when rollback or cleanup also fails, and the error reports every resource that may require manual cleanup.
10. Automatic cleanup never removes a pre-existing container or volume, and never removes a user-supplied bot state volume.

# Related plans

- [CLI parse and execution models](cli-parse-execution-models.md)
- [CLI test coverage initiative](../../testing/index.md)
