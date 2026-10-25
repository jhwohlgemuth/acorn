---
type: Plan
title: GitLab shell runner readiness
description: Document and verify ACORN's existing local shell registration path, then make a native Windows x86_64 GitLab runner available for platform tests.
difficulty: medium
impact: high
package: [acorn-cli]
tags: [testing, gitlab, runner, shell, windows, ci]
status: waiting-on-validation
execution: phased
phase_budget_hours: 5
---

# Goal

Make `acorn create runner` a documented, tested way to register a shell executor on a host with GitLab Runner already installed, and prove that a native Windows x86_64 runner can accept tagged PowerShell jobs. This plan does not install GitLab Runner or manage its service.

Estimated remaining active work is 1–2 hours for Windows registration and a smoke job once the host and access are ready.

# Current evidence

- `parse_executor` accepts `shell`; `resolve_operation` selects it; `register_runner` directly invokes local `gitlab-runner register --non-interactive --executor shell` without creating a container. Configured `"executor": "shell"` deserializes to the shell variant and serializes back to the same string. Focused parsing, selection, executor serialization, and schema tests pass; the create and configuration guides and generated CLI help cover the local path. Local GitLab Runner 19.5.0 help accepts the registration arguments.
- No native Windows host is available. On October 25, GitLab project 16689 listed runner 55601 offline, and its manager reported `linux/amd64`. Needle's prepared Windows job requires a native `windows-x86_64` shell runner.
- GitLab Runner registration adds an entry to `config.toml`; a running service reloads most configuration changes automatically. Registration must write the configuration used by that service. [GitLab commands](https://docs.gitlab.com/runner/commands/), [GitLab configuration](https://docs.gitlab.com/runner/configuration/advanced-configuration/).
- Windows shell jobs default to PowerShell Core (`pwsh`); the host must provide it to the service or explicitly configure `shell = "powershell"`. [GitLab shells](https://docs.gitlab.com/runner/shells/).

# Boundaries and decisions

- Use the existing `--runtime shell` and `executor: shell` interfaces. `--host`, `--context`, and configured `target.host` continue to describe container targets; shell registration is local to the ACORN process.
- The direct path is `acorn create runner --no-config --group <ID> --description <NAME> --runtime shell --tags windows-x86_64 --repo <GITLAB_DOMAIN>`; a project ID may replace the group ID.
- Require an installed `gitlab-runner`, a running service using the same configuration, GitLab runner-creation access, a supported PowerShell shell, and a native Windows x86_64 host before the Windows phase. These are external prerequisites, not ACORN installation work.
- Use `--repo` for a self-managed GitLab domain. Do not add a new flag or alter runner creation API semantics for this outcome.
- The Windows host phase uses its own external environment and does not block Linux or macOS implementation phases.

# Validation and acceptance

- Local CLI parsing, executor selection, serialization, and schema tests pass; the shell registration branch invokes `gitlab-runner` directly without a Docker/Podman command, and the command guide and generated help describe the same path.
- On Windows x86_64, register a runner tagged `windows-x86_64`, verify its service uses the written configuration, and run a small tagged PowerShell job that records native OS and architecture. Keep the runner ID, tag, job link, and configuration context in durable developer documentation without recording tokens.
- The verified Windows runner is available to the Needle, model registry, and Podman Windows test phases; each retains its own toolchain and workload prerequisites.

# Execution phases

## GSR-02 — Prove a native Windows shell runner

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: native Windows x86_64 host with GitLab Runner installed and its service running, PowerShell Core available or Windows PowerShell configured, and GitLab runner-creation access
- Exact write set: Windows x86_64 host, GitLab test project and disposable CI job, `CONTRIBUTING.md`, `.wiki/testing/gitlab-shell-runner-readiness.md`, `.wiki/testing/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/migrations/needle-3-upgrade.md`, `.wiki/features/model-acquisition-hardening.md`, `.wiki/migrations/podman-makefile.md`
- Primary integration lock: Windows runner host
- Outcome: Register the shell runner using ACORN, confirm the service reads its configuration, and make the `windows-x86_64` tag schedulable.
- Focused validation: Run a small tagged PowerShell job; record its native OS/architecture, runner ID, and job link without credentials.
- Handoff: Remove GSR-02, preserve the host and job evidence in `CONTRIBUTING.md`, then retire this plan and its index, network, and lane entries.
