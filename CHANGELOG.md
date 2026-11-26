# CHANGELOG 🌱
> Wisdom is knowledge of things that don't change, intelligence is the knowledge of things that do

## [0.7.1](https://code.ornl.gov/research-enablement/acorn/compare/v0.7.0..v0.7.1) - 2026-11-25

### 🐛 Bug Fixes

- Let `acornd` shut down gracefully on systemd SIGTERM so uninstall removes its local RPC socket.

## [0.7.0](https://code.ornl.gov/research-enablement/acorn/compare/v0.6.3..v0.7.0) - 2026-11-25

### 🚀 Features

- Publish signed paired Linux `acorn` and `acornd` bundles for x86_64 and ARM64.
- Verify release signatures and binary hashes with `acorn daemon verify-bundle`; stage and activate a local release with `acorn daemon install --bundle`, including readiness checks and rollback.

## [0.6.2](https://code.ornl.gov/research-enablement/acorn/compare/v0.6.1..v0.6.2) - 2026-11-05

### 🚀 Features

- Release v0.6.2 - ([4e4fef8](https://code.ornl.gov/research-enablement/acorn/commit/4e4fef8b5dd4abd846ef3cbeaa408f4abd892355))

### 🐛 Bug Fixes

- *(ci)* Publish crates in dependency order and validate ordering - ([091ade5](https://code.ornl.gov/research-enablement/acorn/commit/091ade5534730c9fb8d028f4f59a7de9f9b26d35))

