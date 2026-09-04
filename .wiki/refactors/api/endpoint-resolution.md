---
type: Plan
title: "Extract endpoint resolution helpers"
description: "Resolve named templates and provider domains through one shared API helper."
difficulty: not-assessed
impact: high
package: [acorn-lib]
tags: [refactor, api, endpoints]
status: in-progress
---

# Project code boundaries

- **Update:** remaining provider endpoint construction in `crates/acorn-lib/src/io/api/`, endpoint catalog entries in `crates/acorn-lib/assets/constants/application.json`, and focused provider/config tests; later `acorn-api`.
- **Create:** None; the shared `Endpoint` lookup, domain, and URL-rendering helpers are implemented.
- **Delete:** duplicated provider endpoint interpolation and fallback branches.

# Dependencies

Consumes the completed authentication contract, coordinates with request composition, and precedes OpenAPI endpoint validation; environment reads stay in host composition.

# Evidence

`ApplicationConfiguration.endpoints` and application-file I/O live in `crates/acorn-lib/src/io/config/application.rs`. `Endpoint::{from_template,with_domain,resource_url}` now provide catalog lookup, provider-domain substitution, and request-free resource rendering. Hugging Face `resolve_url` and bucket GitLab raw-file downloads use those helpers with templates from `crates/acorn-lib/assets/constants/application.json`. Provider service functions still repeat template lookup followed by domain substitution, so the migration is incomplete.

# Recommendation

Migrate the remaining provider endpoint-construction pipelines to the existing `Endpoint` helpers, preserving provider-specific resource names and parameter validation.

# Validation

- Cover application endpoint parsing with `cargo test -p acorn-lib io::config::tests::test_config_with_endpoints --lib` and the built-in endpoint snapshot.
- `cargo test -p acorn-lib test_download_item_gitlab --lib` passed on 2026-09-04 for catalog-rendered raw-file URLs, including default and resolved branches.
- `make lint` and `make test` passed on 2026-09-04; the workspace suite ran 1,424 tests successfully with 14 skipped.
- Run focused provider endpoint tests, `make lint`, and `make test` after the helper migration.

# Acceptance criteria

- [x] Named resources can be rendered without a request through `Endpoint::resource_url`.
- [x] Hugging Face resolve URLs and bucket GitLab raw-file URLs use the embedded endpoint catalog.
- [ ] OpenAI, GitLab, RAiD, and other applicable providers no longer duplicate endpoint construction pipelines.
- [ ] Application endpoint configuration remains compatible through the `io::config` facade after the remaining provider migrations.
