---
type: Plan
title: "Create containers on remote Docker hosts"
description: "Add an SSH-backed --remote target to container-producing create commands."
difficulty: medium
impact: high
package: [acorn-cli, acorn-lib]
tags: [feature, cli, create, docker, ssh]
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

# Validation

Automated validation completed on 2026-07-26:

- CLI tests cover valid SSH targets with usernames, ports, and socket paths and reject invalid schemes, missing hosts, whitespace, queries, fragments, invalid ports, and embedded passwords.
- Bot tests verify explicit remote targeting and retained detached behavior.
- Runner tests verify remote mounts, unsupported runtime and executor rejection, and unchanged local arguments. The copy operation is encapsulated by `Remote::copy_gpu_template`.
- Three focused `acorn-lib` Remote tests and six focused `acorn-cli` create-command tests pass.
- Formatting, diff whitespace checks, and workspace Clippy pass.
- Generated command help contains `--remote <URI>` for both supported create subcommands.
- The full default test run reached the new tests successfully. Existing unrelated failures remain in PowerPoint export, live HTTP/SPDX access, and zip fixture extraction.

# Remaining work

Perform a manual smoke test against a disposable SSH-accessible Docker host:

1. Create a bot remotely and confirm it is absent from local `docker ps` and present in `docker --host <URI> ps`.
2. Verify its published port, restart policy, and named state volume on the remote host.
3. Create and register a runner remotely, then verify its configuration volume and Docker socket mount.
4. Exercise GPU configuration and verify the copied in-container template.
5. Remove the disposable containers and volumes from the explicitly targeted remote host.

No disposable SSH Docker endpoint was available in the development environment, so this plan remains `in-progress`.

# Acceptance criteria

1. `acorn create bot <PROJECT_ID> --remote ssh://some-domain` creates the detached bot container on `some-domain` and not on the local daemon.
2. `acorn create runner ... --remote ssh://some-domain` creates and registers a Docker runner on `some-domain`, including the GPU-template path when enabled.
3. Every Docker operation in one create workflow targets the same explicit endpoint without changing Docker context or process-wide environment.
4. Invalid SSH endpoints and unsupported runtime or executor combinations fail before container creation; runner incompatibilities fail before the GitLab create-runner API call.
5. Omitting `--remote` preserves current bot and runner behavior and argument construction.
6. Help text documents prerequisites, remote ownership of ports and volumes, and the fact that bot creation is already detached.

# Related plans

- [CLI parse and execution models](cli-parse-execution-models.md)
- [CLI test coverage initiative](../../testing/index.md)
