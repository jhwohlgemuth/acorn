---
type: Plan
title: "Compose Vale configuration normalization"
description: "Model Vale configuration changes as composable endomorphisms applied once."
difficulty: low
impact: medium
package: [acorn-lib]
tags: [refactor, analyzer, vale, category-theory]
status: planned
---

# Goal

Make Vale configuration normalization a deterministic value transformation, then hand the normalized value to separate host operations for remote package resolution, INI materialization, filesystem writes, and process execution.

# Project code boundaries

- **Update:** `ValeConfig` in `crates/acorn-lib/src/analyzer/host/vale.rs`, its `StaticAnalyzerConfig` implementation and callers in `crates/acorn-lib/src/analyzer/host/{context,runtime}/`, and adjacent tests.
- **Create:** None initially. Extract a portable configuration value or a dedicated host materializer only when the existing module boundary cannot keep pure normalization separate from I/O.
- **Delete:** I/O-coupled normalization and duplicated package/rule cleanup at migrated call sites; do not remove consuming builder methods solely because they use local mutation internally.

# Dependencies

Follow the completed in-tree analyzer kernel/host split recorded by the [portability roadmap](../portability/roadmap.md). This Vale-only leaf may proceed independently when it avoids shared analyzer model files, but any package move remains deferred until a concrete portable consumer proves the ownership boundary.

# Evidence

- `ValeConfig` now lives under `analyzer::host`, while resolution and materialization live in `host/runtime.rs`; the paths in the earlier plan no longer match the code.
- `ValeConfig::resolve_package` is already a pure string normalization that trims values, preserves URI/path inputs, and expands named Vale packages.
- `StaticAnalyzerConfig::ini` currently mixes deterministic INI construction with an asynchronous GitLab release lookup and fallback URL selection.
- `save` creates directories and writes the INI file, while `ExecutionOptions::resolve_analyzer` unconditionally sequences `config.save().await` before analyzer resolution.
- Consuming methods such as `with_path(mut self, ...) -> Self` are value-preserving builder operations from the caller's perspective; local mutation alone is not evidence of an architectural problem.

# Recommendation

First name the actual transformations: trimming and resolving package sources, merging defaults and overrides, stable deduplication, and disabled-rule policy. Express each applicable step as a pure `ValeConfig -> ValeConfig` transformation and compose them in an explicit order. Keep remote release discovery, directory creation, INI writing, and Vale process execution in host functions that consume the normalized value.

Do not force asynchronous I/O into an endomorphism abstraction, and do not add a generic composition trait. If no second normalization step exists after the inventory, retain a named pure function rather than manufacturing a pipeline.

# Validation

Unit-test every pure transformation without filesystem or network access. Add host tests for fallback package URL selection and generated INI content, including the required `StylesPath` before `Vocab` ordering, then run the focused analyzer runtime tests.

# Acceptance criteria

1. Named packages, HTTP(S)/file URIs, and local paths retain current resolution behavior.
2. The normalization pipeline has a documented order and is idempotent where each named normalization claims idempotence.
3. Pure normalization tests require no filesystem, process, clock, or network capability.
4. Remote release lookup and fallback selection occur in a host operation, not while normalizing the value.
5. Offline materialization performs no network access; online fallback selection, generated INI content and ordering, save paths, and Vale invocation are covered by behavior-preserving tests.

# Related concepts

- [Category-inspired composition in ACORN](../../../docs/src/concepts/category-theory.md)
- [Portability roadmap](../portability/roadmap.md)
