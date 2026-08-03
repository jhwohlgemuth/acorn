---
type: Plan
title: "Coordinate CLI progress and log output"
description: "Route standalone progress indicators and tracing output through one shared terminal renderer."
difficulty: medium
impact: medium
package: [acorn-cli, acorn-lib]
tags: [refactor, cli, logging, progress]
status: planned
---

# Goal

Ensure progress indicators and tracing events share terminal ownership so logs suspend active indicators, write complete lines, and redraw each indicator in place.

# Current evidence

`acorn-cli/src/main.rs` installs a `tracing_indicatif::IndicatifLayer` and sends tracing output through its stderr writer. The writer suspends only the `MultiProgress` owned by that layer.

`acorn-lib/src/io/mod.rs` creates independent `indicatif::ProgressBar` values with the default draw target. Download, export, database, configuration, GitLab, Models.dev, and SPDX operations use those standalone indicators. Because they are not registered with the tracing layer's `MultiProgress`, a tracing event can interrupt a rendered indicator and leave a stale line behind.

The immediate download defect is mitigated by logging before constructing the download indicator. That ordering constraint does not protect other operations or logs emitted while a long-running indicator is active.

# Recommendation

Introduce one CLI-owned progress renderer and make it available to library progress construction without coupling `acorn-lib` to `tracing-indicatif`.

1. Define a small progress-rendering context in `acorn-lib` that owns or accepts an `indicatif::MultiProgress` and creates registered bars and spinners.
2. Initialize the context from `acorn-cli` using the same `MultiProgress` that protects the tracing stderr writer.
3. Route `create_progress_bar` and `with_progress` through that context while preserving hidden indicators for quiet output.
4. Keep a standalone default for library consumers that do not install CLI tracing.
5. Migrate direct progress construction and verify that no output path bypasses the shared renderer while an indicator is active.

# Validation

- Add terminal-output tests that hold an active bar, emit a tracing event, advance the bar, and assert that the initial rendering is cleared rather than retained as a second line.
- Cover determinate bars, indeterminate spinners, quiet mode, successful completion, and error cleanup.
- Exercise parallel progress operations to verify stable ordering and absence of deadlocks.
- Run `make lint` and `make test`.

# Acceptance criteria

- Tracing output never shares a physical terminal line with an active ACORN progress indicator.
- A progress update redraws the existing indicator instead of appearing as a second bar.
- All CLI progress indicators use the renderer coordinated with tracing.
- Quiet mode and non-CLI `acorn-lib` consumers retain their current behavior.
- Concurrent logging and progress updates do not deadlock or leave stale terminal lines.

# Related plans

- [CLI startup separation](cli-startup-separation.md)
