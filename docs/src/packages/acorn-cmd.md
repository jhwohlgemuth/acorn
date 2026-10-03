# Native commands

`acorn-cmd` provides ACORN's dependency-light native command surface without depending on `acorn-core`, `acorn-lib`, schemas, or analysis code.

```sh
cargo add acorn-cmd --no-default-features
```

The `cmd!` and `args!` macros preserve argument boundaries and accept path and string values without constructing a shell command. `CommandSpec` provides executable discovery, environment and working-directory configuration, checked output, spawning, and bounded execution:

```rust
use acorn_cmd::{args, CommandSpec};
use std::time::Duration;

let output = CommandSpec::new("git")
    .args(args!["status", "--short"])
    .env("GIT_TERMINAL_PROMPT", "0")
    .checked_output_with_timeout(Duration::from_secs(10))?;
# Ok::<(), std::io::Error>(())
```

## Managed execution

Version 0.3.3 adds managed execution as the preferred API for cancellation, descendant cleanup, streaming, and bounded capture. Each execution owns a process group on Unix or a Job Object on Windows. The group is terminated and reaped on timeout, cancellation, output failure, panic unwind, and after the process leader exits, so supervised descendants are never detached.

Options are optional. `cmd!(exec "git status")` and `cmd!(exec checked "git status")` use `ExecutionOptions::new()`; add the named `opt` suffix only when the defaults need customization.

```rust
use acorn_cmd::{cmd, CancellationToken, ExecutionOptions, OutputStream};
use std::{io, time::Duration};

let cancellation = CancellationToken::new();
let options = ExecutionOptions::new()
    .with_cancellation(cancellation.clone())
    .with_timeout(Duration::from_secs(10))
    .with_sink(|stream: OutputStream, bytes: &[u8]| {
        eprintln!("{stream:?}: received {} bytes", bytes.len());
        Ok::<(), io::Error>(())
    });
let result = cmd!(exec checked "git", ["status", "--short"]; opt: &options)?;
assert!(result.status().is_some_and(|status| status.success()));
# Ok::<(), acorn_cmd::CommandError>(())
```

Capture retains raw stdout and stderr separately. Defaults keep the newest 1 MiB and 10,000 lines per stream, and report exact dropped-byte and dropped-line counts. Use `CaptureLimits::unlimited()` only when the caller explicitly accepts unbounded memory.

The existing `output`, `checked_output`, timeout variants, and `spawn` remain available. Output adapters retain unlimited capture for compatibility. `spawn` still returns an unmanaged direct child, so its caller owns descendant cleanup.

Managed `cmd!` forms accept the same options and preserve compile-time interpolation:

```rust
use acorn_cmd::{cmd, CaptureLimits, ExecutionOptions};

let options = ExecutionOptions::new().with_capture_limits(CaptureLimits::new(Some(64 * 1024), Some(500)));
let branch = "main";
let result = cmd!(exec "git rev-parse {branch}"; opt: &options)?;
# Ok::<(), acorn_cmd::CommandError>(())
```

`parse_sh` and `command_string` provide a shared, shell-free grammar for runtime command words and diagnostic rendering. They preserve empty and quoted arguments, but perform no expansion or shell execution.

Shell linting is optional and has no default activation:

```sh
cargo add acorn-cmd --no-default-features --features shell-lint
```

The command-literal parser remains internally isolated in `acorn-macros`; consumers use it only through `acorn-cmd`'s public macro surface. Macro expansion uses `$crate` paths, so renaming the dependency in `Cargo.toml` remains supported.
