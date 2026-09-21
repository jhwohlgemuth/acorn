---
type: Plan
title: "Isolate GitLab CI test and reporting runners"
description: "Route Rust, fuzz, and scheduled artifact-creation jobs to dedicated runners with isolated configuration, credentials, and build storage."
difficulty: medium
impact: high
package: [workspace]
tags: [ci, gitlab, runners, reliability, isolation, reporting]
status: in-progress
---

# Project code boundaries

- **Update:** `.gitlab-ci.yml`, the reusable artifact-creation template described in the [PowerAutomate reporting guide](../../docs/src/automation/powerautomate-reporting.md), runner registration/configuration outside the repository, and CI operations documentation.
- **Create:** dedicated runner registrations and isolated build/cache storage for Rust/fuzz jobs, plus a separately tagged scheduled-reporting pool with protected package and callback credentials; no new product code.
- **Delete:** the shared `tests` resource group only after tag routing and storage isolation are verified; do not remove recovery cleanup prematurely.

# Dependencies

The implemented portability job uses the Rust pool once available. The reporting pool follows the implemented PowerAutomate runner contract.

# Goal

Run Rust and fuzz tests concurrently where their dependencies allow without letting their Git checkouts, runner configurations, or build storage collide. Run scheduled artifact creation on a separately tagged pool whose protected package-upload and PowerAutomate callback credentials are unavailable to merge-request and ordinary test jobs.

# Current evidence

- GitLab Runner 18.5 encountered a stale `/builds/0/research-enablement/acorn/.git/index.lock` while checking out a test job.
- Runner configuration now records explicit tags and `run_untagged` policy, and the create-runner path sends those values to GitLab. The checked-in ACORN configuration identifies independent `arm64` and `amd64` workers.
- CI test, documentation, portability, and fuzz jobs currently select `arm64`, but they still share the `tests` resource group and therefore cannot demonstrate the intended Rust/fuzz concurrency.
- The repository sets `GIT_CLONE_PATH` with both `CI_CONCURRENT_ID` and `CI_JOB_ID`, and the affected jobs remove stale checkout locks in `pre_get_sources_script`.
- No checked-in job uses the planned `rust-tests` or `artifact-creation` tag, and repository evidence cannot prove that runner-manager tokens, configuration volumes, or build volumes are isolated.
- GitLab documents that shared build directories with concurrency greater than one can cause multiple jobs to use the same directory, and places responsibility for isolation on runner administrators.[^1]
- GitLab warns that cloned runner authentication tokens let cloned runner managers pick up the same jobs.[^2]

# Recommendation

Create a project-scoped pool for project `16689`:

| Pool | Required tags | Untagged jobs | Per-runner limit |
|---|---|---:|---:|
| Rust tests and fuzz smoke tests | `cpu`, `rust-tests` | disabled | 1 |
| Scheduled artifact creation | `artifact-creation`, `cpu` | disabled | 1 |

Register every worker independently. Do not copy another worker's authentication token or `config.toml`. Give each runner-manager container a unique name, a dedicated `/etc/gitlab-runner` configuration volume, and a dedicated persistent `/builds` volume. Configure the pool-specific build volume in the Docker executor's `volumes` list so its job containers use that storage. Keep `limit = 1` for each worker; scale a pool by adding another independently registered worker with its own configuration and build volumes.

After the pool is online, require an architecture tag plus `rust-tests` on Rust and fuzz jobs. Confirm that no generic runner has the specialized tag. Remove the shared `tests` resource group only after routing and storage isolation are verified, restoring parallel execution. Retain the stale-lock pre-source cleanup as recovery for interrupted checkouts.

Register the reporting worker independently with isolated configuration/build volumes. Protect the `artifact-creation` tag and restrict its package token plus inbound/outbound webhook secrets to protected scheduled default-branch jobs. Do not expose those credentials to merge-request pipelines. Retain report manifests and failed-render diagnostics under the workflow's explicit retention policy.

# Validation

1. Validate the updated `.gitlab-ci.yml` with GitLab CI Lint.
2. Confirm GitLab schedules `test` and `fuzz-smoke` only on runners tagged `rust-tests`.
3. Run both jobs concurrently and verify their logs identify different runner registrations and isolated build storage.
4. Cancel each job during checkout, rerun it, and confirm checkout recovers without an `index.lock` failure.
5. Run consecutive pipelines and confirm Cargo caches remain reusable within the pool.
6. Inspect runner configuration and Docker volumes to confirm workers do not share tokens, configuration volumes, or build volumes.
7. Add one independently registered worker and verify parallel jobs receive distinct build volumes.
8. Confirm merge-request pipelines cannot select `artifact-creation` or read its protected variables.
9. Run one scheduled default-branch report and verify its logs identify the reporting runner, publish only the declared package files, and retain failure diagnostics without leaking credentials.

# Acceptance criteria

1. Generic or untagged runners cannot select either specialized test job.
2. Rust and fuzz jobs can run concurrently without sharing a runner registration or build volume.
3. Interrupted checkouts recover on the next job without manual lock removal.
4. Adding pool capacity never requires copying a runner token, configuration file, or build volume.
5. The temporary shared resource group is removed only after all routing, concurrency, recovery, and cache checks pass.
6. Artifact-creation jobs run only on protected scheduled default-branch pipelines and cannot expose package or callback credentials to merge-request jobs.

[^1]: [GitLab Runner custom build directories and concurrency](https://docs.gitlab.com/ci/runners/configure_runners/#handling-concurrency)
[^2]: [GitLab Runner security guidance for cloned runners](https://docs.gitlab.com/runner/security/#cloning-a-runner)
