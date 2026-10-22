---
type: Plan
title: GitLab shell runner readiness
description: Document and verify ACORN's existing local shell registration path, then make a native Windows x86_64 GitLab runner available for platform tests.
difficulty: medium
impact: high
package: [acorn-cli]
tags: [testing, gitlab, runner, shell, windows, ci]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Make `acorn create runner` a documented, tested way to register a shell executor on a host with GitLab Runner already installed, and prove that a native Windows x86_64 runner can accept tagged PowerShell jobs. This plan does not install GitLab Runner or manage its service.

Estimated active work is 2–4 hours for local coverage and documentation, then 1–2 hours for Windows registration and a smoke job once the host and access are ready.

# Current evidence

- `parse_executor` accepts `shell`; `resolve_operation` selects it; `register_runner` invokes the local `gitlab-runner register --non-interactive --executor shell` path without creating a container. Configured runners also accept `"executor": "shell"`.
- The runner command's `--runtime` help and create guide list only container runtimes. Existing tests cover shell selection but not the shell registration arguments. The local GitLab Runner 19.4.1 help accepts the arguments ACORN passes.
- No native Windows runner is established in the current plan evidence. Needle's prepared Windows job requires a `windows-x86_64` shell runner; runner 55601 reports `linux/amd64` despite its `windows` tag.
- GitLab Runner registration adds an entry to `config.toml`; a running service reloads most configuration changes automatically. Registration must write the configuration used by that service. [GitLab commands](https://docs.gitlab.com/runner/commands/), [GitLab configuration](https://docs.gitlab.com/runner/configuration/advanced-configuration/).

# Boundaries and decisions

- Use the existing `--runtime shell` and `executor: shell` interfaces. `--host`, `--context`, and configured `target.host` continue to describe container targets; shell registration is local to the ACORN process.
- The direct path is `acorn create runner --no-config --group <ID> --description <NAME> --runtime shell --tags windows-x86_64 --repo <GITLAB_DOMAIN>`; a project ID may replace the group ID.
- Require an installed `gitlab-runner`, a running service using the same configuration, GitLab runner-creation access, and a native Windows x86_64 host before the Windows phase. These are external prerequisites, not ACORN installation work.
- Use `--repo` for a self-managed GitLab domain. Do not add a new flag or alter runner creation API semantics for this outcome.
- Serialize CLI help and parser edits with other CLI-contract owners. The Windows host phase uses its own external environment and does not block Linux or macOS implementation phases.

# Validation and acceptance

- CLI parsing, selected executor, and registration argument tests cover a local shell runner without Docker/Podman invocation; the command guide and generated help describe the same path.
- On Windows x86_64, register a runner tagged `windows-x86_64`, verify its service uses the written configuration, and run a small tagged PowerShell job that records native OS and architecture. Keep the runner ID, tag, job link, and configuration context in durable developer documentation without recording tokens.
- The verified Windows runner is available to the Needle, model registry, and Podman Windows test phases; each retains its own toolchain and workload prerequisites.

# Execution phases

## GSR-01 — Document and cover local shell registration

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `crates/acorn-cli/src/commands/create/runner/mod.rs`, `crates/acorn-cli/src/commands/create/runner/tests.rs`, `docs/src/cli-reference/commands/create.md`, `docs/src/cli-reference/configuration.md`, `CommandLineHelp.md`, `.wiki/testing/gitlab-shell-runner-readiness.md`
- Primary integration lock: CLI contract
- Outcome: Correct runtime/name help, add a shell example, and cover shell parsing and registration arguments. Extract only the registration argument construction needed for focused testing.
- Focused validation: Run focused CLI parse and create-runner tests; check generated help against the command docs.
- Handoff: Remove GSR-01 and record the accepted local registration command and prerequisites.

## GSR-02 — Prove a native Windows shell runner

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: GSR-01; native Windows x86_64 host with GitLab Runner installed and its service running; GitLab runner-creation access
- Exact write set: Windows x86_64 host, GitLab test project and disposable CI job, `CONTRIBUTING.md`, `.wiki/testing/gitlab-shell-runner-readiness.md`, `.wiki/testing/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/migrations/needle-3-upgrade.md`, `.wiki/features/model-acquisition-hardening.md`, `.wiki/migrations/podman-makefile.md`
- Primary integration lock: Windows runner host
- Outcome: Register the shell runner using ACORN, confirm the service reads its configuration, and make the `windows-x86_64` tag schedulable.
- Focused validation: Run a small tagged PowerShell job; record its native OS/architecture, runner ID, and job link without credentials.
- Handoff: Remove GSR-02, preserve the host and job evidence in `CONTRIBUTING.md`, then retire this plan and its index, network, and lane entries.
