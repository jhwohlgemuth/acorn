---
type: Plan
title: "Normalize API environment bootstrap"
description: "Apply consistent dotenv loading and prioritized environment-variable discovery across providers."
difficulty: low
impact: medium
package: [acorn-lib]
tags: [refactor, api, environment, configuration]
status: in-progress
---

# Project code boundaries

- **Update:** environment/config bootstrap in `crates/acorn-lib/src/io/api/mod.rs`, provider modules, `src/io/config/{application,bucket}.rs`, CLI startup, and tests; later `acorn-host` composition.
- **Create:** an immutable resolved API configuration input consumed by `acorn-api` clients.
- **Delete:** provider-local environment reads after callers inject resolved configuration.

# Dependencies

Follow endpoint and authentication contracts; coordinate CLI bootstrap with [startup separation](../core/cli-startup-separation.md).

# Evidence

OpenAI, RAiD, and GitLab options repeat environment loading and fallback rules. Application path discovery now lives in `io/config/application.rs`.

The GitLab slice now resolves `GITLAB_TOKEN`, `PRIVATE_TOKEN`, then `CI_JOB_TOKEN` through the shared `io::api::Authenticator` and `Credential` abstractions, preserves whether the credential requires `PRIVATE-TOKEN` or `JOB-TOKEN`, and shares that result between API options and bucket downloads. OpenAI, RAiD, and the remaining application bootstrap are still pending.

# Recommendation

Provide one bootstrap helper that distinguishes required from optional values and accepts an explicit precedence list.

# Validation

- Add precedence and missing-secret tests beside each migrated provider and retain bucket token coverage under `io::config::tests`.
- Run `cargo test -p acorn-lib io::config::tests --lib`, focused API tests, `make lint`, and `make test`.
- GitLab validation completed on 2026-08-31: focused credential and bucket tests passed, `make lint` passed, and all 1,369 tests in `make test` passed with 15 skipped.

# Acceptance criteria

- GitLab access and job tokens use documented, deterministic precedence and token-specific HTTP headers across API and bucket requests.
- Remaining providers use consistent missing-secret errors without coupling application-file I/O, bucket transfer policy, and provider bootstrap in one module.
