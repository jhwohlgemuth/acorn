---
type: Plan
title: "Extract acorn-py into a standalone repository"
description: "Move the Python bindings, their history, documentation, CI, and release lifecycle out of the ACORN workspace."
difficulty: high
impact: high
package: [workspace, acorn-py, ci, docs]
tags: [migration, repository, python, pyo3, maturin, gitlab, ci, release]
status: planned
---

# Goal

Make [`research-enablement/acorn-py`](https://code.ornl.gov/research-enablement/acorn-py) the canonical source, development, CI, documentation, and release repository for the `acorn-py` Python package. Keep ACORN as the canonical home of `acorn-lib` and its Rust release lifecycle, with only an overview of and links to the separately maintained Python bindings.

The extraction must preserve useful file history, keep the installed Python module name `acorn` and PyPI distribution name `acorn-py`, and avoid an interval in which neither repository can test or publish the package.

# Assumptions

- The [`acorn-py` GitLab repository](https://code.ornl.gov/research-enablement/acorn-py) already exists and is the canonical migration destination.
- Remote project provisioning, namespace selection, visibility, and initial ownership are complete and out of scope. The migration does not create, rename, transfer, or delete a GitLab project.
- Push access and CI/CD administration remain cutover prerequisites. Runner access, branch and tag protection, release permissions, and PyPI publishing configuration must still be verified in the existing project.

# Current evidence

- `crates/acorn-py/` currently contains its Rust source, Python tests, `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, Pixi lockfile, README, and a local PyPI publishing Make target.
- The root Cargo workspace includes `crates/acorn-py`, supplies its package version through `version.workspace = true`, and resolves its PyO3 dependencies in the root `Cargo.lock`.
- `crates/acorn-py/Cargo.toml` uses a path dependency on `../acorn-lib`, so the extracted project cannot build until it depends on a published `acorn-lib` version.
- `xtask` treats the binding's `acorn-lib` dependency as an in-workspace version pin. This unnecessarily couples Python package releases to the ACORN workspace version.
- `.cargo/config.toml` contains PyO3-specific macOS linker flags and a Homebrew Python executable override that apply to every ACORN workspace build.
- `.gitlab-ci.yml` owns one `test-python` job. It runs `pixi run test` for merge requests and default-branch changes but does not build a wheel matrix or publish `acorn-py` to PyPI.
- ACORN's crates.io publishing script does not publish `acorn-py`; the only checked-in PyPI release command is the package-local `make pypi` target using `MATURIN_PASSWORD`.
- ACORN's README, architecture, contributor/agent guidance, mdBook sources, and active wiki plans describe `acorn-py` as part of this checkout. The package README and `pyproject.toml` still point source and changelog links at the monorepo.
- The active [Python binding deduplication](../refactors/core/python-binding-deduplication.md) plan concerns only code that will move. The [CI runner pools](../refactors/ci-runner-pools.md), [crate boundary architecture](../refactors/crate-boundary-architecture.md), and [thiserror adoption](thiserror-adoption.md) plans also contain assumptions about Python jobs or bindings inside ACORN.

# Recommendation

Use the existing [`research-enablement/acorn-py`](https://code.ornl.gov/research-enablement/acorn-py) remote as the independent GitLab home for the bindings. Populate it from a history-filtered copy of `crates/acorn-py/`, then make it a single-package Rust/Python repository that consumes `acorn-lib` from crates.io. Do not add a Git submodule, vendored copy, or mandatory cross-project pipeline to ACORN; those would recreate checkout or release coupling.

Use independent semantic versions for the Python distribution and the Rust library. The `acorn-py` manifest should declare the lowest compatible published `acorn-lib` release, while its committed lockfile selects a reviewed concrete dependency set. Updating ACORN's workspace version must no longer update or release `acorn-py`.

Split documentation ownership as follows:

| Concern | Canonical owner after extraction |
|---|---|
| Python installation, supported Python/platform matrix, API examples, development, CI, and release process | `acorn-py` repository |
| Rust API, CLI, ACORN architecture, and ACORN release process | ACORN repository |
| Explanation that Python bindings exist and how they relate to `acorn-lib` | Short ACORN overview linking to `acorn-py` |

# Migration plan

## 1. Confirm cutover invariants

1. Use `https://code.ornl.gov/research-enablement/acorn-py` as the canonical remote URL and treat its namespace, visibility, ownership, and default branch as fixed migration inputs. Verify Git push access without provisioning or changing the project's identity.
2. Inventory the latest PyPI version and supported wheel files before choosing the first standalone version. Use the next unused `acorn-py` version; do not derive it from ACORN's workspace version.
3. Verify that maintainers can push to the remote and can configure runners, protected branches and tags, CI/CD variables, release environments, and the existing PyPI project. These are cutover prerequisites, not remote-creation tasks.
4. Record the currently supported interface with the existing Python tests and a small import/usage smoke test. Preserve the distribution name `acorn-py`, import name `acorn`, Python minimum `3.10`, and existing public objects during the repository move.
5. Decide and document the initial wheel target matrix from the package's declared Pixi platforms: Linux x86-64, Windows x86-64, macOS x86-64, and macOS arm64. Preserve the current wheel-only release policy for the first standalone release; adding an sdist is separate packaging work.
6. Freeze binding feature work across the two repositories during the final cutover window so no commits are lost between the history export and ACORN deletion.

## 2. Populate the standalone repository with history

1. In a disposable clone, filter the ACORN history to `crates/acorn-py/` and rewrite that prefix to the new repository root. Push the rewritten branch to the existing `acorn-py` remote without importing ACORN release tags, because the two projects will have independent release histories.
2. Record the source ACORN commit and extraction procedure in the first migration commit and new repository documentation. Verify representative historical commits with `git log --follow` before opening development on the new repository.
3. Add repository-level files that were previously inherited from ACORN: `LICENSE`, `CONTRIBUTING.md`, `SECURITY.md`, maintainer/ownership metadata, agent guidance if retained, and a Python-specific `CHANGELOG.md`. Carry forward relevant historical `acorn-py` changelog entries without rewriting ACORN's historical changelog.
4. Move the existing [Python binding deduplication](../refactors/core/python-binding-deduplication.md) plan into the new repository's planning system, preserving its planned status. Remove the ACORN copy and index entry only during the ACORN cleanup phase.

## 3. Make acorn-py independently buildable

1. Replace `version.workspace = true` with an explicit package version owned by `acorn-py`. Give the Rust package metadata and `[project]` metadata one documented version source of truth that Maturin can read.
2. Replace `acorn-lib = { path = "../acorn-lib", ... }` with a crates.io dependency on the verified compatible `acorn-lib` release. Keep local path overrides developer-only and uncommitted so release artifacts cannot accidentally contain a monorepo path assumption.
3. Move only the PyO3-required Cargo configuration from ACORN. Prefer Maturin's extension-module behavior; retain target linker flags or `PYTHON_SYS_EXECUTABLE` only if standalone macOS builds prove they are still required.
4. Copy the relevant Rust toolchain, formatting, lint, and dependency-policy settings into the standalone project instead of relying on ACORN workspace inheritance. Keep `Cargo.lock` and `pixi.lock` committed and regenerate them from the new root.
5. Replace mutating Pixi lint tasks with CI-safe check tasks: Rust formatting and Clippy checks, Ruff format/check, and Black check if Black remains part of the chosen formatter policy. Keep separate developer fix/format tasks.
6. Update package metadata URLs for source, issues, changelog, and documentation to the new project. Update README badges only after their pipeline and release endpoints exist.

## 4. Give acorn-py its own CI and release workflow

Create a root `.gitlab-ci.yml` in `acorn-py` with independently owned stages and no dependency on an ACORN checkout:

1. **Static checks:** validate `cargo fmt --check`, Clippy for the extension crate, non-mutating Ruff/Black checks, package metadata, and lockfile consistency on merge requests and the default branch.
2. **Source tests:** run the current pytest suite through Pixi after `maturin develop`. Test the declared minimum Python version and the newest explicitly supported version, and make platform-specific tests visible rather than relying only on the current Python 3.13 environment.
3. **Wheel builds:** build Maturin wheels for Linux x86-64, Windows x86-64, macOS x86-64, and macOS arm64 on suitable dedicated runners. Store wheels as pipeline artifacts and retain platform and Python ABI information in artifact names.
4. **Wheel smoke tests:** install each produced wheel into a clean environment, import `acorn`, exercise one representative binding, and confirm package metadata/version. At least the minimum and newest supported Python versions must install the `abi3-py310` wheel successfully.
5. **Release gate:** accept only protected tags whose version exactly matches the package version. Require all static, test, build, and wheel-install jobs before publication.
6. **PyPI publication:** publish the already-tested wheel artifacts exactly once from a protected environment. Prefer a short-lived trusted publishing identity if the GitLab/PyPI deployment supports it; otherwise use a protected, masked, environment-scoped token. Remove the local `MATURIN_PASSWORD` workflow after the CI publisher is proven.
7. **Release record:** create a GitLab release with changelog notes, checksums, and links to PyPI after publication succeeds. A retry must be idempotent and must not rebuild different artifacts for the same tag.
8. **Dependency compatibility:** add scheduled or dependency-update pipelines that refresh `acorn-lib` and other locked dependencies, run the full tests, and open reviewed updates. Changes to bindings that require a new Rust API must release `acorn-lib` first, then update and release `acorn-py` independently.

Configure branch protection, tag protection, required merge-request pipeline success, release-environment permissions, runner tags, artifact retention, dependency update ownership, and CI variables in the existing GitLab project. Document these settings because they are not fully represented by `.gitlab-ci.yml`.

## 5. Update acorn-py documentation

1. Make the new README the canonical landing page with installation commands, supported Python/platform versions, a minimal API example, compatibility policy for `acorn-lib`, links to PyPI and ACORN, and CI/release badges.
2. Add contributor documentation for Pixi setup, Maturin development builds, Rust and Python checks, wheel smoke testing, and the release/tag process.
3. Add a release checklist covering version change, changelog, compatibility test, protected tag, artifact verification, PyPI publication, and rollback/yank ownership.
4. Explain that `acorn-py` is a separately versioned binding project over published `acorn-lib`, identify where Rust API issues versus binding issues should be filed, and link to the ACORN Rust documentation.
5. Verify every source, issue, changelog, docs, and PyPI link after the new project is visible.

## 6. Cut ACORN over to the external project

Only merge the ACORN cleanup after the standalone default-branch pipeline passes and one release candidate or real release has built and installed every required wheel.

1. Delete `crates/acorn-py/` from ACORN.
2. Remove `crates/acorn-py` from workspace members, regenerate the root `Cargo.lock`, and confirm PyO3-only dependencies disappear unless another ACORN crate uses them.
3. Remove the `acorn-py` entry from `xtask` version synchronization and update its tests or expected output. ACORN release preparation must no longer inspect or edit a Python manifest.
4. Delete the `test-python` job and its path rules from `.gitlab-ci.yml`. Update the [CI runner pools](../refactors/ci-runner-pools.md) plan so ACORN no longer provisions or routes a Python test pool.
5. Remove the PyO3-only macOS linker configuration and `PYTHON_SYS_EXECUTABLE` override from `.cargo/config.toml` after native macOS ACORN builds confirm they have no other owner.
6. Update `ARCHITECTURE.md`, `AGENTS.md`, `.github/copilot-instructions.md`, and other contributor guidance so they list only crates present in the ACORN workspace and link to the standalone bindings when relevant.
7. Keep historical `CHANGELOG.md` entries intact. Remove or regenerate current generated artifacts such as coverage data if they still name `crates/acorn-py`.
8. Move the binding-deduplication plan as described above, remove its ACORN index entry, and update other active plans that describe an in-workspace Python boundary, including crate-boundary architecture and thiserror adoption.

## 7. Update ACORN user documentation

1. Retain a concise Python-package page in the ACORN book, but make it an overview that links to the canonical standalone documentation and issue tracker. Do not maintain duplicate detailed installation, API, development, or release instructions.
2. Update the root README's Python link, the package relationship diagram, developer learning page, concepts overview, mdBook summary text if needed, and all source-code links that currently target `crates/acorn-py`.
3. Describe `acorn-py` as a companion repository that consumes published `acorn-lib`, not as a generated workspace crate. Make independent versioning explicit so users do not expect ACORN and `acorn-py` version numbers to match.
4. Build the book, check internal links, and inspect the Python overview and neighboring package navigation in the rendered output.

## 8. Complete the handoff

1. Archive or redirect any obsolete package-development issue templates and links in ACORN to the new tracker while leaving historical issues and commits discoverable.
2. Open a small cross-repository smoke checklist for the next `acorn-lib` release: dependency update in `acorn-py`, full binding tests, and an issue routed to the correct repository if compatibility fails.
3. After one successful standalone release, revoke any ACORN project variable or maintainer credential that could publish `acorn-py` and verify only the new protected release job can publish to PyPI.
4. Announce the repository move in both changelogs/release notes with the cutover commit, new source URL, issue URL, and versioning policy.

# Validation

## Standalone acorn-py

1. Clone only `acorn-py` into a clean directory with no sibling ACORN checkout and run all documented setup, format, lint, build, and test commands.
2. Confirm `cargo metadata` resolves `acorn-lib` from the registry and contains no path outside the new repository.
3. Verify representative pre-extraction commits are reachable in the rewritten history and the recorded source commit matches the ACORN cutover base.
4. Run merge-request and default-branch pipelines, then build and smoke-test all four platform wheel targets on clean Python environments.
5. Dry-run the protected-tag/version gate. Publish one release candidate to TestPyPI if available, then install it by distribution name and import `acorn` before the first production release.
6. Verify a production tag publishes exactly the tested artifacts, creates a release record, and can be retried without changing or duplicating artifacts.

## ACORN cleanup

1. Run `cargo metadata --no-deps` and confirm no workspace member or manifest path contains `acorn-py`.
2. Run `cargo xtask check-version` and confirm it checks only ACORN-owned internal Rust dependencies.
3. Search tracked, non-historical sources for `crates/acorn-py`, `test-python`, `maturin`, `pyo3`, `PYTHON_SYS_EXECUTABLE`, and the old source URL. Every remaining occurrence must be an intentional historical note or external companion-project link.
4. Run GitLab CI Lint and confirm the ACORN pipeline contains no Python runner, Pixi, Maturin, wheel, or PyPI job.
5. Run `make lint`, `make test`, the native platform build checks affected by `.cargo/config.toml`, `make book`, and link validation.
6. Run `git diff --check` in both repositories and inspect their rendered documentation entry points.

# Rollback

Before the first standalone PyPI release, keep the ACORN removal merge request unmerged; fixes belong in the new repository and the history export can be repeated from the frozen cutover commit. After publication, do not restore a second writable copy of `crates/acorn-py` in ACORN. Roll back by fixing or yanking the standalone release according to the documented release policy, while ACORN continues to link to the new canonical repository.

# Acceptance criteria

1. [`research-enablement/acorn-py`](https://code.ornl.gov/research-enablement/acorn-py) contains the binding source and useful filtered history, builds without an ACORN checkout, and is the canonical source and issue location advertised on PyPI.
2. The Python distribution has an explicit independent version and consumes a published, documented-compatible `acorn-lib` dependency.
3. Its merge-request/default-branch CI performs non-mutating Rust and Python checks, source tests, platform wheel builds, and clean wheel-install smoke tests.
4. A protected, version-matched tag publishes only previously tested wheels to PyPI and creates an auditable release record without relying on ACORN secrets or jobs.
5. ACORN contains no `crates/acorn-py` workspace member, Python test job, PyO3-only Cargo configuration, Python version-sync entry, or package publication credential.
6. ACORN's current README, architecture, contributor guidance, mdBook, and active plans describe `acorn-py` as an external companion project and link to its canonical documentation.
7. `acorn-py` documents installation, compatibility, development, CI, releases, security reporting, contribution flow, and the ACORN relationship from its own repository.
8. ACORN's Rust lint, test, release-version, docs, and affected native build checks pass after cleanup; every required standalone wheel installs and passes its smoke test.
9. Historical ACORN changelog entries and commits remain discoverable, while new binding changes and releases occur only in `acorn-py`.

# Related plans

- [Python binding deduplication](../refactors/core/python-binding-deduplication.md)
- [GitLab CI runner pools](../refactors/ci-runner-pools.md)
- [Crate boundary architecture](../refactors/crate-boundary-architecture.md)
- [thiserror adoption](thiserror-adoption.md)
