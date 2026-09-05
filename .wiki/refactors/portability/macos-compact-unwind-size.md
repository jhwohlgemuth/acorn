---
type: Plan
title: "Reduce macOS compact-unwind pressure"
description: "Keep ACORN's Darwin debug and test binaries below the Mach-O compact-unwind encoding limit without suppressing linker diagnostics or weakening unwind correctness."
difficulty: high
impact: medium
package: [acorn-cli, acorn-lib, workspace]
tags: [refactor, macos, linker, binary-size, dependencies, features, build-profiles]
status: planned
---

# Project code boundaries

- **Update:** workspace/profile configuration in `Cargo.toml`, `crates/acorn-{lib,cli}/Cargo.toml`, feature selection, benchmark tooling, and CI measurement jobs.
- **Create:** reproducible size-measurement scripts/results only when needed to compare feature/profile changes.
- **Delete:** no linker diagnostics or required test coverage; remove experimental profile overrides that fail the acceptance threshold.

# Dependencies

Measure after [dependency feature defaults](dependency-feature-defaults.md) classify heavy stacks; crate extraction may change attribution but is not itself evidence of size reduction.

# Goal

Eliminate the macOS linker warning that reports an oversized `__eh_frame` section by reducing the amount of unwind-producing code linked into ACORN's debug and test binaries. Preserve command behavior, panic backtraces, crash reporting, foreign-exception safety, and useful linker diagnostics.

The implementation should solve the underlying size problem. It must not merely hide the warning, disable compact unwind globally, or move unrelated debug information out of the executable.

# Current evidence

The warning was observed while linking the `acorn-cli` binary on Apple Silicon:

```text
warning: linker stderr: ld: __eh_frame section too large (max 16MB) to encode dwarf unwind offsets in compact unwind table, performance of exception handling might be affected
  |
  = note: `#[warn(linker_messages)]` on by default

warning: `acorn-cli` (bin "acorn") generated 1 warning
```

The baseline was inspected on 2026-08-03 with:

- macOS/Darwin `25.6.0` on `arm64`;
- Apple linker `ld-1267`;
- `rustc 1.97.1 (8bab26f4f 2026-07-14)`, installed through Homebrew;
- LLVM `22.1.8`;
- the existing `target/debug/acorn` artifact.

`size -m target/debug/acorn` reported:

| Measurement | Baseline |
| --- | ---: |
| Binary size | approximately 317 MiB |
| `__text` | 91,264,676 bytes |
| `__gcc_except_tab` | 4,250,052 bytes |
| `__unwind_info` | 3,861,328 bytes |
| `__eh_frame` | 21,230,536 bytes (20.25 MiB) |
| Compact-unwind limit named by `ld` | 16,777,216 bytes (16 MiB) |
| Amount above the limit | 4,453,320 bytes (4.25 MiB) |

The current `__eh_frame` must shrink by at least 20.98% merely to cross the limit. A boundary-only result would be fragile across compiler and dependency updates, so the implementation target is at most 14 MiB (`14,680,064` bytes), which requires a 30.85% reduction from the measured baseline and leaves 2 MiB of headroom.

The warning is a target-linker diagnostic surfaced through Rust's `linker_messages` lint. Rust documents that the lint forwards platform-specific linker warnings rather than generating this diagnosis itself ([Rust lint documentation](https://doc.rust-lang.org/stable/rustc/lints/listing/warn-by-default.html#linker-messages)). LLVM's Mach-O linker work documents the same class of failure: a DWARF offset can exceed the representation available in compact-unwind metadata, so the linker must detect the overflow rather than encode an invalid hint ([LLVM D147505](https://reviews.llvm.org/D147505)).

# Why release settings do not settle the issue

The workspace release profile already uses `panic = "abort"`, size optimization, LTO, and stripping. Those settings reduce the shipped release artifact, but they do not cover the developer and test artifacts that emit this warning.

Cargo's test profile inherits from the development profile, and Rust's test harness requires unwind behavior. Cargo explicitly ignores the profile panic setting for tests and forces dependencies used by tests to use the unwind strategy ([Cargo profile documentation](https://doc.rust-lang.org/cargo/reference/profiles.html#panic)). Therefore, setting development or release panic behavior to `abort` is not a complete stable solution for `cargo test` or `make test`.

Similarly, `debug`, `split-debuginfo`, and `strip` primarily control debugging symbols. Cargo describes these separately from panic strategy and runtime unwinding ([Cargo profile documentation](https://doc.rust-lang.org/cargo/reference/profiles.html#debug)). They may reduce the file or adjacent debug-data size, but they must not be accepted as a fix unless measurement shows that they reduce `__eh_frame` itself.

# Dependency and feature evidence

The default CLI currently combines multiple substantial subsystems in one executable:

- `acorn-cli` defaults to `pdf` and `tui`;
- `pdf` enables `chromiumoxide`, `percy-dom`, `fast_qr`, and the library `chart` feature;
- `acorn-cli` enables `acorn-lib`'s `doctor` and `powerpoint` features while retaining all `acorn-lib` default features;
- `acorn-lib` defaults to `analysis`, `agentic`, `cmd`, `shell-lint`, and `unicode`;
- `analysis` includes `bat`, `polars`, compression, archive, and parallel-processing dependencies;
- `agentic` includes the Axum, RMCP, and Tower stack;
- `shell-lint` includes the Shuck parser and linter;
- the `std` feature is currently an umbrella for unrelated host capabilities including HTTP, Hugging Face access, document generation, link checking, templates, SQLite, cryptography, ZIP archives, process discovery, progress output, and an async runtime.

This plan does not assume that every listed dependency is a major `__eh_frame` contributor. Rust generics, macro-generated protocols, enabled features, and final dead-code elimination can make source-level dependency counts misleading. Each proposed reduction must be justified by linked-artifact measurements.

# Scope

This plan covers:

1. Reproducible Darwin section-size measurement.
2. A low-risk dependency optimization experiment.
3. Feature and dependency attribution for the default CLI.
4. Capability-focused feature wiring if build-profile optimization does not provide durable headroom.
5. Targeted code or dependency replacement only where measured attribution supports it.
6. Binary separation only as a last resort.
7. macOS regression validation for section size and linker warnings.

# Non-goals

- Do not silence `linker_messages` at the workspace, crate, or command level.
- Do not pass `-no_compact_unwind` to Apple `ld` as the solution.
- Do not declare success based only on total file size, `__LINKEDIT`, or `.dSYM` size.
- Do not remove panic backtraces or reduce foreign-exception correctness.
- Do not remove user-facing commands merely to satisfy the size target.
- Do not redesign all workspace features as part of the first phase.
- Do not replace large dependencies without evidence that the replacement materially reduces `__eh_frame`.
- Do not optimize Linux or Windows artifacts unless a shared change naturally benefits them.

# Success metrics

The primary metric is the `__eh_frame` size of the default Apple Silicon debug/test `acorn` binary.

| Metric | Required result |
| --- | --- |
| Default macOS debug/test `__eh_frame` | at most 14 MiB |
| Apple linker warning | absent without lint suppression |
| Compact unwind | remains enabled |
| Default CLI commands and features | preserved |
| `make test` | passes |
| `make lint` | passes |
| Release build | passes with existing release semantics |

Secondary measurements should include `__text`, `__gcc_except_tab`, `__unwind_info`, complete binary size, clean build time, and representative incremental build time. These measurements explain tradeoffs but do not replace the primary criterion.

# Measurement protocol

Use a cleanly identified artifact for every comparison. Record the Git commit, dirty-worktree state, Rust version, Apple linker version, architecture, enabled Cargo features, profile overrides, and exact command.

Baseline commands on macOS:

```bash
rustc -vV
ld -v
cargo build --package acorn-cli --bin acorn
size -m target/debug/acorn
otool -l target/debug/acorn
```

The measurement record should extract at least the named Mach-O sections rather than relying on visual inspection. If a reusable script is added, it should:

- accept an explicit binary path;
- fail when the binary or required section is absent;
- emit stable machine-readable output suitable for comparing variants;
- avoid writing outside `target/`;
- remain a developer tool rather than production CLI code.

For each experiment, build into a distinct target directory or remove only the explicitly identified experimental artifact. Do not compare a stale artifact compiled with different profile or feature settings.

# Implementation strategy

## Phase 0: Reproduce and automate the measurement

1. Reproduce the linker warning with the smallest stable repository command that still links the default `acorn` binary.
2. Confirm whether both `cargo build` and `cargo test` emit it, because the selected profile and test harness can produce different link graphs.
3. Capture the baseline section sizes using `size -m` and confirm them using `otool -l`.
4. Add a narrowly scoped measurement helper only if manual comparisons prove error-prone.
5. Record an Apple Silicon baseline in this plan before changing build profiles or feature wiring.
6. Measure Intel macOS when an available supported runner exists; do not block the Apple Silicon fix solely on unavailable hardware.

Exit condition: the warning and section size are reproducible, and two consecutive unchanged builds identify the same linked artifact and materially consistent section sizes.

## Phase 1: Optimize dependencies in development builds

Test the smallest plausible repository change first: optimize third-party dependencies in the development profile while leaving workspace crates easy to debug. The candidate shape is:

```toml
[profile.dev.package."*"]
opt-level = 1
```

This is an experiment, not a predetermined final patch. Evaluate `opt-level = 1`, `2`, and `s` only when the preceding setting does not achieve sufficient headroom. Avoid broad combinations of optimization, LTO, codegen units, stripping, and debug changes in one trial because they prevent attribution.

For each candidate:

1. Measure `__eh_frame` and the secondary metrics.
2. Run the full test suite.
3. Confirm ACORN workspace crates still retain expected source-level debugging behavior.
4. Compare clean and incremental build times.
5. Confirm the setting applies as intended to dependencies rather than silently changing the release artifact.

Accept this phase as the complete implementation only if the default test/debug artifact reaches at most 14 MiB with no behavioral regression and an acceptable developer-build cost. If it lands between 14 and 16 MiB, treat it as useful but insufficient headroom and continue.

Expected code size: approximately 2-10 manifest lines plus measurement documentation. Expected effort: one-half to one engineering day, dominated by clean rebuilds and validation.

## Phase 2: Attribute unwind size by capability

If Phase 1 is insufficient, build a controlled feature matrix. Change one feature boundary at a time and record the delta from the immediately preceding variant.

At minimum, compare:

1. Default `acorn-cli` features.
2. CLI defaults disabled.
3. PDF enabled without TUI.
4. TUI enabled without PDF.
5. `acorn-lib` defaults disabled with only the minimum features needed to compile a basic CLI.
6. Each library capability added independently where compilation permits: analysis, agentic, command execution, shell lint, doctor, PowerPoint, chart, and database support.
7. The final intended default feature union.

Cargo feature unification means a feature enabled anywhere in the graph can invalidate an intended comparison. Use `cargo tree --edges features --invert <crate>` and the resolved feature tree to verify each variant before interpreting its size.

Where section-level attribution remains unclear, generate and inspect a Darwin linker map or use a Rust-aware symbol-size tool. Treat symbol size as supporting evidence: the decision criterion remains the linked `__eh_frame` delta.

Produce a table containing:

| Variant | Enabled capabilities | `__text` | `__eh_frame` | Delta | Warning | Build time |
| --- | --- | ---: | ---: | ---: | --- | ---: |
| Baseline | Default | measured value | measured value | — | yes/no | measured value |

Exit condition: the team can identify the smallest set of capabilities responsible for enough unwind growth to recover the required headroom.

Expected committed code size: zero if existing feature wiring supports the experiment; otherwise only temporary local measurement changes. Expected effort: one-half to two engineering days depending on rebuild time and feature-boundary breakage.

## Phase 3: Replace the `std` umbrella with explicit capabilities

Proceed only if the feature matrix shows that default dependency wiring is retaining avoidable code. Coordinate this phase with [Tighten dependency feature defaults](dependency-feature-defaults.md), which addresses dependency capability leakage for portable builds.

The likely target model is:

- retain `core`, `alloc`, `std`, and `wasm` as platform capability layers;
- make `std` provide standard-library availability rather than automatically enabling every host service;
- place HTTP and Hugging Face clients behind an explicit network/model-download capability;
- place process discovery and executable checks behind a process capability;
- place link checking behind its own capability;
- place document, template, PowerPoint, PDF, and chart dependencies behind their actual export capabilities;
- keep database implementations behind database-specific features;
- keep Axum/RMCP/Tower behind agentic service features;
- keep analysis and shell lint dependencies behind their existing domain features;
- configure `acorn-cli`'s `acorn-lib` dependency with `default-features = false` and an explicit default feature union.

The exact feature names must follow existing repository terminology and be selected after Phase 2. Do not create one-use aliases that merely rename a dependency without expressing a stable capability.

Implementation steps:

1. Map every optional dependency to the module and command that requires it.
2. Define the minimum coherent capability graph without feature cycles.
3. Update `crates/acorn-lib/Cargo.toml` feature wiring.
4. Update `crates/acorn-cli/Cargo.toml` to request explicit library capabilities.
5. Gate imports, modules, public exports, and command branches at the narrowest maintainable boundary.
6. Keep unavailable-feature behavior explicit at CLI parsing or execution boundaries; avoid obscure downstream compilation failures.
7. Add compile checks for supported feature combinations.
8. Re-run the section-size matrix and retain only changes that contribute meaningful headroom or independently satisfy the linked portability plan.

Expected code size: approximately 300-800 changed lines across manifests, `#[cfg]` boundaries, module exports, command wiring, and tests. Expected effort: two to five engineering days. This estimate grows if public library APIs currently expose feature-specific types unconditionally.

## Phase 4: Reduce a measured heavyweight contributor

If explicit capability wiring preserves all default commands but the all-default binary remains oversized, optimize the largest measured contributor rather than making broad source-level micro-optimizations.

Candidate actions, in decision order:

1. Disable unused features on the responsible dependency.
2. Avoid duplicate major versions or parallel TLS/runtime stacks where the feature tree proves duplication.
3. Move generated protocol or format code behind the command that uses it.
4. Replace a heavyweight library used for a narrow operation with a smaller implementation, but only with compatibility tests for that operation.
5. Reduce generic monomorphization at stable internal boundaries through type erasure or non-generic helpers when symbol evidence identifies duplicated instantiations.

Possible contributors include the analysis stack (`polars` and `bat`), browser/PDF stack (`chromiumoxide` and generated CDP types), agentic server stack, rendering stack, and shell parser. These are hypotheses, not authorization to replace those libraries.

Every dependency replacement needs its own focused compatibility analysis. Preserve supported inputs, outputs, error behavior, offline behavior, and user-visible formatting.

Expected code size and effort: variable. A feature correction may be under 100 lines; replacing a major analysis or browser dependency may exceed 1,000 lines and should become a linked migration plan rather than being hidden inside this refactor.

## Phase 5: Split optional subsystems into helper binaries

Use binary separation only when measurements show that a single executable containing every default subsystem cannot retain adequate compact-unwind headroom.

Potential boundaries include PDF/browser rendering, analysis, agentic services, or the TUI. A valid split must define:

- installation and packaging of every helper;
- version compatibility between the front controller and helper;
- stable argument or message contracts;
- exit-code and diagnostic propagation;
- behavior when a helper is absent;
- offline execution and temporary-file handling;
- Homebrew, release archive, and development workflows;
- integration tests that exercise the installed layout.

Do not make users manually coordinate binaries for ordinary default commands. If transparent dispatch cannot be made reliable, prefer a measured dependency reduction or an explicit opt-in feature over a fragile helper architecture.

Expected code size: approximately 800-2,000 changed lines. Expected effort: one to two engineering weeks including packaging and cross-platform validation.

# Decision gates

Stop at the first phase that achieves the primary target and all acceptance criteria.

| Result | Decision |
| --- | --- |
| Dependency optimization reaches at most 14 MiB | Land Phase 1; do not refactor features solely for this warning. |
| Optimization reaches 14-16 MiB | Keep as candidate, but obtain additional headroom from a measured capability. |
| Optimization remains above 16 MiB | Complete the feature matrix before changing architecture. |
| One avoidable default capability accounts for sufficient size | Narrow its feature wiring. |
| Required all-default capabilities remain above target | Optimize the largest measured contributor. |
| No dependency-level change can provide headroom | Propose a separately reviewed helper-binary architecture. |

# Alternatives considered

## Allow or suppress `linker_messages`

Rejected as a fix. It changes only diagnostic visibility. Rust documents that this lint forwards the linker's own warning; allowing it does not change the generated Mach-O metadata. Suppression may also hide future actionable linker warnings from the same crate.

## Pass `-no_compact_unwind`

Rejected as a fix. It avoids constructing the compact-unwind table rather than making ACORN fit within it. This deliberately abandons the optimized representation implicated by the warning and can affect exception handling, crash reporting, or foreign-language unwinding.

## Strip symbols or split debug information

Rejected unless direct measurement proves an `__eh_frame` reduction. These settings primarily affect debugger information and total artifact size, while the failing section contains runtime unwind metadata.

## Set `panic = "abort"` for development and tests

Rejected as the complete solution. It changes development panic semantics, and Cargo's stable test harness still forces unwind behavior for tests and their dependencies.

## Build only in release mode

Rejected. It avoids the developer artifact rather than fixing it, slows ordinary iteration, and does not address test-harness links.

## Prefer dynamic linking

Deferred. Dynamic linking may reduce the main executable but changes distribution, loader, signing, portability, and packaging behavior. Consider it only as part of a broader packaging proposal with measured benefits.

# Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Optimized dependencies are harder to debug | Keep workspace crates unoptimized; document how to override the profile locally for dependency debugging. |
| Clean builds become slower | Measure clean and incremental times before accepting a profile setting. |
| Feature refactoring breaks uncommon configurations | Add compile checks for each supported feature set and retain explicit capability documentation. |
| Cargo feature unification hides dependency leakage | Inspect resolved feature trees for every matrix entry. |
| Size improvements disappear after a toolchain update | Target 14 MiB rather than the 16 MiB boundary and record toolchain metadata. |
| A replacement changes output semantics | Require command-specific golden or integration tests before replacement. |
| macOS-only settings leak to other targets | Prefer portable Cargo profile settings when beneficial; otherwise use narrowly scoped target configuration and run Linux CI. |
| Binary splitting complicates installation | Treat packaging and installed-layout tests as prerequisites, not follow-up work. |

# Validation

For the selected implementation:

1. Run `make lint`.
2. Run `make test` on macOS and confirm the target linker warning is absent.
3. Build the default debug CLI and record all baseline section metrics.
4. Confirm `__eh_frame` is at most 14 MiB with `size -m` and `otool -l`.
5. Build the release CLI and run representative smoke tests.
6. Exercise commands belonging to every affected capability, including download, analysis/checking, export/PDF, doctor, TUI, and agentic commands where applicable.
7. Check supported non-default feature combinations with `cargo check` or `cargo test` as appropriate.
8. Run the normal Linux CI path to catch unintended platform regressions.
9. Verify no `allow(linker_messages)`, `-no_compact_unwind`, or equivalent suppression was introduced.
10. Update this plan's evidence table with the final toolchain, commands, before/after sizes, build-time tradeoffs, and test results.

# Acceptance criteria

- A default Apple Silicon debug/test `acorn` artifact has `__eh_frame <= 14,680,064` bytes.
- The Apple linker no longer reports the 16 MiB compact-unwind warning during the documented reproduction command.
- The result is achieved with compact unwind enabled and without suppressing `linker_messages`.
- The measurement is reproducible from documented commands on a cleanly identified artifact.
- All existing default CLI capabilities remain available unless a separately approved user-facing feature-default change says otherwise.
- Panic backtraces, crash reporting assumptions, and foreign-exception behavior are not intentionally weakened.
- `make lint` and `make test` pass.
- Supported feature combinations compile and their resolved feature trees match the intended capability model.
- Release packaging and smoke tests pass if manifests, default features, or binary layout change.
- The final wiki evidence contains before/after `__eh_frame`, `__text`, total binary size, toolchain identity, and validation commands.
- Any dependency replacement or helper-binary work too large for this atomic plan is captured in a linked migration or refactor concept before implementation.

# Rollback

Build-profile and feature-wiring changes should remain independently revertible. If a selected change causes unacceptable build-time or debugging regressions, revert that phase and retain the measurement evidence for the next decision gate. Do not retain diagnostic suppression as a rollback substitute.

# Related plans

- [Tighten dependency feature defaults](dependency-feature-defaults.md)
- [Portability roadmap](roadmap.md)
- [Portability roadmap](roadmap.md)
- [no-std and WASM CI](no-std-wasm-ci.md)
