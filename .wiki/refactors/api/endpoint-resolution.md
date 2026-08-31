---
type: Plan
title: "Extract endpoint resolution helpers"
description: "Resolve named templates and provider domains through one shared API helper."
difficulty: not-assessed
impact: high
package: [acorn-lib]
tags: [refactor, api, endpoints]
status: planned
---

# Project code boundaries

- **Update:** endpoint/configuration logic in `crates/acorn-lib/src/io/api/mod.rs`, provider modules, `src/io/config/application.rs`, and config/provider tests; later `acorn-api`.
- **Create:** pure template/domain resolution values and validation helpers.
- **Delete:** duplicated provider endpoint interpolation and fallback branches.

# Dependencies

Precedes authentication, request composition, and OpenAPI endpoint validation; environment reads stay in host composition.

# Evidence

`ApplicationConfiguration.endpoints` and application-file I/O now live in `crates/acorn-lib/src/io/config/application.rs`, while provider modules still repeat template lookup followed by domain substitution. The stable `io::config` re-export means this move does not itself implement endpoint resolution.

# Recommendation

Add root helpers for named endpoint lookup and template-plus-domain resolution, preserving provider-specific template names.

# Validation

- Cover application endpoint parsing with `cargo test -p acorn-lib io::config::tests::test_config_with_endpoints --lib` and the built-in endpoint snapshot.
- Run focused provider endpoint tests, `make lint`, and `make test` after the helper migration.

# Acceptance criteria

OpenAI, GitLab, RAiD, and other applicable providers no longer duplicate endpoint construction pipelines, and application endpoint configuration remains compatible through the `io::config` facade.
