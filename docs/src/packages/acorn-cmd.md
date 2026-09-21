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

Shell linting is optional and has no default activation:

```sh
cargo add acorn-cmd --no-default-features --features shell-lint
```

The command-literal parser remains internally isolated in `acorn-macros`; consumers use it only through `acorn-cmd`'s public macro surface. Macro expansion uses `$crate` paths, so renaming the dependency in `Cargo.toml` remains supported.
