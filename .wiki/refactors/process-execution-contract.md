---
type: Plan
title: "Make process execution lossless and cancellable"
description: "Give acorn-cmd a reusable process-group lifecycle, bounded dual-stream capture, structured termination results, and cooperative cancellation."
difficulty: high
impact: critical
package: [acorn-cmd]
tags: [refactor, process, command, cancellation, timeout, stdout, stderr]
status: planned
---

# Goal

Make `acorn-cmd` the dependency-light owner of native child-process execution for ACORN and downstream consumers. One public contract must preserve arguments and byte output, bound memory, stream diagnostics, distinguish exit from timeout and cancellation, terminate descendants, and reap every child.

The crate's scope ends at process mechanics. Callers retain application policy: global signal handlers, sibling-work decisions, application exit codes, workflow workspaces, and journals.

# Current evidence

- `CommandSpec` already owns an immutable program, argument vector, working directory, and child-only environment overrides.
- `CommandSpec::output` delegates to `std::process::Command::output`, which captures complete stdout and stderr without a size bound.
- `output_with_timeout` drains both pipes on reader threads, but it polls every ten milliseconds, kills only the direct `Child`, returns timeout as an `io::Error`, and discards captured output and termination metadata on that path.
- `checked` converts a nonzero status into a formatted `io::Error`, retaining neither a typed status nor the full output for callers.
- `CommandOutput` trims and decodes output for presentation. Non-UTF-8 bytes become diagnostic text instead of remaining lossless data at the execution boundary.
- Runtime `cmd!(sh ...)` already uses `shell-words`, while `parse_sh` and `command_string` are hidden implementation helpers rather than a stable parser and renderer for consumers.
- The crate has no cancellation token, bounded tail capture, output sink, duration, process-group ownership, or unwind cleanup contract.
- `acorn-cmd` has ACORN-internal and external consumers, so replacing current `Output`-returning methods would create an unnecessary breaking release.

# Project code boundaries

- **Update:** `crates/acorn-cmd/src/{lib,process,runtime}.rs`, `crates/acorn-cmd/Cargo.toml`, workspace dependency/version metadata, `Cargo.lock`, focused crate tests, examples, and public documentation.
- **Create:** a focused execution module when needed; public execution/result/error types; helper-process integration fixtures for output, descendants, timeout, and cancellation.
- **Delete:** the direct-child-only timeout loop, lossy checked-error conversion inside the new API, and duplicate hidden parsing or rendering helpers after all callers use the public equivalents.

# Dependencies and ordering

Add `command-group` 5.0.1 without Tokio features. It extends `std::process::Command` with Unix process groups and Windows Job Objects, and its group handle can kill and wait for the owned group ([crate documentation](https://docs.rs/command-group/5.0.1/command_group/), [standard-library adapter](https://docs.rs/command-group/5.0.1/command_group/stdlib/trait.CommandGroup.html)). Do not use its synchronous `wait_with_output`: the crate documents a Windows dual-pipe blocking limitation, so ACORN retains its own concurrent drain implementation ([`GroupChild` documentation](https://docs.rs/command-group/5.0.1/command_group/struct.GroupChild.html)).

Keep `shell-words` as the single runtime parser. Do not add `shlex` or invoke a shell for word parsing.

Land the additive contract, compatibility adapters, tests, and documentation before publishing workspace version 0.3.3. Publish `acorn-cmd` 0.3.3 before downstream Xylem changes its version and Git revision. The source change alone is not a usable prerequisite for crates.io builds.

# Public contract

Expose these generic types from `acorn-cmd`:

- `CancellationToken`: cheap to clone, safe to share across threads, idempotently cancellable, and independent of operating-system signal registration.
- `CaptureLimits`: optional byte and line ceilings per stream. Its default retains at most 1 MiB and 10,000 lines for each of stdout and stderr.
- `CapturedOutput`: raw retained bytes plus exact truncated-byte and truncated-line counts.
- `OutputStream`: `Stdout` or `Stderr`.
- `OutputSink`: a `Send + Sync` callback that receives attributed byte chunks and can fail.
- `ExecutionOptions`: timeout, cancellation token, capture limits, and optional sink.
- `ProcessStatus`: success, exit-code, and platform signal information from the reaped process leader.
- `ProcessTermination`: `Exited`, `TimedOut`, or `Cancelled`.
- `ExecutionResult`: optional status, termination, stdout, stderr, and duration. Status is absent only when cancellation prevents spawn.
- `CommandError`: typed invalid-option, spawn, capture, sink, wait, and cleanup failures plus checked nonzero, timeout, and cancellation variants that retain the complete `ExecutionResult`.

Add:

```rust
CommandSpec::execute(&ExecutionOptions) -> Result<ExecutionResult, CommandError>
CommandSpec::checked_execute(&ExecutionOptions) -> Result<ExecutionResult, CommandError>
```

`execute` returns `Ok(ExecutionResult)` when cancellation prevents spawn or when a spawned command's capture work completes and its owned process group is reaped. This includes observed nonzero, timeout, and cancellation termination. `checked_execute` maps nonzero exit, timeout, and cancellation to typed `CommandError` variants carrying that result. Operational failures remain errors in both methods.

Do not expose Xylem-specific failure policy, cancellation reasons, journal states, or scheduler types.

# Lifecycle and cancellation semantics

Every new execution creates an owned Unix process group or Windows Job Object before user code can observe the child. Timeout, token cancellation, output-sink failure, reader failure, and panic unwind trigger one cleanup path that terminates the group and waits for it. Successful and nonzero exits also wait for the group before returning.

Cancellation is level-triggered: a token cancelled before `execute` prevents spawn and returns a cancelled result with no process status; cancellation after spawn terminates and reaps the group. Define the exact pre-spawn representation in the public type so callers never infer it from a fabricated exit code.

Timeout starts immediately before spawn and includes spawn-to-reap duration. When timeout and cancellation become observable in the same polling interval, cancellation wins because it records an explicit caller decision. A naturally observed exit wins over a later token check.

No method detaches a supervised group. `CommandSpec::spawn` remains available with its existing direct-child semantics for compatibility, but its documentation must state that the caller owns descendant lifecycle and that managed execution should use `execute`.

# Capture and streaming semantics

Drain stdout and stderr concurrently from process start through EOF. Preserve bytes exactly and preserve order within each stream; do not claim a total order between streams.

The default capture keeps a tail. Enforce both the byte and line ceiling, retaining the newest data that satisfies both and tracking everything dropped. An unlimited `CaptureLimits` constructor supports compatibility adapters and callers that explicitly accept unbounded memory.

When a sink is present, deliver every byte once in per-stream order while independently retaining the bounded tail. Sink calls may occur from separate reader threads, so the sink must handle concurrency. A sink failure stops normal execution, terminates and reaps the group, joins both readers, and returns a typed error with the available bounded tails.

Keep decoding, trimming, redaction, and terminal formatting outside the execution core. `CommandOutput` can remain as a convenience for legacy `std::process::Output` users.

# Parsing and diagnostic rendering

Promote the existing runtime word parser to a documented public function that returns one program plus exact arguments. It uses `shell-words` syntax consistently on every platform, performs no expansion or shell execution, preserves quoted spaces and empty quoted arguments, and rejects empty or malformed input.

Promote command rendering to a public diagnostic helper. It must make every argument boundary visible and escape values according to the documented display grammar. Rendering is for logs and copyable diagnostics; execution always uses the structured program and argument vector.

Keep compile-time `cmd!` parsing in `acorn-macros`. Add shared contract tests so compile-time literal words and runtime words agree for their common supported syntax without coupling the proc-macro crate to process supervision.

# Compatibility and release

Preserve `CommandSpec::{output,checked_output,output_with_timeout,checked_output_with_timeout,spawn}` and existing `cmd!` forms in the 0.3.3 release.

- Back the output methods with the new lifecycle core where that does not change their return type or documented behavior.
- Use unlimited capture for legacy `Output` methods so adding this contract does not silently truncate existing callers.
- Map structured errors back to `io::Error` only at the legacy boundary.
- Do not change macro syntax or the `CommandOutput` text helpers in this plan.

Add a release note that managed execution is the preferred API for cancellation, descendant cleanup, streaming, and bounded capture. ACORN-internal migrations may follow independently.

# Implementation phases

1. Add result, option, cancellation, capture, sink, and error types with pure unit tests.
2. Add group spawn, concurrent bounded readers, the single kill-and-reap path, and unwind protection.
3. Implement `execute` and `checked_execute`, then route compatible legacy methods through unlimited capture.
4. Promote parser and renderer helpers and migrate runtime macro support to them without changing macro syntax.
5. Add cross-platform helper-process tests, downstream compile fixtures, rustdoc, and the 0.3.3 release change.

# Validation

- Test exact program and argument preservation for spaces, quotes, escapes, empty values, Unicode, and malformed runtime strings.
- Generate stdout and stderr concurrently beyond pipe capacity and beyond both capture limits; assert completion, exact sink delivery, separate ordered tails, and truncation counts.
- Test silent success, nonzero exit, missing executable, bad working directory, non-UTF-8 bytes, sink failure, reader/wait failure where injectable, timeout, pre-spawn cancellation, active cancellation, and timeout/cancellation races.
- Spawn a helper that creates a grandchild holding both output handles. On Linux, macOS, and Windows, prove timeout, cancellation, sink failure, and unwind leave no owned descendant and return only after reap.
- Run repeated race tests under the normal crate suite and narrowly gate platform-specific status or signal assertions.
- Compile existing ACORN callers and a downstream fixture against the preserved methods and new public API.
- Run `cargo test -p acorn-cmd`, workspace lint and test gates, and supported platform CI before publishing.

# Acceptance criteria

- Managed execution owns a process group or Job Object and leaves no supervised child or descendant running on any return path.
- Stdout and stderr are drained concurrently, remain separate raw bytes, and are bounded by default with exact truncation metadata.
- Optional sinks receive attributed bytes in per-stream order without disabling bounded capture.
- Exit, signal, timeout, cancellation, and operational failures remain distinguishable; checked errors retain the full result.
- Runtime command strings preserve supported quoting and empty arguments, while diagnostic rendering exposes argument boundaries.
- `acorn-cmd` installs no global signal handler and contains no application scheduler or failure policy.
- Existing methods and macro forms remain source-compatible and retain unlimited output behavior in 0.3.3.
- Linux, macOS, Windows, downstream compile, lint, and test gates pass.

# Related plans

- [Crate boundary architecture](crate-boundary-architecture.md)
- [thiserror adoption](../migrations/thiserror-adoption.md)
- [Xylem workflow environment and concurrency](https://code.ornl.gov/research-enablement/xylem/-/blob/main/.wiki/refactors/workflow-environment-and-concurrency.md)
