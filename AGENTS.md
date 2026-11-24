# AGENTS.md

ACORN (Accessible Content Optimization for Research Needs) is a Rust CLI and library for maintaining and validating research activity data (RAD). This file holds always-on rules and pointers; load detailed guidance only when a task needs it.

## Skills

| When | Use |
| --- | --- |
| Any code is touched or considered | **Required:** [guidelines](./.agents/skills/guidelines/SKILL.md) |
| `.wiki` plans are created, reviewed, updated, executed, pruned, or retired | **Required:** [planning](./.agents/skills/planning/SKILL.md) |
| ACORN recipes: schema fields, CLI commands, analyzers, export, tests, debugging, unwrap fixes, eserde, logging, feature flags | [acorn-dev](./.agents/skills/acorn-dev/SKILL.md), loading only the reference it routes to |
| Rust work (supplemental) / requested Rust code reviews | [rust-best-practices](./.agents/skills/rust-best-practices/SKILL.md) / [rust-review](./.agents/skills/rust-review/SKILL.md) |

The two Rust skills are copied from [BeautifulRust](https://github.com/morganlinton/BeautifulRust/tree/main/skills) under its [MIT license](./.agents/skills/BeautifulRust-LICENSE). When they conflict with other guidance, follow the user's request, this file, the `guidelines` skill, and the repository's lint and build configuration. Do not apply BeautifulRust's suggested lint configuration or run workspace-wide formatting or fixes solely because a skill recommends them. ACORN's `unsafe` prohibition and existing prelude take precedence.

## Context discipline

- Do not read `progress.md` unless resuming planned work or a draft; then read only its relevant section.
- Read the specific plan, module, or reference a task needs. Avoid repository-wide scans and full-file dumps when a focused read suffices.
- Start a fresh session for unrelated work; carry state through `progress.md` checkpoints rather than conversation history.

## Rules

- No `unsafe` (`unsafe_code = "forbid"`).
- No `?` operators in new Rust code.
- No early returns in new code.
- Every public item has `///` docs (`missing_docs = "deny"`); intra-doc links must resolve.
- No bare `.unwrap()` in production code (`unwrap_used = "warn"`; allowed in tests). See `acorn-dev` errors reference.
- No new warnings from `cargo clippy --workspace --no-deps --all-targets`; code is `cargo fmt` clean.
- Errors use `color-eyre`; user-facing JSON entry points use `eserde::json::from_str`.
- New public structs use `#[serde(deny_unknown_fields)]` where appropriate, `#[validate(...)]` attributes, and `#[validate(nested)]` on nested fields.
- CLI status output uses `Label::*()` from `acorn_host::terminal` with `tracing`, never bare `println!`.
- Network code respects the `offline` flag and uses `acorn-lib` I/O utilities.
- Host-only code is feature-gated (`std`, `net`, and others in `crates/acorn-lib/Cargo.toml`); keep `make check-wasm` passing.
- Update `ARCHITECTURE.md` for structural changes, `README.md` for user-facing features, and this file when a new always-on rule emerges.

## Commands

```bash
make lint          # format and lint
make test          # cargo nextest run
make test-all      # include ignored tests
make check-wasm    # portable acorn-lib check (needs wasm32-unknown-unknown)
make help-cli      # regenerate CommandLineHelp.md after CLI changes
make help          # list all make targets
```

Focused tests: `cargo nextest run --package <crate> <filter>`. Run from source with `cargo run --bin acorn -- <args>`.

## Concurrent implementation

When multiple implementations share a working tree:

- Declare and stay within each assignment's exact write set; do not edit files owned by concurrent work without explicit coordination.
- Do not run workspace-wide formatting while another implementation is editing.
- Run focused tests independently. Serialize final `make lint`, `make test`, and portability gates through one agent after all implementations are stable.
- If a concurrent change temporarily breaks compilation, wait or coordinate with its owner instead of patching their files.
- Respect dependencies and exclusive locks in [.wiki/parallel-execution.md](.wiki/parallel-execution.md).

## Ask before

Stop and ask when requirements are ambiguous, a change would restructure command flow, a trade-off affects performance or memory, a schema change may break compatibility, or required test fixtures do not exist.

## Planning wiki

[.wiki/index.md](.wiki/index.md) is the entry point for refactor, fix, migration, and implementation plans. Before implementing planned work, verify the plan's evidence against current code. The planning skill defines plan metadata, phase rules, pruning, and retirement.

## Workspace

Crates live in `crates/`: `acorn-cli`, `acorn-cmd`, `acorn-core`, `acorn-host`, `acorn-lib`, `acorn-analysis`, `acorn-diff`, `acorn-macros`, `acorn-schema`, `acorn-service`, `acorn-server`, `acorn-daemon`, `acorn-tui`, `acorn-wasm`, and `acorn-web`. Each crate's `Cargo.toml` `description` states its purpose; [ARCHITECTURE.md](ARCHITECTURE.md) explains boundaries.

- Repository: https://code.ornl.gov/research-enablement/acorn
- Documentation: https://acorn.ornl.gov
- License: MIT; version in `[workspace.package].version` of `Cargo.toml`; Rust edition 2021 (set per crate); minimum Rust in `rust-version`.
