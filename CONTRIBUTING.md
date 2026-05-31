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
  cargo install cargo-nextest cargo-tarpaulin cargo-audit cargo-auditable
  ```

### Development Tasks
| Purpose                                  | Command            |
| ---------------------------------------: | ------------------ |
| Clean code artifacts                     | `make clean`       |
| Build code artifacts                     | `make build`       |
| Lint **ALL** code                        | `make lint`        |
| Lint **ALL** code and run **ALL** tests  | `make lint test`   |
| Run **ALL** tests                        | `make test`        |
| Run **ALL** tests with coverage          | `make cover`       |
| Run security audit on dependencies       | `make audit`       |
| Generate documentation                   | `make doc`         |
| Generate CHANGELOG<sup>[[1]](#1)</sup>   | `make changelog`   |

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
- Required CI variables:
  - `HOMEBREW_TAP_REPO_URL` (authenticated clone URL for your tap repo)
  - `HOMEBREW_TAP_BRANCH` (optional, defaults to `main`)
  - `HOMEBREW_TAP_FORMULA_PATH` (formula file path in the tap repo)

> [!TIP]
> This flow updates the tap repository formula; it does not install `acorn` onto the CI runner.

## Project Standards
- Running `make lint` should not introduce any new lint and all format results should be included in proposed changes
- Running `make test` should not have any failures
- Running `make cover` should not decrease coverage
- Exceptions to any of these standards should be supported by strong reasoning and sufficient effort

## Good First Issues
> See [GOOD_FIRST_ISSUES.md](./GOOD_FIRST_ISSUES.md) for a curated list of beginner-friendly tasks that apply functional programming principles to improve the codebase.

## Running ACORN tasks from source
ACORN's full API is available without installation. Simply use `cargo run -- <commands, options, and flags>`
### Examples
| ACORN Command                                     | Equivalent Command                                      |
| ------------------------------------------------- | ------------------------------------------------------- |
| `acorn help`                                      | `cargo run -- help`                                     |
| `acorn check /path/to/folder -vvvv`               | `cargo run -- check /path/to/folder -vvvv`              |
| `acorn download --buckets /path/to/buckets.json`  | `cargo run -- download --buckets /path/to/buckets.json` |

-------------


**Footnotes**
-------------

[1]
---
> CHANGELOG generation requires settting `GITLAB_TOKEN` in an `.env` file and installing [typos](https://github.com/crate-ci/typos)
