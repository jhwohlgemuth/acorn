[[_TOC_]]

# Getting Started
- Read this guide (including the [Code of Conduct](CODE_OF_CONDUCT.md))
- Fork this repository and clone your fork locally
- [Setup your environment](#project-setup) to use development tasks (test, lint, etc...)
- Adhere to the [project standards](#project-standards)
- Write some code and stuff...
- Push your changes and create a pull request

## Code of Conduct
> Please read and adhere to the [code of conduct](CODE_OF_CONDUCT.md)

## Introduction
> First off, thank you for considering contributing to `ACORN`!

If you would like to make a feature request or enhancement suggestion, please open an issue.

If you would like to generously provide a pull request to correct a verified issue, please adhere to this project's [standards](#project-standards). Before making a pull request for a desired feature, please socialize the concept by opening an issue first.

## Project Architecture
> See [ARCHITECTURE.md](./ARCHITECTURE.md)

## Project Setup
- When on Windows, execute `./setup.bat` to install and configure OpenSSL before trying to use any [development task](#development-tasks)
- Install task dependencies with
  ```shell
  cargo install cargo-nextest cargo-tarpaulin cargo-audit cargo-auditable sqruff
  ```
- Install the [Buf CLI](https://buf.build/docs/cli/installation/) before running `make lint` (`brew install bufbuild/buf/buf` on macOS or `scoop install buf` on Windows).

### Development Tasks
| Purpose                                  | Command            |
| ---------------------------------------: | ------------------ |
| Clean code artifacts                     | `make clean`       |
| Build code artifacts                     | `make build`       |
| Lint **ALL** code                        | `make lint`        |
| Lint **ALL** code and run **ALL** tests  | `make lint test`   |
| Run **ALL** tests                        | `make test`        |
| Run **ALL** tests with coverage          | `make cover`       |
| Run benchmarks                           | `make bench`       |
| Save benchmark baseline                  | `make bench-baseline` |
| Compare to saved baseline                | `make bench-compare` |
| Generate a benchmark history report      | `make bench-report` |
| Run security audit on dependencies       | `make audit`       |
| Generate documentation                   | `make doc`         |
| Build the documentation book              | `make book`        |
| Regenerate command-line help              | `make help`        |
| Generate CHANGELOG<sup>[[1]](#1)</sup>   | `make changelog`   |

Container image builds use `docker.io/library` for Docker Official Images by default. ORNL developers can avoid Docker Hub throttling by using the institutional proxy cache:

```shell
make acorn-image DOCKER_HUB_REGISTRY=regproxy.ornl.gov/hub_proxy
```

### Benchmarking

ACORN uses Criterion benchmarks for repeatable micro-performance measurements in `crates/acorn-lib/benches/`.

Run all library benchmarks:

```shell
make bench
```

Run only the research activity benchmark target:

```shell
cargo bench --package acorn-lib --bench research_activity
```

Run only the readability benchmark target:

```shell
cargo bench --package acorn-lib --bench readability
```

During local iteration, reduce runtime while preserving relative signal:

```shell
cargo bench --package acorn-lib --bench research_activity -- --sample-size 10
```

Analyze benchmark performance over time by saving and reusing Criterion baselines:

```shell
make bench-baseline
make bench-compare
```

Use a named baseline for branch or release tracking:

```shell
make bench-baseline BENCH_BASELINE=release-0.1.56
make bench-compare BENCH_BASELINE=release-0.1.56
```

Criterion stores benchmark history and comparisons under `target/criterion/`, including HTML reports and baseline deltas.

Generate normalized JSON, CSV, and a self-contained HTML report from the current Criterion results:

```shell
make bench-report
```

To append the current results to a previously downloaded history, provide its path explicitly:

```shell
make bench-report BENCH_HISTORY=previous-history.json
```

The report is written to `target/benchmark-results/`. GitLab CI runs `make bench-baseline` for every semantic version tag and after relevant default-branch library or benchmark changes, saving the raw Criterion baseline under the workspace version. It uploads an immutable `acorn-benchmarks` generic package containing `run.json`, cumulative `history.json`, `history.csv`, `report.html`, and the raw Criterion results. Release package versions are prefixed with the ACORN tag, and each normalized run records the project version. Pipeline artifacts expire after 30 days; the package registry is the persistent source of record and must be excluded from package cleanup policies.

Timing comparisons are grouped by runner ID, architecture, CPU model, and logical core count. Results from different environment fingerprints remain visible but are never used to calculate performance deltas. Toolchain versions are recorded with every run. The report is informational and does not fail CI for regressions.

### PDF Visual Regression Tests

PDF export has visual regression tests for output with and without ASPECT data. These tests are ignored by default because they require a Chromium-based browser and Poppler's `pdftoppm` executable.

Install Poppler for your platform:

- macOS: `brew install poppler`
- Ubuntu or Debian: `sudo apt install poppler-utils`
- Windows: install the [MSYS2 UCRT64 Poppler package](https://packages.msys2.org/packages/mingw-w64-ucrt-x86_64-poppler) with `pacman -S mingw-w64-ucrt-x86_64-poppler`, then add `C:\msys64\ucrt64\bin` to `PATH`

Ensure `pdftoppm -v` succeeds and Chrome or Chromium is available on `PATH`. If the browser is installed elsewhere, set `CHROME_PATH` to its executable. For example, in PowerShell:

```powershell
$env:CHROME_PATH = "C:\Program Files\Google\Chrome\Application\chrome.exe"
```

Run both PDF visual regression tests:

```shell
cargo test -p acorn-cli --all-features visual_snapshot -- --ignored --nocapture
```

The snapshots are stored in `crates/acorn-cli/src/commands/export/tests/snapshots/`. When an intentional PDF layout change produces new snapshots, install the review tool with `cargo install cargo-insta`, inspect the differences, and accept or reject them with:

```shell
cargo insta review
```

### Real adapter interoperability tests

The ACP and OpenAI-compatible interoperability tests use real configured services. They are ignored by default and never run as part of `make test`. Each test uses synthetic input, checks that it remains unchanged, and does not install software or apply a suggested fix.

Run the OpenCode ACP test only when `opencode` is on `PATH`, authenticated, and configured with an available model:

```shell
make test-acp
```

Run the OpenAI-compatible test against a service that supports both `/v1/responses` and `/v1/chat/completions` for the selected model. Set `OPENAI_API_KEY` only when the endpoint requires bearer authentication; the test reads credentials only from that variable.

```shell
# Set OPENAI_API_KEY here when the endpoint requires bearer authentication.
export OPENAI_API_KEY=token
make test-openai OPENAI_BASE_URL=https://provider.example/v1 OPENAI_MODEL=example-model
```

For a local OpenAI-compatible endpoint, [Indigo](https://github.com/jhwohlgemuth/indigo) can start its `llama-swap` service at `http://127.0.0.1:10732/v1`. After starting the Indigo stack and selecting an exposed model, run:

```shell
make test-openai OPENAI_BASE_URL=http://127.0.0.1:10732/v1 OPENAI_MODEL=<model-id>
```

Without the corresponding `ACORN_TEST_*` gate, the ignored test exits before starting a process or making a network request. These tests complement the fake ACP and local HTTP coverage; use them to validate a configured provider before changing adapter behavior.

### SSL / TLS Support

ACORN uses `git2` (via `swhid`) for software heritage identifier computation, which requires OpenSSL for network operations.

- **Windows**: run `./scripts/setup.bat` before building to install and configure OpenSSL.
- **macOS / glibc Linux**: OpenSSL is resolved from the system at link time; no extra steps required.
- **musl Linux** (Alpine, static Docker images): OpenSSL is vendored and statically linked into the binary automatically when building for `target_env = "musl"`. This adds ~2–4 MB to the binary but removes all runtime shared-library dependencies, producing a fully portable static executable.

### Visual Studio Code Configuration
**General Development**
- Install [VSCode](https://code.visualstudio.com/)
- Install [rust-analyzer VSCode extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)..they are all really handy and highly recommended 🤓

**Linux Development within Docker Container**
- [Follow the instructions of this article](https://code.visualstudio.com/docs/remote/containers)

## Fuzz Testing

ACORN includes a dedicated cargo-fuzz harness in [fuzz/Cargo.toml](fuzz/Cargo.toml) with an initial target focused on `ResearchActivity` parsing and validation across JSON and YAML inputs.

Available targets:
- `research_activity`
- `location_repository`

Seed corpora:
- `fuzz/corpus/research_activity`
- `fuzz/corpus/location_repository`

### Install tooling

```bash
make setup
```

### Run fuzzing

```bash
make fuzz
```

Optional runtime arguments can be passed through `FUZZ_ARGS`, for example:

```bash
make fuzz FUZZ_ARGS='-- -max_total_time=60'
```

Choose a non-default target when needed:

```bash
make fuzz FUZZ_TARGET=location_repository FUZZ_ARGS='-- -max_total_time=60'
```

### Reproduce a crash artifact

```bash
make fuzz-repro FUZZ_CRASH='fuzz/artifacts/research_activity/crash-...'
```

## Homebrew Tap Maintenance

- This repository includes a formula generator script:
  ```sh
  ./scripts/homebrew/generate-formula.sh 0.1.56 ./Formula/acorn-cli.rb
  ```
- The GitLab release pipeline can update your tap automatically when a version tag is created.
- The `test-homebrew-macos` job installs and tests the generated formula in an isolated Tart VM after the tagged crate is published. Relevant merge requests expose the same smoke test as a manual job for versions already available on crates.io.
- Required CI variables:
  - `HOMEBREW_TAP_REPO_URL` (authenticated clone URL for your tap repo)
  - `HOMEBREW_TAP_BRANCH` (optional, defaults to `main`)
  - `HOMEBREW_TAP_FORMULA_PATH` (formula file path in the tap repo)

> [!TIP]
> Formula publication and installation testing are separate jobs. The installation job changes only its ephemeral macOS VM.

## Project Standards
- Running `make lint` should not introduce any new lint and all format results should be included in proposed changes
- Running `make test` should not have any failures
- Running `make cover` should not decrease coverage
- Exceptions to any of these standards should be supported by strong reasoning and sufficient effort

### Keeping CLI documentation in sync

The CLI definitions and generated help are the option-level source of truth. Curated command pages explain intent, workflows, and troubleshooting without duplicating every flag.

When `Commands`, a subcommand enum, or an argument struct changes:

1. Run `make help` to regenerate `CommandLineHelp.md`.
2. Update the matching page under `docs/src/cli-reference/commands/`.
3. Add or remove its entry in `docs/src/SUMMARY.md` and the command index.
4. Run `make book`.
5. Include the generated-help and documentation diff in the same merge request as the CLI change.

Ownership follows the code that implements each workflow:

| Commands | Code owner | Documentation |
|---|---|---|
| `check`, `format`, `link`, `export` | RAD workflow module maintainers | Matching command page |
| `gather` | Identifier workflow module maintainers | Gather and persistent identifier lifecycle |
| `download`, `import`, `sync` | Model and I/O workflow module maintainers | Matching command page and model workflows |
| `create`, `serve` | Service and infrastructure module maintainers | Matching command page |
| `doctor`, `skill`, `tui` | CLI experience module maintainers | Matching command page |

The docs check builds the mdBook, verifies relative Markdown targets, requires one page and navigation entry per generated top-level command, and rejects WIP markers in command pages.

## Good First Issues
> See [GOOD_FIRST_ISSUES.md](./GOOD_FIRST_ISSUES.md) for a curated list of beginner-friendly tasks that apply functional programming principles to improve the codebase.

## Running ACORN tasks from source
ACORN's full API is available without installation. Simply use `cargo run --bin acorn -- <commands, options, and flags>`
### Examples
| ACORN Command                                     | Equivalent Command                                      |
| ------------------------------------------------- | ------------------------------------------------------- |
| `acorn help`                                      | `cargo run --bin acorn -- help`                                     |
| `acorn check /path/to/folder -vvvv`               | `cargo run --bin acorn -- check /path/to/folder -vvvv`              |
| `acorn download --buckets /path/to/buckets.json`  | `cargo run --bin acorn -- download --buckets /path/to/buckets.json` |

-------------


**Footnotes**
-------------

[1]
---
> CHANGELOG generation requires settting `GITLAB_TOKEN` in an `.env` file and installing [typos](https://github.com/crate-ci/typos)
