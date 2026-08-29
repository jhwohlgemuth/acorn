---
type: Plan
title: "Isolate GitLab CI runner pools"
description: "Route Rust, fuzz, and Python tests to separately registered runners with isolated configuration and build storage."
difficulty: medium
impact: high
package: [workspace]
tags: [ci, gitlab, runners, reliability, isolation]
status: planned
---

# Project code boundaries

- **Update:** `.gitlab-ci.yml`, runner registration/configuration outside the repository, and CI operations documentation.
- **Create:** dedicated runner registrations and isolated build/cache storage for Rust/fuzz and Python jobs; no new product code.
- **Delete:** the shared `tests` resource group only after tag routing and storage isolation are verified; do not remove recovery cleanup prematurely.

# Dependencies

Coordinate with [acorn-py extraction](../migrations/acorn-py-repository-extraction.md) so Python jobs are not migrated twice. Portable feature jobs from [no-std/WASM CI](portability/no-std-wasm-ci.md) use the Rust pool once available.

# Goal

Run Rust, fuzz, and Python tests concurrently where their dependencies allow without letting their Git checkouts, runner configurations, or build storage collide. Route each job only to its dedicated project-runner pool and retain warm build caches within that pool.

# Current evidence

- GitLab Runner 18.5 failed while checking out `test-python` because `/builds/0/research-enablement/acorn/.git/index.lock` already existed.
- The `test`, `fuzz-smoke`, and `test-python` jobs currently select the generic `cpu` tag and can be scheduled on the same runner infrastructure.
- The repository sets `GIT_CLONE_PATH` from `CI_CONCURRENT_ID` and preserves `target/` and the Pixi environment across fetches, so runner-manager instances that share build storage must have correctly isolated concurrency slots.
- As an immediate mitigation, all three jobs use the `tests` resource group and remove a stale checkout lock in `pre_get_sources_script`. This prevents them from overlapping but lengthens pipelines.
- GitLab documents that shared build directories with concurrency greater than one can cause multiple jobs to use the same directory, and places responsibility for isolation on runner administrators.[^1]
- GitLab warns that cloned runner authentication tokens let cloned runner managers pick up the same jobs.[^2]

# Recommendation

Create two project-scoped pools for project `16689`:

| Pool | Required tags | Untagged jobs | Per-runner limit |
|---|---|---:|---:|
| Rust tests and fuzz smoke tests | `cpu`, `rust-tests` | disabled | 1 |
| Python tests | `cpu`, `python-tests` | disabled | 1 |

Register every worker independently. Do not copy another worker's authentication token or `config.toml`. Give each runner-manager container a unique name, a dedicated `/etc/gitlab-runner` configuration volume, and a dedicated persistent `/builds` volume. Configure the pool-specific build volume in the Docker executor's `volumes` list so its job containers use that storage. Keep `limit = 1` for each worker; scale a pool by adding another independently registered worker with its own configuration and build volumes.

After both pools are online, require `cpu` and `rust-tests` on `test` and `fuzz-smoke`, and require `cpu` and `python-tests` on `test-python`. Confirm that no generic runner has either specialized tag. Remove the shared `tests` resource group only after routing and storage isolation are verified, restoring parallel execution. Retain the stale-lock pre-source cleanup as recovery for interrupted checkouts.

# Validation

1. Validate the updated `.gitlab-ci.yml` with GitLab CI Lint.
2. Confirm GitLab schedules `test` and `fuzz-smoke` only on runners tagged `rust-tests`, and `test-python` only on a runner tagged `python-tests`.
3. Run each Rust-pool job concurrently with `test-python` where dependencies allow and verify their logs identify different runner registrations and isolated build storage.
4. Cancel each of the three jobs during checkout, rerun it, and confirm checkout recovers without an `index.lock` failure.
5. Run consecutive pipelines and confirm Cargo and Pixi caches remain reusable within their respective pools.
6. Inspect runner configuration and Docker volumes to confirm no token, configuration volume, or build volume is shared across pools.
7. Add one independently registered worker to a pool and verify parallel jobs receive distinct build volumes.

# Acceptance criteria

1. Generic or untagged runners cannot select any of the three specialized test jobs.
2. Rust and fuzz jobs can run concurrently with Python tests where dependencies allow without sharing a runner registration or build volume.
3. Interrupted checkouts recover on the next job without manual lock removal.
4. Adding pool capacity never requires copying a runner token, configuration file, or build volume.
5. The temporary shared resource group is removed only after all routing, concurrency, recovery, and cache checks pass.

[^1]: [GitLab Runner custom build directories and concurrency](https://docs.gitlab.com/ci/runners/configure_runners/#handling-concurrency)
[^2]: [GitLab Runner security guidance for cloned runners](https://docs.gitlab.com/runner/security/#cloning-a-runner)
