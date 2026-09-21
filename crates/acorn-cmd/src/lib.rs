//! Command construction and execution utilities for ACORN
#![forbid(unsafe_code)]
extern crate alloc;

mod process;
pub mod runtime;

pub use process::CommandOutput;
pub use runtime::{find_executable, CommandSpec};

#[doc(hidden)]
pub mod __private {
    pub use acorn_macros::cmd_sh_words;
    pub use alloc::vec::Vec;
}
/// Execute a command and capture its output.
///
/// Simplifies `Command::new(binary).args(args).output()` patterns.
/// Thread safe — `Command` is `Send + Sync`.
///
/// Supports several calling conventions:
///
/// | Prefix         | Returns                        | Description                              |
/// |----------------|--------------------------------|------------------------------------------|
/// | `"literal"`    | `io::Result<Output>`           | Interpolated literal, capture output     |
/// | `status "lit"` | `io::Result<ExitStatus>`       | Interpolated literal, exit status only   |
/// | `try "lit"`    | `Result<String, String>`       | Interpolated literal, stdout or error    |
/// | *(none)*       | `io::Result<Output>`           | Run and capture output                   |
/// | `status`       | `io::Result<ExitStatus>`       | Exit status only (no output)             |
/// | `sh`           | `io::Result<Output>`           | Parse string, capture output             |
/// | `sh status`    | `io::Result<ExitStatus>`       | Parse string, exit status only           |
/// | `try sh`       | `Result<String, String>`       | Parse string, stdout or error            |
/// | `bash`         | `io::Result<Output>`           | Run via `bash -c <command>`              |
/// | `pwsh`         | `io::Result<Output>`           | Run via `pwsh -NoProfile -Command ...`   |
/// | `try`          | `Result<String, String>`       | Run, return stdout or error message      |
///
/// All forms support an optional `; dir: path` suffix to set the working directory.
///
/// # When to use each form
///
/// | Use case                            | Recommended form                        |
/// |-------------------------------------|-----------------------------------------|
/// | Known command, no shell features    | `cmd!("git rev-parse {branch}")`        |
/// | Known command, with interpolation   | `cmd!("echo {name}")`                   |
/// | Simple static command               | `cmd!("git" "status")`                  |
/// | Dynamic command string              | `cmd!(sh format!("diff {file}"))`       |
/// | Need shell features (pipes, globs)  | `cmd!(bash "echo * \| wc -l")`          |
/// | Need PowerShell                     | `cmd!(pwsh "Get-ChildItem \| Select-Object Name")` |
///
/// On the surface `cmd!("...")` and `cmd!(sh "...")` both accept a literal string, but
/// the bare literal form is the preferred entry point — it's the simplest, avoids the
/// unnecessary `sh` prefix, and supports the full `{var}` / `{args...}` interpolation.
/// The `sh` prefix is useful when the command string is a *runtime expression* (e.g.
/// `format!(...)`), since `cmd!(sh $expr)` parses it with shell-aware word splitting.
///
/// # Literal interpolation
///
/// A literal string (a bare `"..."` token without `sh`) is parsed at compile time
/// and supports both interpolation forms:
///
/// - **`{name}`** — expands a single value implementing `AsRef<OsStr>` (works with
///   `String`, `&str`, `OsString`, `PathBuf`, etc.) into one command argument
/// - **`{name...}`** — expands an iterable (splat) into zero or more command
///   arguments
///
/// Both forms must occupy a full unquoted shell word. Embedded or double-quoted
/// interpolation like `--flag={name}` and `"{name}"` is rejected at compile time.
/// Single quotes keep the placeholder literal:
///
/// ```ignore
/// cmd!("echo '{name}'")  // prints {name}
/// cmd!("echo '{args...}'") // prints {args...}
/// ```
///
/// ## Single-value `{var}` interpolation
///
/// ```ignore
/// let msg = "hello world";
/// let output = cmd!("echo {msg}").expect("echo should succeed");
/// assert_eq!(output.stdout(), "hello world");
///
/// let branch = "main".to_string();
/// let output = cmd!("git rev-parse {branch}").expect("git should succeed");
/// ```
///
/// `{var}` accepts any type implementing [`AsRef<OsStr>`] — this includes `str`,
/// `String`, `OsStr`, `OsString`, `Path`, and `PathBuf`. For types like integers
/// that don't implement `AsRef<OsStr>`, convert first:
///
/// ```ignore
/// let port = 8080;
/// cmd!("curl http://localhost:{port}");              // compile error
/// cmd!("curl http://localhost:{}", port.to_string()); // use .to_string()
/// let p = format!("localhost:{}", port);
/// cmd!("curl http://{p}");                           // works via String
/// ```
///
/// ## Splat `{args...}` interpolation
///
/// Splat placeholders expand a local variable that implements borrowed
/// iteration (arrays, `Vec`, `Option`, slices, etc.):
///
/// ```ignore
/// let args = ["hello", "world"];
/// let output = cmd!("echo {args...}").expect("echo should succeed");
/// assert_eq!(output.stdout(), "hello world");
///
/// let arg1: Option<&str> = Some("hello");
/// let arg2: Option<&str> = None;
/// let output = cmd!("echo {arg1...} {arg2...}").expect("echo should succeed");
/// assert_eq!(output.stdout(), "hello");
/// ```
///
/// ## In `sh` literals
///
/// The same `{var}` and `{args...}` interpolation also works in literal `sh`
/// command strings:
///
/// ```ignore
/// cmd!(sh "echo {msg}")
/// cmd!(sh "echo {args...}")
/// cmd!(sh status "echo {msg}")
/// cmd!(try sh "echo {args...}")
/// cmd!(sh "echo {msg}"; dir: project_dir)
/// ```
///
/// Runtime `sh` strings (non-literal `$cmd:expr`) are split via `shell-words`
/// at runtime and do not support interpolation — use `format!` or build an
/// argument list explicitly.
///
/// # Syntax
///
/// ```ignore
/// // Interpolated literal (compile-time parsed, fastest)
/// cmd!("git rev-parse {branch}")
/// cmd!("echo {args...}")
/// cmd!(status "git status")
/// cmd!(try "git rev-parse HEAD")
/// cmd!("echo {msg}"; dir: project_dir)
///
/// // String form via sh (shell-aware quoting)
/// cmd!(sh "git diff --name-only")
/// cmd!(sh format!("git diff --name-only {branch}"))
/// cmd!(sh "echo 'hello world'")   // handles quotes
/// cmd!(sh "echo {args...}")       // literal-only interpolation
/// cmd!(sh status "echo {args...}")
/// cmd!(try sh "echo {args...}")
/// cmd!(bash "echo 'hello world'")
/// cmd!(pwsh "Write-Output 'hello world'")
///
/// // CLI-style literals
/// cmd!("git" "diff-tree" "--no-commit-id" "--name-only")
/// cmd!("git" "log" "--oneline"; dir: repo_path)
///
/// // Array literal (dynamic types)
/// cmd!("git", ["branch", "--show-current"])
///
/// // Variable args
/// cmd!("git", args)
/// cmd!("git", args; env: [("GIT_CONFIG_NOSYSTEM", "1")])
///
/// // Try form — returns Result<String, String>
/// cmd!(try "git" "rev-parse" "HEAD")
/// cmd!(try "git", args)
///
/// // With working directory
/// cmd!("git" "status"; dir: project_dir)
/// cmd!("git", args; dir: project_dir, env: environment)
/// cmd!(try sh "npm test"; dir: project_dir)
/// cmd!(status bash "echo hello"; dir: project_dir)
/// cmd!(try pwsh "Write-Output 'hello'"; dir: project_dir)
/// ```
///
/// # Examples
///
/// ```ignore
/// use acorn_cmd::cmd;
/// use acorn_cmd::CommandOutput;
///
/// // Interpolated literal (no shell overhead)
/// let branch = "main";
/// let output = cmd!("git rev-parse {branch}").expect("git should succeed");
///
/// // String form with shell-aware quoting
/// match cmd!(sh format!("git diff --name-only {branch}")) {
///     Ok(output) if output.status.success() => {
///         println!("{}", output.stdout());
///     }
///     _ => {},
/// }
///
/// // CLI-style
/// match cmd!("git" "branch" "--show-current") {
///     Ok(output) if output.status.success() => {
///         println!("{}", output.stdout());
///     }
///     Ok(output) => eprintln!("{}", output.stderr()),
///     Err(why) => eprintln!("Error: {}", why),
/// }
///
/// // Try form — simplified error handling
/// match cmd!(try "git" "rev-parse" "HEAD") {
///     Ok(hash) => println!("{hash}"),
///     Err(msg) => eprintln!("failed: {msg}"),
/// }
///
/// // Explicit shell selection
/// let output = cmd!(bash "echo bash-mode").expect("bash should succeed");
/// let output = cmd!(pwsh "Write-Output 'pwsh-mode'").expect("PowerShell should succeed");
///
/// // Splat interpolation
/// let args = ["hello", "world"];
/// let output = cmd!("echo {args...}").expect("echo should succeed");
/// assert_eq!(output.stdout(), "hello world");
///
/// let dry_run = Some("--dry-run");
/// let extra: Option<&str> = None;
/// let output = cmd!(try "cargo publish {dry_run...} {extra...}").expect("cargo publish should succeed");
/// ```
#[macro_export]
macro_rules! cmd {
    // ── spawned child ──────────────────────────────────────────
    (spawn $binary:expr, $args:expr; dir: $dir:expr) => {
        $crate::runtime::spawn($binary, $args, Some($dir.as_ref()))
    };
    (spawn $binary:expr, $args:expr) => {
        $crate::runtime::spawn($binary, $args, None)
    };
    // ── sh status ──────────────────────────────────────────────
    // sh status literal + dir
    (sh status $cmd:literal; dir: $dir:expr) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::run_status(&__binary, __args, Some($dir.as_ref()))
            }
            | None => Err(::std::io::Error::other("cmd!(sh ...): empty command string")),
        }
    }};
    // sh status string + dir
    (sh status $cmd:expr; dir: $dir:expr) => {{
        match $crate::runtime::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::runtime::run_status(&binary, &args, Some($dir.as_ref())),
            Err(e) => Err(e),
        }
    }};
    // sh status literal (no dir)
    (sh status $cmd:literal) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::run_status(&__binary, __args, None)
            }
            | None => Err(::std::io::Error::other("cmd!(sh ...): empty command string")),
        }
    }};
    // sh status string (no dir)
    (sh status $cmd:expr) => {{
        match $crate::runtime::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::runtime::run_status(&binary, &args, None),
            Err(e) => Err(e),
        }
    }};
    // ── sh output ──────────────────────────────────────────────
    // sh literal + dir
    (sh $cmd:literal; dir: $dir:expr) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::run_output(&__binary, __args, Some($dir.as_ref()))
            }
            | None => Err(::std::io::Error::other("cmd!(sh ...): empty command string")),
        }
    }};
    // sh string + dir
    (sh $cmd:expr; dir: $dir:expr) => {{
        match $crate::runtime::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::runtime::run_output(&binary, &args, Some($dir.as_ref())),
            Err(e) => Err(e),
        }
    }};
    // sh literal (no dir)
    (sh $cmd:literal) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::run_output(&__binary, __args, None)
            }
            | None => Err(::std::io::Error::other("cmd!(sh ...): empty command string")),
        }
    }};
    // sh string (no dir)
    (sh $cmd:expr) => {{
        match $crate::runtime::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::runtime::run_output(&binary, &args, None),
            Err(e) => Err(e),
        }
    }};
    // ── bash status ────────────────────────────────────────────
    // status bash string + dir
    (status bash $cmd:expr; dir: $dir:expr) => {{
        $crate::runtime::run_shell_status("bash", &["-c"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // status bash string (no dir)
    (status bash $cmd:expr) => {{
        $crate::runtime::run_shell_status("bash", &["-c"], &$cmd.to_string(), None)
    }};
    // bash status string + dir
    (bash status $cmd:expr; dir: $dir:expr) => {{
        $crate::runtime::run_shell_status("bash", &["-c"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // bash status string (no dir)
    (bash status $cmd:expr) => {{
        $crate::runtime::run_shell_status("bash", &["-c"], &$cmd.to_string(), None)
    }};
    // ── bash output ────────────────────────────────────────────
    // bash string + dir
    (bash $cmd:expr; dir: $dir:expr) => {{
        $crate::runtime::run_shell_output("bash", &["-c"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // bash string (no dir)
    (bash $cmd:expr) => {{
        $crate::runtime::run_shell_output("bash", &["-c"], &$cmd.to_string(), None)
    }};
    // ── pwsh status ────────────────────────────────────────────
    // status pwsh string + dir
    (status pwsh $cmd:expr; dir: $dir:expr) => {{
        $crate::runtime::run_shell_status("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // status pwsh string (no dir)
    (status pwsh $cmd:expr) => {{
        $crate::runtime::run_shell_status("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), None)
    }};
    // pwsh status string + dir
    (pwsh status $cmd:expr; dir: $dir:expr) => {{
        $crate::runtime::run_shell_status("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // pwsh status string (no dir)
    (pwsh status $cmd:expr) => {{
        $crate::runtime::run_shell_status("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), None)
    }};
    // ── pwsh output ────────────────────────────────────────────
    // pwsh string + dir
    (pwsh $cmd:expr; dir: $dir:expr) => {{
        $crate::runtime::run_shell_output("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // pwsh string (no dir)
    (pwsh $cmd:expr) => {{
        $crate::runtime::run_shell_output("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), None)
    }};
    // ── try sh ─────────────────────────────────────────────────
    // try sh literal + dir
    (try sh $cmd:literal; dir: $dir:expr) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::try_from_output(
                    $crate::runtime::run_output(&__binary, __args, Some($dir.as_ref())),
                )
            }
            | None => Err("cmd!(sh ...): empty command string".to_string()),
        }
    }};
    // try sh string + dir
    (try sh $cmd:expr; dir: $dir:expr) => {{
        match $crate::runtime::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::runtime::try_from_output(
                $crate::runtime::run_output(&binary, &args, Some($dir.as_ref())),
            ),
            Err(e) => Err(format!("{e}")),
        }
    }};
    // try sh literal (no dir)
    (try sh $cmd:literal) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::try_from_output(
                    $crate::runtime::run_output(&__binary, __args, None),
                )
            }
            | None => Err("cmd!(sh ...): empty command string".to_string()),
        }
    }};
    // try sh string (no dir)
    (try sh $cmd:expr) => {{
        match $crate::runtime::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::runtime::try_from_output(
                $crate::runtime::run_output(&binary, &args, None),
            ),
            Err(e) => Err(format!("{e}")),
        }
    }};
    // ── try bash ───────────────────────────────────────────────
    // try bash string + dir
    (try bash $cmd:expr; dir: $dir:expr) => {{
        $crate::runtime::try_from_output(
            $crate::runtime::run_shell_output("bash", &["-c"], &$cmd.to_string(), Some($dir.as_ref())),
        )
    }};
    // try bash string (no dir)
    (try bash $cmd:expr) => {{
        $crate::runtime::try_from_output(
            $crate::runtime::run_shell_output("bash", &["-c"], &$cmd.to_string(), None),
        )
    }};
    // ── try pwsh ───────────────────────────────────────────────
    // try pwsh string + dir
    (try pwsh $cmd:expr; dir: $dir:expr) => {{
        $crate::runtime::try_from_output(
            $crate::runtime::run_shell_output("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), Some($dir.as_ref())),
        )
    }};
    // try pwsh string (no dir)
    (try pwsh $cmd:expr) => {{
        $crate::runtime::try_from_output(
            $crate::runtime::run_shell_output("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), None),
        )
    }};
    // ── status literal interpolation ─────────────────────────
    // status literal + dir
    (status $cmd:literal; dir: $dir:expr) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::run_status(&__binary, __args, Some($dir.as_ref()))
            }
            | None => Err(::std::io::Error::other("cmd!(...): empty command string")),
        }
    }};
    // status literal (no dir)
    (status $cmd:literal) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::run_status(&__binary, __args, None)
            }
            | None => Err(::std::io::Error::other("cmd!(...): empty command string")),
        }
    }};
    // ── try literal interpolation ──────────────────────────
    // try literal + dir
    (try $cmd:literal; dir: $dir:expr) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::try_from_output(
                    $crate::runtime::run_output(&__binary, __args, Some($dir.as_ref())),
                )
            }
            | None => Err("cmd!(...): empty command string".to_string()),
        }
    }};
    // try literal (no dir)
    (try $cmd:literal) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::try_from_output(
                    $crate::runtime::run_output(&__binary, __args, None),
                )
            }
            | None => Err("cmd!(...): empty command string".to_string()),
        }
    }};
    // ── default output literal interpolation ──────────────
    // literal + dir
    ($cmd:literal; dir: $dir:expr) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::run_output(&__binary, __args, Some($dir.as_ref()))
            }
            | None => Err(::std::io::Error::other("cmd!(...): empty command string")),
        }
    }};
    // literal (no dir)
    ($cmd:literal) => {{
        let mut __parts = $crate::__private::cmd_sh_words!($cmd).into_iter();
        match __parts.next() {
            | Some(__binary) => {
                let __binary = __binary.to_string_lossy().to_string();
                let __args = __parts.collect::<$crate::__private::Vec<_>>();
                $crate::runtime::run_output(&__binary, __args, None)
            }
            | None => Err(::std::io::Error::other("cmd!(...): empty command string")),
        }
    }};
    // ── status CLI-style ───────────────────────────────────────
    // status CLI-style + dir
    (status $binary:literal $($arg:literal)*; dir: $dir:expr) => {{
        $crate::runtime::run_status($binary, [$($arg),*], Some($dir.as_ref()))
    }};
    // status CLI-style (no dir)
    (status $binary:literal $($arg:literal)*) => {{
        $crate::runtime::run_status($binary, [$($arg),*], None)
    }};
    // ── status array ───────────────────────────────────────────
    // status array + dir
    (status $binary:expr, [ $($arg:expr),* $(,)? ]; dir: $dir:expr) => {{
        let mut __cmd = ::std::process::Command::new($binary);
        __cmd.args([$($arg),*]);
        __cmd.current_dir($dir);
        __cmd.status()
    }};
    // status array (no dir)
    (status $binary:expr, [ $($arg:expr),* $(,)? ]) => {{
        ::std::process::Command::new($binary).args([$($arg),*]).status()
    }};
    // ── status variable ───────────────────────────────────────
    // status variable + dir
    (status $binary:expr, $args:expr; dir: $dir:expr) => {{
        let mut __cmd = ::std::process::Command::new($binary);
        __cmd.args($args);
        __cmd.current_dir($dir);
        __cmd.status()
    }};
    // status variable (no dir)
    (status $binary:expr, $args:expr) => {{
        ::std::process::Command::new($binary).args($args).status()
    }};
    // ── try CLI-style ──────────────────────────────────────────
    // try CLI-style + dir
    (try $binary:literal $($arg:literal)*; dir: $dir:expr) => {{
        $crate::runtime::try_from_output(
            $crate::runtime::run_output($binary, [$($arg),*], Some($dir.as_ref())),
        )
    }};
    // try CLI-style (no dir)
    (try $binary:literal $($arg:literal)*) => {{
        $crate::runtime::try_from_output(
            $crate::runtime::run_output($binary, [$($arg),*], None),
        )
    }};
    // ── try array ──────────────────────────────────────────────
    // try array + dir
    (try $binary:expr, [ $($arg:expr),* $(,)? ]; dir: $dir:expr) => {{
        let mut __cmd = ::std::process::Command::new($binary);
        __cmd.args([$($arg),*]);
        __cmd.current_dir($dir);
        $crate::runtime::try_from_output(__cmd.output())
    }};
    // try array (no dir)
    (try $binary:expr, [ $($arg:expr),* $(,)? ]) => {{
        $crate::runtime::try_from_output(
            ::std::process::Command::new($binary).args([$($arg),*]).output(),
        )
    }};
    // ── try variable ───────────────────────────────────────────
    // try variable + dir
    (try $binary:expr, $args:expr; dir: $dir:expr) => {{
        let mut __cmd = ::std::process::Command::new($binary);
        __cmd.args($args);
        __cmd.current_dir($dir);
        $crate::runtime::try_from_output(__cmd.output())
    }};
    // try variable (no dir)
    (try $binary:expr, $args:expr) => {{
        $crate::runtime::try_from_output(
            ::std::process::Command::new($binary).args($args).output(),
        )
    }};
    // ── default output CLI-style ──────────────────────────────
    // CLI-style + dir
    ($binary:literal $($arg:literal)*; dir: $dir:expr) => {{
        $crate::runtime::run_output($binary, [$($arg),*], Some($dir.as_ref()))
    }};
    // CLI-style (no dir)
    ($binary:literal $($arg:literal)*) => {{
        $crate::runtime::run_output($binary, [$($arg),*], None)
    }};
    // ── default output with child environment ────────────────
    // expression args + dir + environment
    ($binary:expr, $args:expr; dir: $dir:expr, env: $env:expr) => {{
        $crate::runtime::run_output_with_env($binary, $args, Some($dir.as_ref()), $env)
    }};
    // expression args + environment
    ($binary:expr, $args:expr; env: $env:expr) => {{
        $crate::runtime::run_output_with_env($binary, $args, None, $env)
    }};
    // ── default output array ──────────────────────────────────
    // array + dir
    ($binary:expr, [ $($arg:expr),* $(,)? ]; dir: $dir:expr) => {{
        $crate::runtime::run_output($binary, [$($arg),*], Some($dir.as_ref()))
    }};
    // array (no dir)
    ($binary:expr, [ $($arg:expr),* $(,)? ]) => {{
        $crate::runtime::run_output($binary, [$($arg),*], None)
    }};
    // ── default output variable ───────────────────────────────
    // variable + dir
    ($binary:expr, $args:expr; dir: $dir:expr) => {{
        $crate::runtime::run_output($binary, $args, Some($dir.as_ref()))
    }};
    // variable (no dir)
    ($binary:expr, $args:expr) => {{
        $crate::runtime::run_output($binary, $args, None)
    }};
}
/// Build a [`Vec<OsString>`] of command arguments with automatic `.into()` conversion
/// and `..` spread for conditional sub-vectors.
///
/// Eliminates the `[vec!["x".to_string()], vec![y], ..].concat()` pattern by
/// accepting bare string literals, expressions, and spread iterables in a flat list.
///
/// # Syntax
///
/// ```ignore
/// // Literals and variables — auto-converted via Into<OsString>
/// let args = args!["--name", container_name, "--url", url];
///
/// // Spread — injects all items from an IntoIterator
/// let extra = vec!["--gpus", "all"];
/// let args = args!["run", "--detach", ..extra, "image"];
///
/// // Empty
/// let args: Vec<OsString> = args![];
/// ```
///
/// Internally composes iterator chains and collects — no mutable state.
#[macro_export]
macro_rules! args {
    // Spread with trailing items
    (@build $acc:expr; .. $item:expr, $($rest:tt)*) => {
        $crate::args!(@build ($acc.chain($item.into_iter().map(|s| s.into()))); $($rest)*)
    };
    // Spread, last item
    (@build $acc:expr; .. $item:expr) => {
        $acc.chain($item.into_iter().map(|s| s.into())).collect::<Vec<std::ffi::OsString>>()
    };
    // 2-tuple (key, value) — trailing items follow
    (@build $acc:expr; ($k:expr, $v:expr), $($rest:tt)*) => {
        $crate::args!(@build ($acc.chain(core::iter::once($k.into())).chain(core::iter::once($v.into()))); $($rest)*)
    };
    // 2-tuple (key, value) — last item
    (@build $acc:expr; ($k:expr, $v:expr)) => {
        $acc.chain(core::iter::once($k.into())).chain(core::iter::once($v.into())).collect::<Vec<std::ffi::OsString>>()
    };
    // Regular expression, trailing items follow
    (@build $acc:expr; $item:expr, $($rest:tt)*) => {
        $crate::args!(@build ($acc.chain(core::iter::once($item.into()))); $($rest)*)
    };
    // Regular expression, last item
    (@build $acc:expr; $item:expr) => {
        $acc.chain(core::iter::once($item.into())).collect::<Vec<std::ffi::OsString>>()
    };
    // Base case — no more tokens
    (@build $acc:expr;) => {
        $acc.collect::<Vec<std::ffi::OsString>>()
    };
    // Entry point
    ($($tt:tt)*) => {
        $crate::args!(@build (core::iter::empty::<std::ffi::OsString>()); $($tt)*)
    };
}
/// Return whether an executable can be discovered on the process `PATH`.
pub fn command_exists(program: impl AsRef<std::ffi::OsStr>) -> bool {
    find_executable(program).is_some()
}

#[cfg(test)]
mod tests;
