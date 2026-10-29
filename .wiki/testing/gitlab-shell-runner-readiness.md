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

GSR-02 is blocked. The host and service are running, but a tagged job cannot finish and publish its final status. Active work resumes only after that external failure clears.

# Current evidence

- `parse_executor` accepts `shell`; `resolve_operation` selects it; `register_runner` directly invokes local `gitlab-runner register --non-interactive --executor shell` without creating a container. Configured `"executor": "shell"` deserializes to the shell variant and serializes back to the same string. Focused parsing, selection, executor serialization, and schema tests pass; the create and configuration guides and generated CLI help cover the local path. Local GitLab Runner 19.5.0 help accepts the registration arguments.
- On October 29, 2026, this Windows 11 Pro x86_64 host had GitLab Runner 19.5.2 (`66ad5002`, `windows/amd64`). Service `gitlab-runner` runs as `LocalSystem`, Automatic, with config `C:\Users\jason\dev\re\acorn\config.toml` and working directory `C:\Users\jason\dev\re\acorn`. Runner 55645 (`https://code.ornl.gov/`) is executor `shell`. `gitlab-runner verify` accepts the token. `token_expires_at = 0001-01-01T00:00:00Z` is Go's zero time and means no expiration; it is not the job-update failure. Do not record or commit the token; `config.toml` and `builds/` are gitignored. `gitlab-runner list` prints the token.
- `LocalSystem` cannot execute user-scoped Scoop `pwsh` or Scoop `git`. The config shell is `powershell`. Machine-wide Git is installed at `C:\Program Files\Git\cmd\git.exe` and is on the machine PATH. Job [4483707](https://code.ornl.gov/research-enablement/acorn/-/jobs/4483707) failed on missing `pwsh`. Job [4483726](https://code.ornl.gov/research-enablement/acorn/-/jobs/4483726) failed on missing `git`.
- Runner 55645 can receive the `windows-x86_64` tag, check out a branch, and upload an early trace, then lose the job. Job [4483744](https://code.ornl.gov/research-enablement/acorn/-/jobs/4483744) checked out `fc7a746f` on `ci/windows-shell-smoke` and was canceled after 34 seconds. The final trace and status update returned 403, so the UI can remain on the checkout line and still say running. Leave `ci/windows-shell-smoke` in place until a passing job is recorded. Do not register a second runner.
- GitLab Runner registration adds an entry to `config.toml`; a running service reloads most configuration changes automatically. Registration must write the configuration used by that service. [GitLab commands](https://docs.gitlab.com/runner/commands/), [GitLab configuration](https://docs.gitlab.com/runner/configuration/advanced-configuration/).
- Windows shell jobs default to PowerShell Core (`pwsh`); the host must provide it to the service or explicitly configure `shell = "powershell"`. [GitLab shells](https://docs.gitlab.com/runner/shells/).

# Boundaries and decisions

- Use the existing `--runtime shell` and `executor: shell` interfaces. `--host`, `--context`, and configured `target.host` continue to describe container targets; shell registration is local to the ACORN process.
- The direct path is `acorn create runner --no-config --group <ID> --description <NAME> --runtime shell --tags windows-x86_64 --repo <GITLAB_DOMAIN>`; a project ID may replace the group ID.
- The Windows host prerequisites are met: GitLab Runner 19.5.2 is installed, service `gitlab-runner` is running against the written config, and the service shell is `powershell`. Do not register a second runner.
- Use `--repo` for a self-managed GitLab domain. Do not add a new flag or alter runner creation API semantics for this outcome.
- The Windows host phase uses its own external environment and does not block Linux or macOS implementation phases.

# Validation and acceptance

- Local CLI parsing, executor selection, serialization, and schema tests pass; the shell registration branch invokes `gitlab-runner` directly without a Docker/Podman command, and the command guide and generated help describe the same path.
- Runner 55645 keeps the `windows-x86_64` tag, its running service keeps reading the written configuration, and a small tagged PowerShell job records native OS and architecture. Keep the runner ID, tag, job link, and configuration context in `CONTRIBUTING.md` without recording tokens.
- The verified Windows runner is available to the Needle, model registry, and Podman Windows test phases; each retains its own toolchain and workload prerequisites.

# Troubleshooting

Run these from `C:\Users\jason\dev\re\acorn` so they use the service config. Do not copy the token printed by `gitlab-runner list`.

```text
gitlab-runner --version
gitlab-runner verify --config C:\Users\jason\dev\re\acorn\config.toml
gitlab-runner --debug verify --config C:\Users\jason\dev\re\acorn\config.toml
gitlab-runner list --config C:\Users\jason\dev\re\acorn\config.toml
gitlab-runner status
```

`gitlab-runner status` and a runner-managed restart return Access denied unless elevated, because the service runs as `LocalSystem`. Use an elevated PowerShell for the service:

```powershell
Get-Service gitlab-runner | Format-List Name, Status, StartType
Get-CimInstance Win32_Service -Filter "Name='gitlab-runner'" | Format-List State, StartName, PathName
Restart-Service gitlab-runner
Get-WinEvent -LogName Application -MaxEvents 30 | Where-Object ProviderName -eq 'gitlab-runner'
```

A valid `verify` result does not mean job updates work. A 403 on "Checking for jobs" or "Appending trace" leaves the UI stuck on the last uploaded line.

# Execution phases

## GSR-02 — Prove a native Windows shell runner

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: runner 55645 must finish a tagged job and upload its final status. Job 4483744 was canceled after checkout, and the final update returned 403.
- Exact write set: GitLab project 16689 runner 55645 tags, one disposable CI job, `CONTRIBUTING.md`, `.wiki/testing/gitlab-shell-runner-readiness.md`, `.wiki/testing/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/migrations/index.md`, `.wiki/migrations/needle-3-upgrade.md`, `.wiki/features/model-acquisition-hardening.md`, `.wiki/migrations/podman-makefile.md`
- Primary integration lock: Windows runner host
- Outcome: Finish a tagged PowerShell job on runner 55645 and record its native OS and architecture.
- Focused validation: Run a disposable project 16689 job tagged only `windows-x86_64` that prints the PowerShell version and native OS/architecture. Confirm GitLab assigns it to runner 55645.
- Handoff: Do not delete `ci/windows-shell-smoke`, do not register another runner, and do not paste the token. Cancel the stuck UI job before retrying it. After a job finishes and its log shows native OS, architecture, and PowerShell version, record the job link in `CONTRIBUTING.md`, then remove GSR-02 and retire this plan.
