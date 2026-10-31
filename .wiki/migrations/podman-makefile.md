---
type: Plan
title: "Replace Docker with Podman in the Makefile"
description: "Move ACORN's local container build, image publication, and build-in-image tasks to Podman on Linux, macOS, and Windows without changing product runtime support."
difficulty: medium
impact: high
package: [workspace]
tags: [migration, podman, containers, make, build, release, portability]
status: planned
execution: phased
phase_budget_hours: 5
---

# Project code boundaries

- **Update:** `Makefile`, focused container-development guidance in `CONTRIBUTING.md` and `docs/src/getting-started.md`, and `.gitlab-ci.yml` only if cross-platform validation is added to existing build jobs.
- **Create:** None. Add a dedicated validation script only if the final platform matrix cannot remain readable as Make targets and CI commands.
- **Delete:** No production files. Delete this plan and its index, network, and lane entries after the migration passes the full acceptance matrix.

# Goal

Make Podman the only container engine invoked by ACORN's Makefile while preserving the existing image build, local build-in-image, cleanup, tagging, registry publication, and multi-architecture release outcomes.

This migration is scoped to the Makefile workflow. It does not remove ACORN's user-facing Docker executor, Docker-compatible remote targets, Docker Hub publication, Dockerfile inputs, OCI media types with Docker names, or GitLab Runner's Docker executor.

# Assessment

The migration is viable, but it is not a literal replacement of the executable name. Most commands are CLI-compatible; multi-architecture publication and cross-platform execution require deliberate Podman-native handling.

| Platform | Viability | Required condition |
|---|---|---|
| Linux | High | Rootless Podman must be configured; foreign-architecture builds require registered QEMU/binfmt handlers, and enforcing SELinux requires an explicit bind-mount labeling policy. |
| macOS | High | A Podman machine must be initialized and running with enough CPU, memory, and disk for Rust and browser dependencies; non-native builds require the machine provider's emulation support. |
| Windows | Medium | Use the native Podman client with a Windows 11 WSL2 or Hyper-V machine and GNU Make from the existing Git Bash/MSYS2-style environment. Do not require one Windows host to execute both release architectures. |

The cross-platform promise is that each supported host can use the local image and build-in-image workflows and that native AMD64 and ARM64 builders can participate in the release workflow. It is not a promise that every host can execute arbitrary foreign-architecture `RUN` steps.

# Current evidence

- The Makefile has 55 recipe lines that invoke `docker` across `build-in-image`, ACORN and bucket image lifecycle targets, and Rust toolchain image targets.
- `docker image inspect`, `run`, `build`, `tag`, `push`, and `rmi` have corresponding Podman commands with compatible arguments for the options currently used.
- The repository does not use Docker Compose in the Makefile.
- `Dockerfile`, `Dockerfile.bucket`, and `Dockerfile.rust` use ordinary multi-stage Dockerfile syntax rather than Docker-only Compose or BuildKit mount extensions. Podman accepts both `Containerfile` and `Dockerfile` inputs.[^1]
- `Dockerfile.rust` declares `ARG TARGETARCH` inside its build stage. Podman/Buildah exposes the platform build arguments when they are declared in the relevant stage.[^2]
- The existing `rust-image` target builds `linux/amd64` and `linux/arm64` images separately. Both Docker and Podman need host emulation to execute a foreign architecture's `RUN` instructions; Podman documents this requirement explicitly.[^1]
- The existing `rust-push` target uses `docker buildx imagetools create`. Podman's `buildx build` command is only a compatibility alias and does not provide the complete Docker Buildx surface. Podman instead provides `podman manifest create`, `add`, `inspect`, `push`, and `rm` for image indexes.[^3]
- The existing `build-in-image` target binds `$(CURDIR)` to `/app` and mounts the named `acorn-target` volume. Podman supports both bind mounts and named volumes, but remote clients mount from the Podman server; Podman machines must therefore share or translate the client path.[^4]
- Podman supports Windows-style and Unix-style Windows bind paths when using the native Windows client with a Podman machine.[^5] The current `MSYS_NO_PATHCONV` and `MSYS2_ARG_CONV_EXCL` guards should remain until the Windows smoke test proves a narrower setting.
- Podman on macOS and Windows requires a Linux virtual machine. Current Podman documentation identifies WSL and Hyper-V as Windows providers and machine-backed providers on macOS.[^6]
- A 2026 upstream report shows ARM64 execution failing on an AMD64 Windows Podman machine when binfmt was unavailable. The release design must therefore support native per-architecture builders instead of making Windows cross-emulation a prerequisite.[^7]
- The assessment host has Podman 6.1.2 installed, but its default connection points to the remote `vergil` service. Live builds were intentionally not run because they would have changed that remote image store. Linux and Windows hosts were not available during the assessment.

# Compatibility decisions

## Engine and naming

- Set Podman 6.0 as the minimum cross-platform baseline and record both client and server versions in validation results. Older Linux packages may work, but they are not sufficient evidence for the Windows and remote-client contract.
- Define `PODMAN ?= podman` once and invoke `$(PODMAN)` from every container recipe. This allows callers to select a Podman binary or connection wrapper without introducing a generic Docker/Podman engine abstraction.
- Remove every Makefile invocation of the `docker` executable.
- Keep the `Dockerfile*` filenames. Podman supports them directly, and renaming would create churn without removing a runtime dependency.
- Keep `DOCKER_HUB_REGISTRY` because it names Docker Hub as a registry source rather than the Docker engine.
- Keep the compatibility targets `acorn-push-docker` and `bucket-push-docker`; their destination is Docker Hub. Clarify their help text and optionally add `*-dockerhub` aliases, but do not silently break scripts that call the existing targets.

## Image format and registries

- Add `--format docker` to Podman image builds for the first migration. This preserves the existing registry media format and health-check metadata while changing only the engine. A later OCI-format switch requires separate registry and consumer evidence.
- Continue using fully qualified Savannah and GHCR names and the existing Docker Hub namespace.
- Require `podman login` for each publication registry. Do not add credentials to command lines, Make variables, or repository files.
- Preserve TLS verification and existing registry mirrors. Registry certificate or proxy configuration remains host administration, not Makefile logic.

## Local build-in-image workflow

- Replace image inspection and execution with `podman image inspect` and `podman run`.
- Retain the named `acorn-target` volume and the `/app` working directory.
- Retain the MSYS path-conversion guards for the Windows client.
- Apply a documented SELinux-safe bind policy for `$(CURDIR)`. Prefer a private `:Z` label for this single-container development mount; verify that macOS and Windows remote clients accept the same volume specification. If a platform rejects it, select the suffix conditionally rather than disabling SELinux separation globally.
- Do not initialize, start, stop, or reset Podman machines from Make. A failed `podman info` should report the installation/machine prerequisite without changing host state.
- Correct the current fallback contract while touching this target: after a missing combined Rust image, the fallback must produce or pull the exact tag that `podman run` consumes. It must not build only architecture-suffixed tags and then rely on an absent combined local tag.

## Architecture-specific builds

- Split Rust toolchain image work into explicit AMD64 and ARM64 build/push targets while retaining aggregate compatibility targets.
- Each architecture target builds and pushes only its declared `linux/<arch>` image. Native release builders are the supported path; emulation-capable hosts may run the aggregate target.
- Pass the target platform explicitly. Preserve `TARGETARCH` use in `Dockerfile.rust` and verify that the installed PowerShell archive matches the resulting image architecture.
- Keep `PLATFORM` as the local ACORN and bucket image selector, but make it overrideable with `?=` so ARM64 builders are not forced to produce AMD64 images.
- Do not infer successful runtime support from image metadata alone. Execute a command from every native image during validation.

## Multi-architecture manifest publication

Replace `docker buildx imagetools create` with this Podman lifecycle:

1. Build and push the immutable AMD64 and ARM64 tags from native builders.
2. On the publication host, remove a stale local manifest with the destination name only when `podman manifest exists` succeeds.
3. Create the versioned manifest from the two fully qualified, already-pushed architecture references. Use `docker://` transport references when adding remote images.
4. Push the versioned manifest with `podman manifest push --all`.
5. Create a separate `latest` manifest from the same immutable architecture references and push it with `--all`.
6. Inspect both remote manifests and require exactly the intended `linux/amd64` and `linux/arm64` entries before declaring publication successful.

Do not assemble a release manifest from unpushed local images on a workstation. The architecture images are the durable inputs, and the manifest publisher should need only registry access and image metadata.

# Implementation sequence

1. Introduce the `PODMAN` variable, preflight, Podman-compatible help text, and direct command replacements for inspection, run, cleanup, build, tag, and ordinary pushes.
2. Preserve Docker-format output and verify each Dockerfile on a native Linux builder before changing multi-architecture publication.
3. Split the Rust architecture targets and repair the `build-in-image` fallback so it consumes an existing exact image tag.
4. Replace Buildx manifest creation with the versioned and `latest` Podman manifest workflow.
5. Validate rootless Linux and SELinux behavior, then macOS Podman machine path sharing, then the Windows native client under GNU Make/Git Bash.
6. Add or adjust CI jobs only after the commands pass manually on the named hosts. Keep architecture builds native and serialize final manifest publication after both immutable tags exist.
7. Update contributor guidance with Podman installation, machine startup, login, native-architecture, and troubleshooting prerequisites. Keep end-user examples that intentionally demonstrate either OCI runtime.

# Validation

## Static checks

1. Run `make -n` for every affected target and confirm no recipe invokes the `docker` executable or `docker buildx`.
2. Confirm the compatibility target names still expand to the intended Docker Hub registry pushes.
3. Verify every Makefile target in `TASKS` resolves to an actual rule; fix the existing `acorn-build-in-image`/`build-in-image` mismatch if it remains present at implementation time.
4. Run the repository's Markdown link and whitespace checks for the updated wiki and contributor documentation.

## Image behavior

1. Build `Dockerfile`, `Dockerfile.bucket`, and `Dockerfile.rust` with Podman on a native Linux AMD64 host.
2. Build and run the Rust image on native Linux ARM64.
3. Inspect image architecture, entry point, command, environment, labels, health-check metadata, and Docker media type.
4. Run `build-in-image` twice. The first run may build or pull its image; the second must reuse it. Both runs must write usable artifacts without changing repository file ownership.
5. Run the ACORN image's `help` command and the bucket image's shell command.
6. Remove all local tags through the clean targets and confirm the targets handle Podman manifests and ordinary images intentionally.

## Platform matrix

| Host | Required smoke test |
|---|---|
| Rootless Linux AMD64 without SELinux | Build all native images, run `build-in-image`, push a disposable tag, and remove it locally. |
| Rootless enforcing-SELinux Linux | Run `build-in-image` against a clean checkout and verify read/write access without disabling host SELinux. |
| Linux ARM64 | Build and execute the ARM64 Rust image and publish its immutable test tag. |
| macOS ARM64 | Start an existing Podman machine, build the selected platform, exercise the source and named-volume mounts, and run `build-in-image`. |
| Windows 11 AMD64 | With the native Podman client and WSL2 provider, run GNU Make from Git Bash/MSYS2, build an AMD64 image, exercise the repository bind mount, and run `build-in-image`. Repeat with Hyper-V when that provider is part of the supported developer baseline. |

Record the Podman client/server versions, machine provider, host architecture, and whether emulation was used for each result.

## Registry and manifest behavior

1. Authenticate with Podman to disposable or staging namespaces for Savannah, Docker Hub, and GHCR.
2. Push architecture-specific Rust images and ordinary ACORN/bucket images.
3. Assemble versioned and `latest` Rust manifests from the pushed architecture tags.
4. Inspect the registry copies independently of local storage and confirm exactly one `linux/amd64` and one `linux/arm64` manifest entry.
5. Pull and execute the combined tag on native AMD64 and ARM64 hosts.
6. Confirm a failed architecture push prevents manifest publication and does not overwrite the previous versioned or `latest` manifest.

# Acceptance criteria

1. No Makefile recipe invokes the Docker executable or Docker Buildx.
2. Podman is the sole Makefile container engine; no transparent fallback to Docker is introduced.
3. Existing local build, cleanup, tagging, Docker Hub, Savannah, and GHCR publication outcomes remain available through compatible targets.
4. `build-in-image` works on the supported Linux, macOS, and Windows host configurations with correct source visibility and artifact ownership.
5. AMD64 and ARM64 Rust images are built and executed on native hosts, then published under immutable architecture tags.
6. Versioned and `latest` registry manifests contain exactly the intended Linux AMD64 and ARM64 images and are assembled with Podman manifest commands.
7. Windows support does not depend on unresolved ARM emulation inside an AMD64 Podman machine.
8. Docker-format metadata, entry points, health checks, and registry consumption remain compatible during the engine migration.
9. Contributor documentation states the Podman machine, resource, login, volume, SELinux, and native-architecture prerequisites.
10. ACORN's product-level Docker and Podman runtime choices remain unchanged outside the Makefile workflow.

# Dependencies and coordination

- Lane: Build/CI/performance in the [parallel execution guide](../parallel-execution.md).
- POD-11 remains planned and cannot start until [GitLab shell runner readiness](../testing/gitlab-shell-runner-readiness.md) GSR-02 passes. Runner 55645 cannot finish a tagged job. Podman, GNU Make, Git Bash/MSYS2, and a supported machine provider remain separate prerequisites.
- Take the CI/build entry-point lock for `Makefile` and any `.gitlab-ci.yml` wiring. Do not overlap with runner-pool, portability-gate, Miri, Creusot, fuzz, Needle host-validation, SLM runner, or other Makefile integration work.
- [Runner-pool isolation](../refactors/ci-runner-pools.md) remains independent operationally, but any Makefile or CI edits serialize. This plan does not change the GitLab Runner executor.
- [Remote container creation](../refactors/core/remote-create-container-targets.md) retains user-selectable Docker and Podman behavior. Do not use this migration to change CLI defaults, remote endpoint semantics, sockets, or rollback behavior.
- Docker Hub naming and Docker-compatible OCI media types are not dependencies on the Docker executable.

# Completion handling

After the platform and registry acceptance matrix passes, delete this plan and remove its migration-index, plan-network, and parallel-execution entries. Keep the supported host matrix, Podman prerequisites, manifest publication contract, and troubleshooting details in contributor and release documentation.

# Related plans

- [Runner-pool isolation](../refactors/ci-runner-pools.md)
- [Remote container creation](../refactors/core/remote-create-container-targets.md)
- [Needle 3 release-host validation](needle-3-upgrade.md)

[^1]: [Podman build documentation](https://docs.podman.io/en/stable/markdown/podman-build.1.html)
[^2]: [Containerfile platform build arguments](https://github.com/containers/common/blob/main/docs/Containerfile.5.md#platformosarch-arg)
[^3]: [Podman manifest documentation](https://docs.podman.io/en/stable/markdown/podman-manifest.1.html)
[^4]: [Podman run volume documentation](https://docs.podman.io/en/stable/markdown/podman-run.1.html#volume-vsource-volumehost-dircontainer-diroptions)
[^5]: [Podman for Windows volume mounting](https://github.com/containers/podman/blob/main/docs/tutorials/podman-for-windows.md#volume-mounting)
[^6]: [Podman machine documentation](https://docs.podman.io/en/latest/markdown/podman-machine.1.html)
[^7]: [Podman Windows binfmt issue 28174](https://github.com/containers/podman/issues/28174)

# Execution phases

## POD-01 — Add Podman preflight and static contract

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: none
- Exact write set: `Makefile`, `CONTRIBUTING.md`
- Primary integration lock: Makefile
- Outcome: Define `PODMAN`, minimum version checks, help text, and dry-run invariants without changing image outcomes.
- Focused validation: Run `make -n` for affected targets and verify the existing task-name mismatch.
- Handoff: Remove POD-01 and freeze preflight behavior for command replacement.

## POD-02 — Replace local inspect, run, and cleanup commands

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-01
- Exact write set: `Makefile`
- Primary integration lock: Makefile
- Outcome: Use Podman for image inspection, execution, removal, and local tag cleanup.
- Focused validation: Dry-run each changed target and run local inspection/cleanup against disposable tags.
- Handoff: Remove POD-02 and record the compatible command set.

## POD-03 — Replace ordinary image build and push commands

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-01
- Exact write set: `Makefile`
- Primary integration lock: Makefile
- Outcome: Build, tag, and push ACORN and bucket images with Docker-format Podman output.
- Focused validation: Dry-run and execute disposable native builds and pushes.
- Handoff: Remove POD-03 and preserve registry/media-type evidence.

## POD-04 — Repair build-in-image mounts and fallback

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-02
- Exact write set: `Makefile`, `CONTRIBUTING.md`
- Primary integration lock: Makefile
- Outcome: Preserve source/target mounts, MSYS guards, SELinux policy, ownership, and exact-tag fallback behavior.
- Focused validation: Run build-in-image twice and verify reuse, visibility, and artifact ownership.
- Handoff: Remove POD-04 and record host-specific mount requirements.

## POD-05 — Split native Rust architecture targets

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-03
- Exact write set: `Makefile`, `Dockerfile.rust`
- Primary integration lock: Makefile
- Outcome: Build and push immutable AMD64 and ARM64 Rust images independently with explicit platforms.
- Focused validation: Dry-run aggregate/leaf targets and inspect native image architecture and PowerShell artifacts.
- Handoff: Remove POD-05 and provide immutable tags to POD-06.

## POD-06 — Replace Buildx with Podman manifests

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-05
- Exact write set: `Makefile`
- Primary integration lock: Makefile
- Outcome: Create, add, inspect, push, and clean versioned/latest manifests from remote immutable tags.
- Focused validation: Exercise the lifecycle in a disposable registry and reject incomplete architecture sets.
- Handoff: Remove POD-06 and record the publication commands.

## POD-07 — Validate rootless Linux AMD64

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-03, POD-04
- Exact write set: rootless Linux AMD64 host, `docs/src/developer/podman-platform-matrix.md`
- Primary integration lock: Podman platform matrix
- Outcome: Prove native builds, build-in-image, disposable push, and cleanup without SELinux.
- Focused validation: Execute the documented Linux AMD64 smoke matrix.
- Handoff: Remove POD-07 and preserve versions and measured results.

## POD-08 — Validate enforcing-SELinux Linux

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-04
- Exact write set: enforcing-SELinux Linux host, `CONTRIBUTING.md`
- Primary integration lock: Podman platform matrix
- Outcome: Prove labeled source access and artifact writes without weakening host SELinux.
- Focused validation: Run build-in-image from a clean checkout and inspect labels/ownership.
- Handoff: Remove POD-08 and retain the accepted mount policy.

## POD-09 — Validate native Linux ARM64

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-05
- Exact write set: Linux ARM64 host, `docs/src/developer/podman-platform-matrix.md`
- Primary integration lock: Podman platform matrix
- Outcome: Build, run, and publish the immutable ARM64 Rust image natively.
- Focused validation: Execute the image and inspect architecture, metadata, and pushed tag.
- Handoff: Remove POD-09 and provide the immutable tag to POD-12.

## POD-10 — Validate macOS ARM64

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-04
- Exact write set: macOS ARM64 Podman machine, `CONTRIBUTING.md`
- Primary integration lock: Podman platform matrix
- Outcome: Prove machine-backed builds, shared paths, named volumes, and build-in-image.
- Focused validation: Run the documented macOS smoke matrix and record client/server versions.
- Handoff: Remove POD-10 and retain machine prerequisites.

## POD-11 — Validate Windows AMD64

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-04, GSR-02. GSR-02 is blocked: runner 55645 cannot finish a tagged job.
- Exact write set: Windows 11 Podman machine and Git Bash/MSYS2 workflow, `CONTRIBUTING.md`
- Primary integration lock: Podman platform matrix
- Outcome: Prove native-client AMD64 build and source/volume behavior without ARM emulation assumptions.
- Focused validation: Run the WSL2 provider matrix and Hyper-V only when it is supported baseline.
- Handoff: Remove POD-11 and retain path-conversion prerequisites.

## POD-12 — Validate registry manifests end to end

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-06, POD-07, POD-09
- Exact write set: staging Savannah, Docker Hub, and GHCR namespaces, `docs/src/developer/podman-platform-matrix.md`
- Primary integration lock: release manifest publication
- Outcome: Publish, inspect, pull, and execute versioned/latest AMD64+ARM64 manifests without overwriting on partial failure.
- Focused validation: Inspect remote manifests independently and run combined tags on both native architectures.
- Handoff: Remove POD-12 and preserve the release evidence.

## POD-13 — Update guidance and retire

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: POD-08, POD-10, POD-11, POD-12
- Exact write set: `CONTRIBUTING.md`, `.gitlab-ci.yml`, `.wiki/index.md`, `.wiki/migrations/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, `.wiki/migrations/podman-makefile.md`
- Primary integration lock: CI build entry points
- Outcome: Document the supported matrix, add only proven CI checks, and remove the migration plan.
- Focused validation: Run all Make dry-runs, CI lint, link checks, and wiki validation.
- Handoff: Delete this plan and all index, network, and lane entries.
