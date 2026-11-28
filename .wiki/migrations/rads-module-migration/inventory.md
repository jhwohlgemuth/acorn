# Exact Rust path inventory

Generated from `rg -l 'research_activity::' crates --glob '*.rs'` on November 27, 2026 at `a78440bf` plus the working tree. These are the source paths with explicit module-path references; re-run the inventory at phase start to catch concurrent additions. A phase's exact write set is its named group below plus the explicitly named extra paths in the plan. Paths in a move group mean both old and corresponding new paths.

## Schema source directory (move every path to standard/rads/) (18)

- `crates/acorn-schema/src/research_activity/aspect/data.rs`
- `crates/acorn-schema/src/research_activity/aspect/mod.rs`
- `crates/acorn-schema/src/research_activity/common.rs`
- `crates/acorn-schema/src/research_activity/graph.rs`
- `crates/acorn-schema/src/research_activity/graph_rdf.rs`
- `crates/acorn-schema/src/research_activity/logbook.rs`
- `crates/acorn-schema/src/research_activity/markdown.rs`
- `crates/acorn-schema/src/research_activity/mod.rs`
- `crates/acorn-schema/src/research_activity/output.rs`
- `crates/acorn-schema/src/research_activity/s3a.rs`
- `crates/acorn-schema/src/research_activity/tests/graph.rs`
- `crates/acorn-schema/src/research_activity/tests/graph_rdf.rs`
- `crates/acorn-schema/src/research_activity/tests/laws.rs`
- `crates/acorn-schema/src/research_activity/tests/mod.rs`
- `crates/acorn-schema/src/research_activity/tests/snapshots/acorn_schema__research_activity__tests__to_prose_default.snap`
- `crates/acorn-schema/src/research_activity/tests/snapshots/acorn_schema__research_activity__tests__to_prose_with_subtitle.snap`
- `crates/acorn-schema/src/research_activity/tests/snapshots/acorn_schema__research_activity__tests__to_prose_with_websites.snap`
- `crates/acorn-schema/src/research_activity/tests/validation_regressions.rs`

## Schema crosswalk directory (move every path to standard/crosswalk/rads/) (4)

- `crates/acorn-schema/src/standard/crosswalk/research_activity/datacite.rs`
- `crates/acorn-schema/src/standard/crosswalk/research_activity/mod.rs`
- `crates/acorn-schema/src/standard/crosswalk/research_activity/output.rs`
- `crates/acorn-schema/src/standard/crosswalk/research_activity/tests/mod.rs`

## Other schema consumers (18)

- `crates/acorn-schema/src/agent/mod.rs`
- `crates/acorn-schema/src/agent/tests/mod.rs`
- `crates/acorn-schema/src/codec.rs`
- `crates/acorn-schema/src/computation/mod.rs`
- `crates/acorn-schema/src/computation/tests/mod.rs`
- `crates/acorn-schema/src/hardware/mod.rs`
- `crates/acorn-schema/src/hardware/tests/mod.rs`
- `crates/acorn-schema/src/infrastructure.rs`
- `crates/acorn-schema/src/standard/cff.rs`
- `crates/acorn-schema/src/standard/crosswalk/dublin_core.rs`
- `crates/acorn-schema/src/standard/crosswalk/mod.rs`
- `crates/acorn-schema/src/standard/crosswalk/rrid.rs`
- `crates/acorn-schema/src/standard/crosswalk/tests/dublin_core.rs`
- `crates/acorn-schema/src/standard/invenio.rs`
- `crates/acorn-schema/src/tests/mod.rs`
- `crates/acorn-schema/src/tests/types.rs`
- `crates/acorn-schema/src/validation/custom.rs`
- `crates/acorn-schema/src/validation/tests/custom.rs`

## Portable and interface consumers (11)

- `crates/acorn-analysis/src/check.rs`
- `crates/acorn-analysis/src/error.rs`
- `crates/acorn-analysis/src/tests/mod.rs`
- `crates/acorn-service/src/repository.rs`
- `crates/acorn-service/src/service.rs`
- `crates/acorn-service/src/tests/mod.rs`
- `crates/acorn-wasm/src/lib.rs`
- `crates/acorn-wasm/src/tests.rs`
- `crates/acorn-web/src/research_graph.rs`
- `crates/acorn-web/src/sample.rs`
- `crates/acorn-web/src/tests/mod.rs`

## Host API consumers (15)

- `crates/acorn-lib/src/io/api/gitlab/automation.rs`
- `crates/acorn-lib/src/io/api/gitlab/package.rs`
- `crates/acorn-lib/src/io/api/gitlab/tests/jsonl.rs`
- `crates/acorn-lib/src/io/api/gitlab/tests/package.rs`
- `crates/acorn-lib/src/io/api/openalex.rs`
- `crates/acorn-lib/src/io/api/osti.rs`
- `crates/acorn-lib/src/io/api/powerautomate/form.rs`
- `crates/acorn-lib/src/io/api/powerautomate/jsonl.rs`
- `crates/acorn-lib/src/io/api/powerautomate/local/general.rs`
- `crates/acorn-lib/src/io/api/powerautomate/local/manifest.rs`
- `crates/acorn-lib/src/io/api/powerautomate/local/mod.rs`
- `crates/acorn-lib/src/io/api/powerautomate/tests/local.rs`
- `crates/acorn-lib/src/io/api/powerautomate/tests/mod.rs`
- `crates/acorn-lib/src/io/api/tests/mod.rs`
- `crates/acorn-lib/src/io/api/tests/openalex.rs`

## Host MCP consumers (10)

- `crates/acorn-lib/src/io/mcp/tests/server.rs`
- `crates/acorn-lib/src/io/mcp/tools/mod.rs`
- `crates/acorn-lib/src/io/mcp/tools/native/export/mod.rs`
- `crates/acorn-lib/src/io/mcp/tools/native/export/projection.rs`
- `crates/acorn-lib/src/io/mcp/tools/native/format.rs`
- `crates/acorn-lib/src/io/mcp/tools/native/link.rs`
- `crates/acorn-lib/src/io/mcp/tools/native/report.rs`
- `crates/acorn-lib/src/io/mcp/tools/native/tests/link.rs`
- `crates/acorn-lib/src/io/mcp/tools/needle.rs`
- `crates/acorn-lib/src/io/mcp/tools/tests/format.rs`

## Host analysis, execution, and benches (10)

- `crates/acorn-lib/benches/research_activity.rs`
- `crates/acorn-lib/benches/vocabulary.rs`
- `crates/acorn-lib/src/analyzer/diff/mod.rs`
- `crates/acorn-lib/src/analyzer/diff/tests.rs`
- `crates/acorn-lib/src/analyzer/fix/execution.rs`
- `crates/acorn-lib/src/analyzer/fix/tests.rs`
- `crates/acorn-lib/src/analyzer/host/runtime/mod.rs`
- `crates/acorn-lib/src/analyzer/host/runtime/tests/mod.rs`
- `crates/acorn-lib/src/analyzer/host/service/mod.rs`
- `crates/acorn-lib/src/exec/tests.rs`

## Host document, export, and other I/O consumers (25)

- `crates/acorn-lib/src/io/adapter/migrate.rs`
- `crates/acorn-lib/src/io/adapter/mod.rs`
- `crates/acorn-lib/src/io/adapter/tests/mod.rs`
- `crates/acorn-lib/src/io/chart.rs`
- `crates/acorn-lib/src/io/chart/tests.rs`
- `crates/acorn-lib/src/io/config/transfer.rs`
- `crates/acorn-lib/src/io/database/logbook.rs`
- `crates/acorn-lib/src/io/database/schema.rs`
- `crates/acorn-lib/src/io/database/tests/logbook.rs`
- `crates/acorn-lib/src/io/document/mod.rs`
- `crates/acorn-lib/src/io/document/research_activity.rs`
- `crates/acorn-lib/src/io/enrichment/mod.rs`
- `crates/acorn-lib/src/io/export/mod.rs`
- `crates/acorn-lib/src/io/export/plan.rs`
- `crates/acorn-lib/src/io/export/powerpoint.rs`
- `crates/acorn-lib/src/io/export/research_activity.rs`
- `crates/acorn-lib/src/io/export/schema.rs`
- `crates/acorn-lib/src/io/export/tests.rs`
- `crates/acorn-lib/src/io/export/updates/html.rs`
- `crates/acorn-lib/src/io/export/updates/mod.rs`
- `crates/acorn-lib/src/io/powerpoint/tests/mod.rs`
- `crates/acorn-lib/src/io/tests/enrichment/mod.rs`
- `crates/acorn-lib/src/io/workflow/logbook.rs`
- `crates/acorn-lib/src/io/workflow/tests/logbook.rs`
- `crates/acorn-lib/src/io/workflow/tests/mod.rs`

## Host test utilities and integration tests (5)

- `crates/acorn-lib/src/test/utils.rs`
- `crates/acorn-lib/tests/acp_interoperability.rs`
- `crates/acorn-lib/tests/ollama_interoperability.rs`
- `crates/acorn-lib/tests/openai_interoperability.rs`
- `crates/acorn-lib/tests/ownership.rs`

## CLI consumers (32)

- `crates/acorn-cli/examples/semantic_diff.rs`
- `crates/acorn-cli/examples/validate_activity.rs`
- `crates/acorn-cli/src/cli/mod.rs`
- `crates/acorn-cli/src/cli/tests/mod.rs`
- `crates/acorn-cli/src/commands/check/mod.rs`
- `crates/acorn-cli/src/commands/check/tests/mod.rs`
- `crates/acorn-cli/src/commands/download/tests/mod.rs`
- `crates/acorn-cli/src/commands/export/html.rs`
- `crates/acorn-cli/src/commands/export/mod.rs`
- `crates/acorn-cli/src/commands/export/pdf.rs`
- `crates/acorn-cli/src/commands/export/powerpoint/tests/mod.rs`
- `crates/acorn-cli/src/commands/export/tests/mod.rs`
- `crates/acorn-cli/src/commands/export/tests/updates.rs`
- `crates/acorn-cli/src/commands/export/updates.rs`
- `crates/acorn-cli/src/commands/format/mod.rs`
- `crates/acorn-cli/src/commands/format/tests/mod.rs`
- `crates/acorn-cli/src/commands/gather/tests/mod.rs`
- `crates/acorn-cli/src/commands/intake/graduation.rs`
- `crates/acorn-cli/src/commands/intake/graduation/tests.rs`
- `crates/acorn-cli/src/commands/intake/source/tests.rs`
- `crates/acorn-cli/src/commands/intake/tests.rs`
- `crates/acorn-cli/src/commands/link/mod.rs`
- `crates/acorn-cli/src/commands/link/tests/legacy.rs`
- `crates/acorn-cli/src/commands/report/mod.rs`
- `crates/acorn-cli/src/commands/report/powerpoint.rs`
- `crates/acorn-cli/src/commands/report/tests/mod.rs`
- `crates/acorn-cli/src/commands/report/tests/powerpoint.rs`
- `crates/acorn-cli/src/commands/report/tests/registry.rs`
- `crates/acorn-cli/src/template/mod.rs`
- `crates/acorn-cli/src/template/tests/mod.rs`
- `crates/acorn-cli/tests/intake.rs`
- `crates/acorn-cli/tests/registry/export.rs`

## Additional source and build paths

- `crates/acorn-schema/src/lib.rs`
- `crates/acorn-schema/src/standard/mod.rs`
- `crates/acorn-lib/src/io/document/mod.rs`
- `crates/acorn-lib/src/io/enrichment/mod.rs`
- `crates/acorn-lib/src/io/export/mod.rs`
- `crates/acorn-lib/Cargo.toml`
- `crates/acorn-lib/src/io/enrichment/research_activity.rs` (also in Other host consumers)
- `fuzz/Cargo.toml`
- `fuzz/fuzz_targets/research_activity.rs` → `fuzz/fuzz_targets/rads.rs`
- `Makefile`
- `.gitlab-ci.yml`
- `README.md`
- `ARCHITECTURE.md`

The three private host files and benchmark already appear in the host groups. Their rename targets are `rads.rs` in the same parent directories. `fuzz/corpus/research_activity/` is an input corpus directory; retain its contents and decide the target path during the fuzz-target phase. Generated coverage (`lcov.info`) is not a source edit.
