---
okf_version: "0.1"
type: Plan
title: Coordinate incremental builds and reload development
description: Connect measured incremental builds, daemon configuration reload, safe development restarts and isolated function hot reload while retaining Cargo and browser release gates.
difficulty: high
impact: medium
package: [acorn-analysis, acorn-cli, acorn-cmd, acorn-core, acorn-daemon, acorn-diff, acorn-host, acorn-lib, acorn-macros, acorn-schema, acorn-server, acorn-service, acorn-tui, acorn-wasm, acorn-web, acorn-downstream-host-fixture, xtask]
tags: [build, developer-workflow, incremental-compilation, hot-reload, daemon, testing, os:macos, os:linux]
status: planned
execution: phased
phase_budget_hours: 5
---

# Outcome and recommendation

Qualify a pinned Blaze executable as an optional accelerator for repeated **native edit → selected tests → run** cycles. The highest potential payoff is `acorn-lib` with its analysis feature and the `acorn-cli` consumer; the best first compatibility probe is the much smaller `acorn-diff`. Portable schema, analysis and service tests are useful intermediate steps and can accelerate browser-domain development without replacing the browser compiler pipeline.

Do not add Blaze as an application dependency. Do not replace Cargo globally, change the default Make targets, or insert `blaze-live` into ACORN. Retain `make web-dev` for Dioxus and Vite development. An unsuccessful Blaze pilot is a valid outcome: publish its evidence and omit Blaze integration. The Cargo-first daemon development loop can still proceed independently. Retire this plan only after every retained development-loop branch is resolved.

Expected benefit is an architectural inference, not a measured speedup. ACORN compilation time, test execution time, cold startup, daemon memory and browser latency have not been measured with Blaze. No crate is declared compatible from this review alone.

# Connected development loop

The [development-loop contract](development-loop.md) connects this compiler review to [acornd](../../features/acornd-service.md). The daemon plan already proposed job configuration reload and an optional Hotcode function experiment; these were insufficiently connected in the initial Blaze review. Measure the complete save → build → verify → activate → ready → observed result path, including state recovery and failure behavior.

ACD-08A owns production configuration reload. BLZ-11 owns an isolated Cargo-first build/restart supervisor; BLZ-12 accepts that supervisor against the implemented daemon and adds its optional development target. BLZ-13 evaluates function hot reload separately from stateful daemon operation. Dioxus/Vite continue to own browser updates. No new coordination epic or second copy of the daemon implementation plan is needed.

# Review scope and evidence boundary

Review date: **November 12, 2026**. Repository baseline: `6c2a51b8`, with pre-existing uncommitted browser and Power Platform/wiki work. All 17 workspace manifests, crate roots, feature edges, build scripts, representative tests, Make entry points, Cargo configuration and relevant CI paths were inspected. The separate `fuzz` workspace was also reviewed. This is a comprehensive adoption-surface review, not a line-by-line correctness audit of every Rust implementation.

The [inventory](_assets/inventory.json) records every workspace package, source-file count, feature table, direct workspace dependencies and build-script presence. Counts describe the inspected tree and do not estimate compile time. `acorn-desktop` is planned under ABW-11 and is not a present workspace member.

Blaze was inspected from GitHub source commit **`39452df1ec738b68fbec6ff6e0d8919839d45355`**, committed at `2026-11-12T19:22:12Z`. Its source archive was read without installing or executing it. The local host reports arm64 and no `blaze` executable on PATH. A main-branch source snapshot does not prove which stable binary contains the same capabilities.

No Cargo tests, Clippy run, browser run or performance experiment was performed for this documentation-only review. In particular, `make lint` mutates sources and is inappropriate in this concurrently modified tree. Test locations below establish available coverage, not a new passing run. The November 7 portfolio-wide currency review remains current; this pass updates only this plan and directly related developer-loop evidence.

# Upstream findings

All links below pin the inspected commit. Verdicts refer to source claims, not ACORN runtime acceptance.

| Claim | Verdict and inspected evidence | Consequence |
|---|---|---|
| Blaze is simply a Cargo/rustc wrapper | **Incorrect.** Its [command implementation](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/crates/blaze-cargo/src/cli.rs) drives its daemon; [toolchain documentation](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/docs/toolchain.md#what-it-is-today) identifies its own dated toolchain. | Run explicit Blaze commands; preserve ordinary Cargo and rustup. |
| Repository Rust pins govern Blaze | **Incorrect.** The [toolchain description](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/docs/toolchain.md) and [harness pin](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/harness/toolchain) identify nightly-2026-10-27. ACORN declares Rust 1.98.0 in the workspace and selects stable through rustup. | Record both exact toolchains. A Blaze success cannot establish ACORN's MSRV or stable compatibility. |
| The headline speedup predicts ACORN gains | **Unresolved.** The [README benchmark section](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/README.md#how-fast) qualifies its numbers as v1 results; this review measured none. | Compare real ACORN edits and end-to-end test cycles, not advertised maxima. |
| Blaze has no WASM target | **Incorrect at this source commit.** [`Target::parse`, lines 479–499](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/crates/blaze-link/src/target.rs#L479) includes `wasm32-unknown-unknown` and `wasm32-wasip1`. | WASM is a candidate experiment, not a demonstrated Dioxus/wasm-bindgen integration. |
| ACORN release triples can all use Blaze | **Incorrect at this source commit.** That same parser accepts GNU Linux and MSVC Windows but has no musl Linux or Windows GNU case. ACORN's `.cargo/config.toml` and release Make recipes use those missing triples. | Leave Linux musl and Windows GNU release builds, packaging and signing on Cargo. Do not substitute MSVC artifacts for GNU artifacts. |
| v2 categorically has no live execution support | **Incorrect as a source claim; user-facing integration remains unresolved.** [Codegen, lines 1097–1107 and 2108–2114](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/crates/blaze-cargo/src/serve/unit/codegen.rs#L1097) includes live host executable policy and image setup. Older [usage prose](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/docs/usage.md#change-a-running-program) still describes missing v2 patches. | Do not copy the older `blaze-live` integration recipe or promise state-preserving reload. Start with explicit rebuild/restart. |
| A Blaze check replaces rustc validation | **Incorrect.** The current [borrow-checker description](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/docs/borrowck.md#refusing-builds) acknowledges accepted programs rustc rejects. Its newer scoring also supersedes older overview percentages. | Cargo check/tests and official Clippy remain authoritative. No compiler-error snapshots are regenerated merely to accommodate Blaze. |
| Cargo plugins automatically work through Blaze | **Incorrect.** [CLI argument tests](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/crates/blaze-cargo/src/cli/args/tests.rs#L115) reject a separate `clippy` command; command documentation describes unknown-command refusal rather than plugin forwarding. | Keep nextest, tarpaulin, fuzz, audit, packaging and cross-compilation tooling on Cargo. |
| A matching command name guarantees equivalent environment and artifacts | **Unresolved.** Source compilation uses a different frontend, linker and execution model. [`makes`, lines 60–99](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/crates/blaze-cargo/src/serve/unit/codegen.rs#L60) distinguishes library and executable outputs and rejects some explicitly requested library types. | Test artifact discovery, subprocesses, compiler diagnostics, panics and exit codes explicitly. |

Upstream documentation has internally inconsistent command and platform summaries. Use the pinned executable's help and actual probes during BLZ-01; treat unsupported flags as failed qualification rather than silently falling back. Do not infer absence from stale prose or completeness from an enum case.

# Workspace review by crate

Library crates are judged as reusable API/compiler consumers; CLI, web, TUI and daemon are application roots. Rankings describe expected opportunity and qualification order, not defects in existing ACORN code. Every row has a proposed use, a limiting factor and a concrete verification target.

| Package | Best Blaze use and priority | Repository evidence and limiting factor | Qualification |
|---|---|---|---|
| `acorn-diff` | **First probe.** Native unified/JSON Patch unit tests after implementation edits. Low absolute savings may still make it the cleanest control. | [Manifest](../../../crates/acorn-diff/Cargo.toml), [root](../../../crates/acorn-diff/src/lib.rs), `src/unified/mod.rs`, `src/json_patch/mod.rs`: small serde/similar graph, no build script, alloc/std variants. | Compare all lib tests and a changed diff result; separately compile alloc-only. Preserve existing patch semantics and property-test coverage. |
| `acorn-core` | **High intermediate value.** URI, similarity, Big-O and scalar tests with downstream invalidation. | [Manifest](../../../crates/acorn-core/Cargo.toml), [root](../../../crates/acorn-core/src/lib.rs), `tests/big_o.rs`, `tests/similarity.rs`, `tests/types.rs`: no_std/alloc surface; default std enables digest/ring and document detection. | Exercise alloc first, then default features. A core function edit must invalidate the selected schema/service consumer. Native success does not prove no_std target portability. |
| `acorn-schema` | **High value, higher compatibility risk.** Validators, standards, codecs and crosswalk tests. | [Manifest](../../../crates/acorn-schema/Cargo.toml), [build script](../../../crates/acorn-schema/build.rs), [portable tests](../../../crates/acorn-schema/tests/portable_semantics.rs): extensive derives; Pomsky files generate `OUT_DIR/patterns.rs`; std adds embedded/native dependencies. | Compare diagnostics and serialized results; edit a Pomsky source and ensure generated patterns change; exercise `alloc`, default std and portable_semantics separately. |
| `acorn-analysis` | **High value.** Supplied-text readability, FAIR and check aggregation tests. | [Manifest](../../../crates/acorn-analysis/Cargo.toml), [root](../../../crates/acorn-analysis/src/lib.rs), [tests](../../../crates/acorn-analysis/src/tests/mod.rs): portable logic with schema/macro dependencies and a readability benchmark. | Compare representative numeric outputs and ordered findings; test default and alloc-only. Do not use Blaze timings as release runtime-performance baselines. |
| `acorn-service` | **High value.** Transport-neutral validation, repository and JSON-RPC policy tests. | [Manifest](../../../crates/acorn-service/Cargo.toml), [root](../../../crates/acorn-service/src/lib.rs), [tests](../../../crates/acorn-service/src/tests/mod.rs): caller-supplied data, async contracts and deterministic result checks. | Test duplicate fields, limits, sorted repository outputs and vocabulary policy; default/alloc variants. Useful browser-domain loop without JS runtime. |
| `acorn-macros` | **Compatibility gate.** Positive expansion/consumer rebuilds; limited direct iteration recommendation. | [Manifest](../../../crates/acorn-macros/Cargo.toml), [macro root](../../../crates/acorn-macros/src/lib.rs), `tests/{big_o_compile,validate_compile_fail,with_compile_fail}.rs`: proc-macro crate, `proc-macro-crate` name resolution and trybuild diagnostics. | Test renamed dependency and no_std consumers plus macro-definition edits. Keep Cargo trybuild stderr snapshots authoritative; inspect whether nested tools invoke Cargo/rustc. |
| `acorn-cmd` | **Medium value.** Command construction first; managed process tests after qualification. | [Manifest](../../../crates/acorn-cmd/Cargo.toml), [tests lines 23–38](../../../crates/acorn-cmd/tests/execution.rs): helper explicitly invokes `rustc`; cancellation tests spawn children and sleep. | Compare stdout/stderr, limits, exit status and descendant termination. Record helper compilation separately; Blaze cannot remove deliberate waits or automatically accelerate that rustc subprocess. |
| `acorn-host` | **Medium/high value.** Config/fs/HTTP adapter tests with precise feature sets. | [Manifest](../../../crates/acorn-host/Cargo.toml), [root](../../../crates/acorn-host/src/lib.rs), `src/tests/`, `src/auth/tests/`: default features empty; auth/download/source add runtime and network stacks. | Begin with config; qualify fs/http using temporary directories and loopback. Explicitly enable each surface so an empty default build is not mistaken for adapter coverage. Auth remains fixture-only. |
| `acorn-lib` | **Highest potential native payoff.** Repeated analysis, database, document and MCP leaf-test builds. | [Manifest](../../../crates/acorn-lib/Cargo.toml), [root](../../../crates/acorn-lib/src/lib.rs), `src/io/database/tests/`, `src/io/mcp/tools/tests/`: default std excludes most application services. `analysis` enables a substantial host graph including SQLite, archives, HTTP and templates; optional DuckDB/Polars/PDF expand it further. | Benchmark `--features analysis` and selected MCP/powerpoint features explicitly. Prove native build scripts, embedded inputs, database output and loopback behavior. Do not call a default-only test a full library pilot. |
| `acorn-cli` | **Highest potential end-user developer payoff after library qualification.** Command edits and restart-based `acorn` runs. | [Manifest](../../../crates/acorn-cli/Cargo.toml), [build script](../../../crates/acorn-cli/build.rs), [diff tests](../../../crates/acorn-cli/tests/diff.rs), [help tests](../../../crates/acorn-cli/tests/help.rs): broad always-enabled library features, optional PDF/TUI, embedded assets and assert_cmd binary discovery. | Begin with `--no-default-features`; this still includes a large native graph. Verify correct rebuilt binary and exit/output contracts, then default PDF/TUI compilation without launching downloads or external services. |
| `acorn-server` | **Medium/high value, gated.** HTTP/gRPC request logic and loopback tests. | [Manifest](../../../crates/acorn-server/Cargo.toml), [build script](../../../crates/acorn-server/build.rs), [root](../../../crates/acorn-server/src/lib.rs): vendored protoc, tonic codegen, generated includes, TLS and Tokio. | A `.proto` edit must regenerate Rust and alter the tested contract; test startup/shutdown and both protocols locally. Do not alter codegen to hide tool incompatibility. |
| `acorn-daemon` | **Lower present payoff.** Configuration tests and controlled restart checks. | [Manifest](../../../crates/acorn-daemon/Cargo.toml), [root](../../../crates/acorn-daemon/src/lib.rs): `run_foreground` currently loads configuration and waits for Ctrl-C; it does not yet run the planned durable job system. | Test parsing, configuration failure and termination in an isolated foreground process. Do not install a service or claim state-preserving reload of future job execution. |
| `acorn-tui` | **Medium value.** Rendering/event edits followed by restart with fixture state. | [Manifest](../../../crates/acorn-tui/Cargo.toml), [entry point](../../../crates/acorn-tui/src/main.rs), [run_with_options](../../../crates/acorn-tui/src/lib.rs): Ratatui/Crossterm, shared host graph, raw terminal mode and cleanup. | Use a disposable PTY; prove normal/error/signal terminal restoration and deterministic fixture state. Stateful hot patching is deferred until separately demonstrated. |
| `acorn-wasm` | **Research only for direct compilation; high indirect value through its portable dependencies.** | [Manifest](../../../crates/acorn-wasm/Cargo.toml), [binding source](../../../crates/acorn-wasm/src/lib.rs): cdylib/rlib, wasm-bindgen 0.2.128, serde JS conversion and wasm-bindgen-test. | Require actual wasm-bindgen processing and nine-operation runtime parity. Native lib tests and recognition of the target triple are insufficient. Keep wasm-pack and portability gates. |
| `acorn-web` | **Defer pipeline replacement.** Native presentation/session contract tests may be useful after build-script qualification. | [Manifest](../../../crates/acorn-web/Cargo.toml), [build script](../../../crates/acorn-web/build.rs), [dev coordinator](../../../crates/acorn-web/scripts/dev.py), [Makefile](../../../Makefile): Dioxus 0.7.3, Manganis, generated asset writes, git/Python skill staging and Vite map Fast Refresh. | Test staging permissions and artifact paths first. Any future experiment must preserve Dioxus rebuilds, RSX updates, map refresh, assets, both servers' cleanup and browser runtime semantics. |
| `acorn-downstream-host-fixture` | **Compatibility sentinel, not a performance target.** | [Manifest](../../../tests/downstream-host/Cargo.toml), [consumer](../../../tests/downstream-host/src/main.rs): renamed `acorn-cmd` dependency, public API composition, files and git subprocess. | Build/run to verify downstream macro paths and exported host types. Preserve `make check-host`; include this row in compatibility evidence but not product speedup totals. |
| `xtask` | **Low priority.** Pure version/archive/checksum unit tests only. | [Manifest](../../../xtask/Cargo.toml), [dispatch](../../../xtask/src/main.rs): mixes local checks with release mutation and uploads; Cargo alias expands to `run --package xtask --`. | Use explicit `blaze test -p xtask`, subject to qualification. Keep publication, version changes and packaging under existing Cargo commands; do not assume `blaze xtask` alias behavior. |

The separate [fuzz package](../../../fuzz/Cargo.toml) uses libfuzzer-sys and cargo-fuzz. Keep its nightly sanitizer/fuzz workflow unchanged; no Blaze replacement or speed claim is justified. The same exclusion applies to Criterion runtime baselines, release-size gates and generated documentation acceptance.

# Prioritized adoption findings

These findings constrain a proposed integration; they are not newly discovered production Rust defects. Rules come from the required [guidelines skill](../../../.agents/skills/guidelines/SKILL.md) and [Rust best practices](../../../.agents/skills/rust-best-practices/SKILL.md).

## Must preserve before any integration

1. **Independent correctness authority.** Rule: Rust best practices §2 and §11, compiler/lint verification and observable tests. Locations: `rust-toolchain.toml:2`, `Cargo.toml` workspace lints, `Makefile:339`, `Makefile:446`. A different frontend is not a drop-in correctness gate. Concrete integration: add separate opt-in tasks only; keep ordinary Cargo check, official Clippy, nextest, docs, portable/WASM and release checks. Never accept an altered trybuild snapshot to mask compiler disagreement.
2. **Target and frontend ownership.** Rule: guidelines §3, surgical changes. Locations: `.cargo/config.toml:11`, `Makefile:264`, `Makefile:498`, `crates/acorn-web/scripts/dev.py:33`. Do not change musl/GNU release targets, replace dx with Blaze, or rewrite browser staging for the pilot. A browser feasibility failure is a recorded non-adoption, not a reason to redesign the app.
3. **Generated-input and macro invalidation.** Rule: Rust best practices §11, test observable behavior. Locations: `crates/acorn-schema/build.rs:17`, `crates/acorn-server/build.rs:3`, `crates/acorn-cli/build.rs:4`, `crates/acorn-web/build.rs:4`. Test changes to Pomsky, proto, assets, macro definitions and environment inputs. A successful no-change rebuild does not prove the build is fresh.

## Should establish before recommending a developer command

4. **Representative feature scope.** Rule: guidelines §4, verifiable success criteria. Locations: `crates/acorn-lib/Cargo.toml` feature table, `crates/acorn-host/src/lib.rs:5`. Use named package/feature selections and enumerate tests actually executed. Separate default, alloc, analysis and optional native-heavy cases; never advertise workspace-wide gains from `acorn-diff` alone.
5. **Process/test-tool compatibility.** Rule: Rust best practices §11, mock external effects and test behavior. Locations: `crates/acorn-cmd/tests/execution.rs:29`, `crates/acorn-macros/tests/validate_compile_fail.rs:3`, `crates/acorn-cli/tests/help.rs:13`. Qualify child compiler calls, executable discovery, exit codes and filtered test execution. Start without affected-test caching; `--reached` is a later experiment requiring full-run comparison after code, fixture and environment changes.
6. **Account for startup and resource cost.** Rule: Rust best practices §12, measure before optimization. Locations: `Cargo.toml` dev/test package optimization tables, `Makefile:286`, `Makefile:446`. Record cold cost, daemon lifetime/RSS, warm edit latency and runtime cost independently. Keep profiles identical in the first comparison. Coverage's forced clean stage and tests' sleeps are separate problems already owned by COVT.

## Consider after evidence

7. **Optional host-local usage, narrow allowlist.** Rule: guidelines §2, simplicity. Prefer explicit `blaze test -p …`/`blaze run -p …` recipes for proven rows. Add Make conveniences only if qualification demonstrates a practical win. No abstraction across all build systems, automatic installation, repository-wide wrapper, Cargo dependency, broad sandbox exception or production live-patching library is needed.

What's already suitable: portable domain crates separate supplied-data work from host effects; feature gates allow smaller experiments; inline, integration and downstream-consumer tests expose useful contracts; Dioxus/Vite already own their distinct reload paths. Preserve these boundaries rather than restructuring crates around a compiler experiment.

# Measurement and acceptance contract

## Reproducible experiment

Use a disposable copy/worktree of a frozen revision, not the active browser worktree. Capture the exact ACORN revision and applied patch, Cargo.lock digest, compiler versions, Blaze binary checksum/source relation, host/OS/CPU/RAM, filesystem, job count, environment, feature selection, optimization goal, cache paths and command line. Keep Cargo and Blaze outputs separate; do not reuse `target/` as a shared cache. Verify actual output paths because Blaze has its own cache/build placement. Keep logs free of credentials.

Blaze's [build/cache documentation](https://github.com/indexable-inc/blaze/blob/39452df1ec738b68fbec6ff6e0d8919839d45355/docs/cargo.md#where-a-build-writes) specifies separate source/build storage and filesystem cloning requirements. Preflight free disk and clone support; record toolchain downloads separately from compilation. An isolated `BLAZE_HOME` does not necessarily isolate its toolchain cache: inspect the pinned version's actual paths. Its macro execution and native build-script sandbox differ from Cargo; qualify all file/process inputs instead of disabling restrictions globally.

For each admitted package run the same selected tests under both tools. Record test identities, counts, ignored tests, pass/fail results, output and process provenance. Enumerate filters before timing and reject zero-test runs. Deliberately change expected behavior in the disposable copy, prove the affected test fails, restore it and prove it passes. Include compile-invalid code as a diagnostic check; do not run invalid code even if Blaze accepts it.

Measure these distinct states, interleaving tool order to limit host-load bias:

- First build with fresh compiler output, dependencies/toolchains already provisioned; report downloads independently.
- Warm no-change invocation.
- Warm private function-body change; warm public type/signature change in a dependency and consumer.
- Macro-definition change and generated input change, plus fixture-only and relevant environment changes.
- Daemon restart with persisted cache, then at least 20 alternating edit/test cycles for memory and stale-result checks.

Take at least ten samples for the main warm-edit case; retain raw samples, median and p95. Measure end-to-end command latency as well as build-only time, runtime test cost, peak/resident daemon memory and disk use. For long cases, cap each run and the phase's active time, reporting incomplete qualification instead of silently reducing the test set. Do not compare a warm cached-test answer with a cold full Cargo run.

Proposed adoption threshold, to freeze before timing: zero semantic/freshness mismatches; at least **2× median warm-edit speedup and 1 second saved** in two representative native workloads; p95 no worse than Cargo; total cold-plus-20-edit session faster; no sustained memory growth after warmup and enough RAM to avoid swapping alongside the editor/browser. These are decision thresholds, not measured findings. A smaller pure crate can pass compatibility without meeting the performance threshold. Record a per-host resource ceiling before running, rather than inventing one from upstream examples.

## Candidate commands

These are proposed probes, **not tested instructions**. BLZ-01 must check the pinned executable supports their flags. Pair every line with the identical `cargo` invocation, excluding Blaze-only flags. Run from the disposable repository root.

```sh
blaze test --locked -p acorn-diff --lib
blaze test --locked -p acorn-core --no-default-features --features alloc --test big_o
blaze test --locked -p acorn-schema --no-default-features --features alloc --test portable_semantics
blaze test --locked -p acorn-analysis --lib
blaze test --locked -p acorn-service --no-default-features --features alloc --lib
blaze test --locked -p acorn-host --features config --lib
blaze test --locked -p acorn-lib --features analysis --lib
blaze test --locked -p acorn-cli --no-default-features --test diff
blaze run --locked -p acorn-cli --no-default-features --bin acorn -- --help
```

After initial discovery, narrow large test runs to a recorded nonempty selection. Prove package selection/feature resolution match Cargo before interpreting timing. Do not use `--all-features` as the first pilot: it adds optional native backends and obscures failure attribution.

## Go/no-go and rollback

Admit a package/feature/host combination only after semantic, invalidation and resource gates pass. List unqualified combinations explicitly. A reproducible compiler error on Cargo-valid code, stale output, wrong subprocess, unsupported target or inadequate performance is a no-go for that combination. Preserve a minimal reproduction; do not fix ACORN behavior to satisfy a compiler bug in this plan.

If no representative workload qualifies for Blaze, publish a short no-go record and prune Blaze-only integration; this does not cancel the independently useful Cargo-first restart loop. If some qualify, document the exact version, commands, limitations and fallback to ordinary Cargo. Rollback is stopping the dedicated pilot daemon and removing the optional wrapper/Make entries; no source/API/schema/release conversion should require reversal. Never clean unrelated user caches or stop another developer's daemon.

# Dependencies and ownership

BLZ-01 → BLZ-02 → BLZ-03 → BLZ-04 → BLZ-05 → BLZ-06. BLZ-07 follows BLZ-05; BLZ-08 follows BLZ-03. BLZ-09 requires a passing native representative workload from BLZ-05/06 and the finalized compatibility allowlist; browser success is not required. BLZ-11 follows the BLZ-01 recording harness and independent Cargo binary-launch checks for a Cargo-first supervised development loop; Blaze is admitted only after BLZ-06/07 qualification. BLZ-12 follows BLZ-11 and ACD-08A for real daemon reload/restart acceptance. BLZ-13 follows BLZ-02 for an isolated live-function comparison. BLZ-10 accepts or rejects the resulting workflow after every retained branch is resolved; negative results may prune branches without implementation. Negative findings can prune later dependent phases immediately after durable recording.

- [Daemon service](../../features/acornd-service.md) owns ACD-08A job reload and runtime recovery. BLZ-11 fake-process supervisor development need not wait for the scheduler; BLZ-12 real acceptance waits for ACD-08A. The optional compiler/hot-function branches are not daemon shipping prerequisites.
- [Coverage throughput](../../testing/coverage-throughput.md) owns cleaning, coverage profiles and slow-test policy. It is independent; baseline again after its changes land. Blaze does not replace tarpaulin or resolve its forced clean stage.
- [Mise integration](../../migrations/mise-toolchain-integration.md) owns broader tool provisioning. Record a Blaze pin but add no installer/backend under this plan. BLZ-09 serializes with mise and all other CI/build owners.
- [Frontend quality](../../testing/frontend-build-quality.md) and [browser workspace](../../features/acorn-browser-workspace/index.md) retain asset staging, minification, HMR and desktop assembly. BLZ-08 is isolated evidence only and blocks none of their phases.
- [Dependency audit](../workspace-dependency-audit.md) retains Cargo.lock and dependency updates. Benchmark a frozen graph; invalidate the baseline if it changes. No whole-plan dependency on completing the audit.
- No new runtime dependencies, manifest/profile changes or workspace-graph lock are requested. If qualification requires any, stop that experiment and author a separately bounded repair proposal.

All phases reserve this plan for lifecycle updates. Serialize root/nearest index, network and lane maintenance at handoff. Preserve existing user changes. Future experimental source mutations occur only inside the named disposable copy, never the shared checkout.

# Execution phases

## BLZ-01 — Pin the executable and establish a viable host

- Status: ready
- Budget: 5 active hours maximum
- Prerequisites: dedicated macOS arm64 pilot location, disk/memory budget and installable binary whose source revision is known; setup/download waits are external prerequisites
- Exact write set: `scripts/blaze-pilot.py`, `scripts/test-blaze-pilot.py`, `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/qualification.json`; external system: dedicated disposable local Blaze pilot checkout and cache
- Primary integration lock: Blaze pilot tooling
- Outcome: Explicit pinned invocation, isolated output directories, command/time/resource recording and failure classification. Resolve help/source/release discrepancies and build/test `acorn-diff` once under both compilers. No shared user config changes.
- Focused validation: Harness tests for failed commands, timeout, zero-test detection and log capture; successful and deliberately failing diff test in the disposable checkout; lockfile unchanged.
- Handoff: Record binary provenance, host constraints and commands. If even this graph fails, record the minimal failure and prune Blaze-only integration phases; retain Cargo baseline evidence for the independent restart loop.

## BLZ-02 — Measure the portable domain loop

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-01
- Exact write set: `scripts/blaze-pilot.py`, `scripts/test-blaze-pilot.py`, `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/portable.json`; disposable checkout mutations limited to `crates/acorn-diff/src/unified/mod.rs`, `crates/acorn-core/src/big_o/mod.rs`, `crates/acorn-analysis/src/tests/mod.rs`, `crates/acorn-service/src/tests/mod.rs`
- Primary integration lock: Blaze pilot tooling
- Outcome: Bounded representative test matrix for diff, core, analysis and service, with one cross-crate invalidation experiment and cold/warm measurements.
- Focused validation: Matching test inventories/results, default versus alloc selection, invalid-code refusal and repeated edit/fail/restore cycles. Split an unfinished matrix row before expanding the five-hour scope.
- Handoff: Publish per-row results; do not extrapolate to native application dependencies.

## BLZ-03 — Qualify schema generation and procedural macros

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-02
- Exact write set: `scripts/blaze-pilot.py`, `scripts/test-blaze-pilot.py`, `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/generated-inputs.json`; disposable checkout mutations limited to `crates/acorn-schema/assets/pomsky/doi.pomsky`, `crates/acorn-macros/src/big_o.rs`, `tests/downstream-host/src/main.rs`
- Primary integration lock: Blaze pilot tooling
- Outcome: Schema portable_semantics, Pomsky regeneration, macro consumer invalidation and renamed-dependency results; document trybuild/nested compiler limitations without changing snapshots.
- Focused validation: Cargo/Blaze comparison before/after each input edit, generated constants checked, downstream consumer built/run, Cargo compile-fail suites retained unchanged. Resolve exact fixture path against the current tree before applying a disposable patch.
- Handoff: Record supported macro/build-script behavior; denied macro reads or stale output block affected consumers.

## BLZ-04 — Qualify native process and host adapters

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-03
- Exact write set: `scripts/blaze-pilot.py`, `scripts/test-blaze-pilot.py`, `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/host.json`; external system: disposable host pilot with temporary files and loopback ports
- Primary integration lock: Blaze pilot tooling
- Outcome: `acorn-cmd`, selected `acorn-host` config/fs/http tests and downstream fixture behave consistently; child rustc and subprocess costs are separated.
- Focused validation: Nonzero test inventory, stdout/stderr/status, cancellation/descendant cleanup, temporary-directory isolation and no remote network requirement. Record auth/download/source features not exercised.
- Handoff: Admit only tested host capabilities; do not broaden sandbox permissions to make a failing row green.

## BLZ-05 — Measure the heavy native library workload

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-04; native dependency toolchain available in the isolated host
- Exact write set: `scripts/blaze-pilot.py`, `scripts/test-blaze-pilot.py`, `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/library.json`; disposable mutations limited to `crates/acorn-lib/src/io/database/tests/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/tests/diff.rs`
- Primary integration lock: Blaze pilot tooling
- Outcome: Real `acorn-lib` analysis-enabled build/test timings and output parity, including bundled native-library linkage and a bounded MCP test selection. Optional DuckDB/dataframe/PDF remain separately unqualified unless explicitly tested within budget.
- Focused validation: Compare fixture database/document/MCP outputs, test identity and invalidation; at least ten warm-edit samples and 20-cycle resource observation on the selected workload.
- Handoff: First representative adoption decision; preserve failure data rather than weakening the feature scope.

## BLZ-06 — Qualify CLI and maintenance-tool test workflows

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-05
- Exact write set: `scripts/blaze-pilot.py`, `scripts/test-blaze-pilot.py`, `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/cli.json`; disposable mutations limited to `crates/acorn-cli/tests/diff.rs`, `crates/acorn-cli/assets/skills/acorn/SKILL.md`
- Primary integration lock: Blaze pilot tooling
- Outcome: Non-default and default CLI compilation, selected assert_cmd tests, rebuilt embedded-input observation, and bounded xtask unit tests. Establish restart-based command latency without publication or external downloads.
- Focused validation: Verify executable path/provenance, help/diff exit codes and outputs, macro/generated server dependencies, no hidden stock-Cargo build included as Blaze work, no mutating xtask subcommands.
- Handoff: Explicit CLI feature allowlist and measured command recipe; unresolved PDF/TUI behavior stays unqualified.

## BLZ-07 — Evaluate server, daemon and terminal consumers

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-05; isolated loopback ports and disposable PTY available
- Exact write set: `scripts/blaze-pilot.py`, `scripts/test-blaze-pilot.py`, `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/applications.json`; disposable mutation limited to `crates/acorn-server/proto/acorn/service/service.proto`
- Primary integration lock: Blaze pilot tooling
- Outcome: Bounded compatibility probes for protoc regeneration, server local request/shutdown, daemon config/termination and TUI startup/cleanup. No installed service or stateful live patching. Freeze subprocess launch, artifact staging and shutdown interfaces for BLZ-11; current config-shell results are not scheduler/reload acceptance.
- Focused validation: One nonempty test/filter or controlled process per consumer, matching results and terminal restoration. This is compatibility sampling; do not promise exhaustive protocol/UI or performance acceptance.
- Handoff: Mark each consumer qualified or explicitly unresolved; split any deeper consumer work into its own phase.

## BLZ-08 — Decide the WASM and browser boundary

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-03; existing Dioxus/wasm-bindgen/Vite tools and browser test engine available
- Exact write set: `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/browser.json`; external system: separate disposable browser pilot checkout, cache and ports
- Primary integration lock: Blaze browser feasibility
- Outcome: Determine whether the pinned binary can compile acorn-wasm and survive wasm-bindgen processing, and whether acorn-web asset staging is permitted. Record Dioxus command/protocol integration feasibility; keep `make web-dev` unchanged.
- Focused validation: Existing nine-operation runtime tests if compilation succeeds, generated assets/exports and one-WASM contract. If a Dioxus path is feasible without source redesign, compare RSX/function edits and Vite map refresh. Failure or incomplete HMR proof yields explicit non-adoption.
- Handoff: Document native-domain testing as the supported browser-adjacent alternative. A browser integration needs a new bounded implementation phase with proven commands before any shared pipeline edit.

## BLZ-09 — Add only the qualified native convenience tasks

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: passing BLZ-05/06 performance and correctness gates, reviewed compatibility allowlist, CI/build lock available
- Exact write set: `Makefile`, `scripts/blaze-pilot.py`, `scripts/test-blaze-pilot.py`, `docs/src/development/blaze.md`, `docs/src/SUMMARY.md`
- Primary integration lock: CI/build
- Outcome: Optional `make blaze-test` and `make blaze-run` recipes constrained to qualified selections, with explicit package/features and version checks, no auto-install and clear failures. Document ordinary Cargo fallback and daemon cleanup. Add no target if the pilot fails adoption criteria.
- Focused validation: Dry-run command/quoting checks, missing/wrong binary failure, invalid/unqualified selection failure, selected real smoke test, and unchanged recipes for test/lint/cover/web-dev/build/release.
- Handoff: Publish exact supported version/host/feature combinations and resource guidance. Global Cargo aliases, wrappers, manifests and CI remain untouched.

## BLZ-11 — Build the isolated incremental restart supervisor

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-01 recording harness and verified independent launch of a Cargo-built binary pair; Blaze success and the actual daemon scheduler are not required for fake-process tests
- Exact write set: `scripts/acorn-dev.py`, `scripts/test-acorn-dev.py`, `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/restart-supervisor.json`; external system: disposable macOS arm64 foreground pilot only
- Primary integration lock: Blaze pilot tooling
- Outcome: Implement the [generation/restart contract](development-loop.md#binary-generations-and-safe-restart) with Cargo as baseline and an explicit qualified Blaze backend. Debounce/coalesce source edits, build matching binaries in isolated outputs, run a fixed focused test selection, stage immutable generations, and perform bounded stop/start/readiness with rollback. Keep the last working process on compile/test failure. No daemon crate or production install edits.
- Focused validation: Fake builder/process tests for failed/partial pair builds, duplicate edits, superseded candidates, hung drain, candidate readiness failure, rollback failure and Ctrl-C cleanup. Verify independently launchable staged artifacts before admitting a backend and record phase timings.
- Handoff: A usable isolated restart supervisor and tested failure contract; do not claim real scheduler/history acceptance until BLZ-12.

## BLZ-12 — Accept daemon reload and the complete development loop

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-11, ACD-08A; use BLZ-06/07 evidence only when admitting the optional Blaze backend; daemon recovery/worker/listener contracts accepted; disposable local store/config/ports; CI/build lock available
- Exact write set: `Makefile`, `scripts/acorn-dev.py`, `scripts/test-acorn-dev.py`, `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/daemon-loop.json`; external system: isolated foreground daemon and fixture workspace
- Primary integration lock: CI/build
- Outcome: Add optional `make daemon-dev` only after actual paired-binary restart and authenticated configuration reload pass the [acceptance matrix](development-loop.md#acceptance-matrix). Keep Cargo as usable default backend and require explicit qualified Blaze selection. Optional config watching invokes the existing reload API; it never implements its own scheduler/config semantics.
- Focused validation: Code save to changed fixture result, config save without PID change, active-job reload, durable pause/history after restart, superseded saves, compile failure, readiness rollback and process cleanup. Compare complete Cargo/Blaze edit-to-result median/p95 and resource use on the same fixture; a current config-only shell cannot satisfy this phase.
- Handoff: Publish timings, observed recovery semantics and backend limits. Runtime defects return to a bounded ACD repair phase. If Blaze fails, retain only a passing Cargo development loop and explicitly record non-adoption.

## BLZ-13 — Compare isolated function hot-reload mechanisms

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-02; pinned Hotcode and Blaze documentation/API review; disposable harness with fixed scalar ABI
- Exact write set: `docs/src/development/blaze.md`, `.wiki/refactors/blaze-development-pilot/_assets/function-reload.json`; external system: isolated pure-function experiment outside the ACORN workspace
- Primary integration lock: Blaze pilot tooling
- Outcome: Compare Cargo + Hotcode and independently Blaze-native live-code support with rebuild/restart using one pure synchronous scalar function. Verify whether the chosen binaries actually support the mechanism; do not stack Hotcode's dylib requirement on an incompatible Blaze linker. Add no ACORN manifest dependency, annotation or crate-type change.
- Focused validation: Body-only edit changes observed output with stable process identity; fixed ABI, no retained references/state, safe refusal or restart before incompatible changes, repeated-update memory/latency and runtime overhead. Do not execute ABI-invalid variants. Unsupported tooling is a valid no-go finding.
- Handoff: Record one recommendation or explicit deferral, distinct from production config reload. Any proposed application integration requires a new bounded phase with an accepted state/API boundary; it does not silently expand this experiment.

## BLZ-10 — Accept the workflow and retire the plan

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: BLZ-09 accepted or pruned with durable no-go evidence, and each retained BLZ-11/12/13 outcome accepted or explicitly pruned after durable no-go evidence; all active pilot processes stopped; serialized validation slot
- Exact write set: `docs/src/development/blaze.md`, `docs/src/SUMMARY.md`, `docs/src/development/_assets/blaze-qualification.json`, this plan, `development-loop.md` and `_assets` for retirement, `.wiki/features/acornd-service.md`, `.wiki/refactors/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: developer workflow acceptance
- Outcome: Durable measured findings and support limits, plus verified opt-out behavior. Preserve the shared reload contract in `docs/src/development/blaze.md` and replace the daemon plan’s inbound links before retirement. Consolidate useful raw results into `docs/src/development/_assets/blaze-qualification.json` before deleting plan assets. No wider platform claim without a separate native-host phase.
- Focused validation: Optional commands on a clean disposable checkout; read-only `cargo fmt --all -- --check` and official Clippy as relevant; serialized `make test` for an implemented workflow; wiki and documentation links. If only a no-go report remains, documentation/wiki validation suffices.
- Handoff: Retire the plan and all index/network/lane entries. Preserve Cargo, web and release ownership in durable developer documentation; record any new work as a separate evidence-backed plan.
