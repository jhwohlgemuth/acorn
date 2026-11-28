# Migration Plans

## Waiting on validation

- [Needle 2 to Needle 3 upgrade](needle-3-upgrade.md) - Linux x86_64 passed on Oklahoma and Linux ARM64 passed on Dante; NDL-02 is blocked on GSR-02 because runner 55645 cannot finish a tagged job.

## Planned

- [RADS Rust module migration](rads-module-migration/index.md) — Move the public schema tree to `standard/rads`, update consumers, and align private source and build-target names.
- [acorn-web shadcn-style component transition](acorn-web-shadcn-components.md) — Prove locally bundled UI SDK Web Components in Dioxus, use unstyled UI SDK React controls in the map where parity holds, and preserve ACORN's appearance.
- [Mise development-tool integration](mise-toolchain-integration.md) — Use one locked, multiplatform tool inventory for local Make workflows and CI while preserving `rust-toolchain.toml` as the stable Rust contract.
- [Podman-only Makefile migration](podman-makefile.md) — Replace Docker CLI and Buildx recipes with Podman-native build, run, publication, and multi-architecture manifest workflows across Linux, macOS, and Windows.
