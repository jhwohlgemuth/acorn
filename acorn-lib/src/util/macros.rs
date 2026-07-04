//! Macros

/// Execute a command and capture its output.
///
/// Simplifies `Command::new(binary).args(args).output()` patterns.
/// Thread safe — `Command` is `Send + Sync`.
///
/// Supports several calling conventions:
///
/// | Prefix    | Returns                        | Description                             |
/// |-----------|--------------------------------|-----------------------------------------|
/// | *(none)*  | `io::Result<Output>`           | Run and capture output                  |
/// | `status`  | `io::Result<ExitStatus>`       | Exit status only (no output)            |
/// | `sh`      | `io::Result<Output>`           | Parse string, capture output            |
/// | `sh status` | `io::Result<ExitStatus>`    | Parse string, exit status only          |
/// | `bash`    | `io::Result<Output>`           | Run via `bash -c <command>`             |
/// | `pwsh`    | `io::Result<Output>`           | Run via `pwsh -NoProfile -Command ...`  |
/// | `try`     | `Result<String, String>`       | Run, return stdout or error message     |
///
/// All forms support an optional `; dir: path` suffix to set the working directory.
///
/// The `sh` variants use shell-aware word splitting (handles quoted arguments via
/// the `shell-words` crate).
///
/// # Syntax
///
/// ```ignore
/// // String form (shell-aware quoting)
/// cmd!(sh "git diff --name-only")
/// cmd!(sh format!("git diff --name-only {branch}"))
/// cmd!(sh "echo 'hello world'")   // handles quotes
/// cmd!(bash "echo 'hello world'")
/// cmd!(pwsh "Write-Output 'hello world'")
///
/// // CLI-style literals (fastest)
/// cmd!("git" "diff-tree" "--no-commit-id" "--name-only")
/// cmd!("git" "log" "--oneline"; dir: repo_path)
///
/// // Array literal (dynamic types)
/// cmd!("git", ["branch", "--show-current"])
///
/// // Variable args
/// cmd!("git", args)
///
/// // Try form — returns Result<String, String>
/// cmd!(try "git" "rev-parse" "HEAD")
/// cmd!(try "git", args)
///
/// // With working directory
/// cmd!("git" "status"; dir: project_dir)
/// cmd!(try sh "npm test"; dir: project_dir)
/// cmd!(status bash "echo hello"; dir: project_dir)
/// cmd!(try pwsh "Write-Output 'hello'"; dir: project_dir)
/// ```
///
/// # Examples
///
/// ```ignore
/// use acorn::cmd;
/// use acorn::prelude::CommandOutput;
///
/// // String form with shell-aware quoting
/// let branch = "main";
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
/// let output = cmd!(bash "echo bash-mode")?;
/// let output = cmd!(pwsh "Write-Output 'pwsh-mode'")?;
/// ```
#[macro_export]
macro_rules! cmd {
    // ── sh status ──────────────────────────────────────────────
    // sh status string + dir
    (sh status $cmd:expr; dir: $dir:expr) => {{
        match $crate::util::cmd::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::util::cmd::run_status(&binary, &args, Some($dir.as_ref())),
            Err(e) => Err(e),
        }
    }};
    // sh status string (no dir)
    (sh status $cmd:expr) => {{
        match $crate::util::cmd::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::util::cmd::run_status(&binary, &args, None),
            Err(e) => Err(e),
        }
    }};
    // ── sh output ──────────────────────────────────────────────
    // sh string + dir
    (sh $cmd:expr; dir: $dir:expr) => {{
        match $crate::util::cmd::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::util::cmd::run_output(&binary, &args, Some($dir.as_ref())),
            Err(e) => Err(e),
        }
    }};
    // sh string (no dir)
    (sh $cmd:expr) => {{
        match $crate::util::cmd::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::util::cmd::run_output(&binary, &args, None),
            Err(e) => Err(e),
        }
    }};
    // ── bash status ────────────────────────────────────────────
    // status bash string + dir
    (status bash $cmd:expr; dir: $dir:expr) => {{
        $crate::util::cmd::run_shell_status("bash", &["-c"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // status bash string (no dir)
    (status bash $cmd:expr) => {{
        $crate::util::cmd::run_shell_status("bash", &["-c"], &$cmd.to_string(), None)
    }};
    // bash status string + dir
    (bash status $cmd:expr; dir: $dir:expr) => {{
        $crate::util::cmd::run_shell_status("bash", &["-c"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // bash status string (no dir)
    (bash status $cmd:expr) => {{
        $crate::util::cmd::run_shell_status("bash", &["-c"], &$cmd.to_string(), None)
    }};
    // ── bash output ────────────────────────────────────────────
    // bash string + dir
    (bash $cmd:expr; dir: $dir:expr) => {{
        $crate::util::cmd::run_shell_output("bash", &["-c"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // bash string (no dir)
    (bash $cmd:expr) => {{
        $crate::util::cmd::run_shell_output("bash", &["-c"], &$cmd.to_string(), None)
    }};
    // ── pwsh status ────────────────────────────────────────────
    // status pwsh string + dir
    (status pwsh $cmd:expr; dir: $dir:expr) => {{
        $crate::util::cmd::run_shell_status("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // status pwsh string (no dir)
    (status pwsh $cmd:expr) => {{
        $crate::util::cmd::run_shell_status("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), None)
    }};
    // pwsh status string + dir
    (pwsh status $cmd:expr; dir: $dir:expr) => {{
        $crate::util::cmd::run_shell_status("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // pwsh status string (no dir)
    (pwsh status $cmd:expr) => {{
        $crate::util::cmd::run_shell_status("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), None)
    }};
    // ── pwsh output ────────────────────────────────────────────
    // pwsh string + dir
    (pwsh $cmd:expr; dir: $dir:expr) => {{
        $crate::util::cmd::run_shell_output("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), Some($dir.as_ref()))
    }};
    // pwsh string (no dir)
    (pwsh $cmd:expr) => {{
        $crate::util::cmd::run_shell_output("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), None)
    }};
    // ── try sh ─────────────────────────────────────────────────
    // try sh string + dir
    (try sh $cmd:expr; dir: $dir:expr) => {{
        match $crate::util::cmd::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::util::cmd::try_from_output(
                $crate::util::cmd::run_output(&binary, &args, Some($dir.as_ref())),
            ),
            Err(e) => Err(format!("{e}")),
        }
    }};
    // try sh string (no dir)
    (try sh $cmd:expr) => {{
        match $crate::util::cmd::parse_sh(&$cmd.to_string()) {
            Ok((binary, args)) => $crate::util::cmd::try_from_output(
                $crate::util::cmd::run_output(&binary, &args, None),
            ),
            Err(e) => Err(format!("{e}")),
        }
    }};
    // ── try bash ───────────────────────────────────────────────
    // try bash string + dir
    (try bash $cmd:expr; dir: $dir:expr) => {{
        $crate::util::cmd::try_from_output(
            $crate::util::cmd::run_shell_output("bash", &["-c"], &$cmd.to_string(), Some($dir.as_ref())),
        )
    }};
    // try bash string (no dir)
    (try bash $cmd:expr) => {{
        $crate::util::cmd::try_from_output(
            $crate::util::cmd::run_shell_output("bash", &["-c"], &$cmd.to_string(), None),
        )
    }};
    // ── try pwsh ───────────────────────────────────────────────
    // try pwsh string + dir
    (try pwsh $cmd:expr; dir: $dir:expr) => {{
        $crate::util::cmd::try_from_output(
            $crate::util::cmd::run_shell_output("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), Some($dir.as_ref())),
        )
    }};
    // try pwsh string (no dir)
    (try pwsh $cmd:expr) => {{
        $crate::util::cmd::try_from_output(
            $crate::util::cmd::run_shell_output("pwsh", &["-NoProfile", "-Command"], &$cmd.to_string(), None),
        )
    }};
    // ── status CLI-style ───────────────────────────────────────
    // status CLI-style + dir
    (status $binary:literal $($arg:literal)*; dir: $dir:expr) => {{
        $crate::util::cmd::run_status($binary, [$($arg),*], Some($dir.as_ref()))
    }};
    // status CLI-style (no dir)
    (status $binary:literal $($arg:literal)*) => {{
        $crate::util::cmd::run_status($binary, [$($arg),*], None)
    }};
    // ── status array ───────────────────────────────────────────
    // status array + dir
    (status $binary:expr, [ $($arg:expr),* $(,)? ]; dir: $dir:expr) => {{
        let mut __cmd = $crate::prelude::Command::new($binary);
        __cmd.args([$($arg),*]);
        __cmd.current_dir($dir);
        __cmd.status()
    }};
    // status array (no dir)
    (status $binary:expr, [ $($arg:expr),* $(,)? ]) => {{
        $crate::prelude::Command::new($binary).args([$($arg),*]).status()
    }};
    // ── status variable ───────────────────────────────────────
    // status variable + dir
    (status $binary:expr, $args:expr; dir: $dir:expr) => {{
        let mut __cmd = $crate::prelude::Command::new($binary);
        __cmd.args($args);
        __cmd.current_dir($dir);
        __cmd.status()
    }};
    // status variable (no dir)
    (status $binary:expr, $args:expr) => {{
        $crate::prelude::Command::new($binary).args($args).status()
    }};
    // ── try CLI-style ──────────────────────────────────────────
    // try CLI-style + dir
    (try $binary:literal $($arg:literal)*; dir: $dir:expr) => {{
        $crate::util::cmd::try_from_output(
            $crate::util::cmd::run_output($binary, [$($arg),*], Some($dir.as_ref())),
        )
    }};
    // try CLI-style (no dir)
    (try $binary:literal $($arg:literal)*) => {{
        $crate::util::cmd::try_from_output(
            $crate::util::cmd::run_output($binary, [$($arg),*], None),
        )
    }};
    // ── try array ──────────────────────────────────────────────
    // try array + dir
    (try $binary:expr, [ $($arg:expr),* $(,)? ]; dir: $dir:expr) => {{
        let mut __cmd = $crate::prelude::Command::new($binary);
        __cmd.args([$($arg),*]);
        __cmd.current_dir($dir);
        $crate::util::cmd::try_from_output(__cmd.output())
    }};
    // try array (no dir)
    (try $binary:expr, [ $($arg:expr),* $(,)? ]) => {{
        $crate::util::cmd::try_from_output(
            $crate::prelude::Command::new($binary).args([$($arg),*]).output(),
        )
    }};
    // ── try variable ───────────────────────────────────────────
    // try variable + dir
    (try $binary:expr, $args:expr; dir: $dir:expr) => {{
        let mut __cmd = $crate::prelude::Command::new($binary);
        __cmd.args($args);
        __cmd.current_dir($dir);
        $crate::util::cmd::try_from_output(__cmd.output())
    }};
    // try variable (no dir)
    (try $binary:expr, $args:expr) => {{
        $crate::util::cmd::try_from_output(
            $crate::prelude::Command::new($binary).args($args).output(),
        )
    }};
    // ── default output CLI-style ──────────────────────────────
    // CLI-style + dir
    ($binary:literal $($arg:literal)*; dir: $dir:expr) => {{
        $crate::util::cmd::run_output($binary, [$($arg),*], Some($dir.as_ref()))
    }};
    // CLI-style (no dir)
    ($binary:literal $($arg:literal)*) => {{
        $crate::util::cmd::run_output($binary, [$($arg),*], None)
    }};
    // ── default output array ──────────────────────────────────
    // array + dir
    ($binary:expr, [ $($arg:expr),* $(,)? ]; dir: $dir:expr) => {{
        $crate::util::cmd::run_output($binary, [$($arg),*], Some($dir.as_ref()))
    }};
    // array (no dir)
    ($binary:expr, [ $($arg:expr),* $(,)? ]) => {{
        $crate::util::cmd::run_output($binary, [$($arg),*], None)
    }};
    // ── default output variable ───────────────────────────────
    // variable + dir
    ($binary:expr, $args:expr; dir: $dir:expr) => {{
        $crate::util::cmd::run_output($binary, $args, Some($dir.as_ref()))
    }};
    // variable (no dir)
    ($binary:expr, $args:expr) => {{
        $crate::util::cmd::run_output($binary, $args, None)
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
/// Build an analyzer [`Check`] with required category/success and optional fields.
///
/// This macro wraps the existing builder API and keeps field assignment explicit.
/// It accepts any builder method name as a field key (e.g. `severity`, `message`,
/// `context`, `uri`, `status_code`, `errors`).
///
/// # Examples
///
/// ```ignore
/// use acorn::{check, analyzer::{CheckCategory, CheckSeverity}};
///
/// let ok = check!(CheckCategory::Prose, true, message: "doc-1");
/// let err = check!(
///     CheckCategory::Readability,
///     false,
///     severity: CheckSeverity::Warning,
///     message: "index.json",
///     context: "12.1",
/// );
/// ```
#[macro_export]
macro_rules! check {
    ($category:expr, $success:expr $(, $field:ident : $value:expr )* $(,)?) => {
        $crate::check!(@apply $crate::analyzer::Check::init().category($category).success($success) $(, $field : $value )*)
    };
    (@apply $builder:expr) => {
        $builder.build()
    };
    (@apply $builder:expr, $field:ident : $value:expr $(, $rest_field:ident : $rest_value:expr )* ) => {
        $crate::check!(@apply $builder.$field($value) $(, $rest_field : $rest_value )*)
    };
}

/// Build a successful [`Check`] with optional fields.
///
/// # Examples
///
/// ```ignore
/// use acorn::{check_ok, analyzer::CheckCategory};
///
/// let check = check_ok!(CheckCategory::Quality, message: "input.json");
/// ```
#[macro_export]
macro_rules! check_ok {
    ($category:expr $(, $field:ident : $value:expr )* $(,)?) => {
        $crate::check!($category, true $(, $field : $value )*)
    };
}

/// Build a failing [`Check`] with `Error` severity and optional fields.
///
/// # Examples
///
/// ```ignore
/// use acorn::{check_err, analyzer::CheckCategory};
///
/// let check = check_err!(CheckCategory::Schema, message: "invalid document");
/// ```
#[macro_export]
macro_rules! check_err {
    ($category:expr $(, $field:ident : $value:expr )* $(,)?) => {
        $crate::check!($category, false, severity: $crate::analyzer::CheckSeverity::Error $(, $field : $value )*)
    };
}
/// Logging macro for failures
#[macro_export]
macro_rules! fail {
    ($msg:literal, $($rest:tt)*) => {
        tracing::error!(
            "{}",
            format!(
                "=> {} {}",
                $crate::util::Label::fail(),
                format!($msg, $($rest)*)
            )
        );
    };
    ($msg:literal) => {
        tracing::error!("{}", format!("=> {} {}", $crate::util::Label::fail(), $msg));
    };
    ($($args:tt)*) => {
        tracing::error!($($args)*);
    };
}
/// Logging macro for skipped operations
#[macro_export]
macro_rules! skip {
    ($msg:literal, $($rest:tt)*) => {
        tracing::warn!(
            "{}",
            format!(
                "=> {}{}",
                $crate::util::Label::skip(),
                format!($msg, $($rest)*)
            )
        );
    };
    ($msg:literal) => {
        tracing::warn!("{}", format!("=> {}{}", $crate::util::Label::skip(), $msg));
    };
    ($($args:tt)*) => {
        tracing::warn!($($args)*);
    };
}
/// Creates a `Param` with the given style, name, and values.
///
/// The macro supports two calling styles:
/// - **Ident style** (shorthand): `param!(QueryPair, "q", ...)`  
///   The style is automatically qualified to `ParamStyle::QueryPair`
/// - **Path style** (explicit): `param!(ParamStyle::QueryPair, "q", ...)`  
///   Use when the full path is preferred or when `ParamStyle` is in scope
///
/// # Parameter Styles
///
/// Common body type parameters and request styles:
/// - `Header`: HTTP header (e.g., `"PRIVATE-TOKEN"`)
/// - `Body`: Request body payload (e.g., JSON or form data)
/// - `QueryPair`: URL query string parameter
/// - `FieldList`: Comma-separated field list
/// - `TemplateValue`: URI template substitution
/// - `KeyValuePair`: Key-value pair in query or body
///
/// # Value Syntaxes
///
/// The macro supports four different value syntaxes:
/// 1. Single value: `param!(FieldList, "fl", "family-name")`
/// 2. Single tuple: `param!(QueryPair, "filter", ("status", "inactive"))`
/// 3. Multiple tuples: `param!(QueryPair, "q", (("key1", "val1"), ("key2", "val2")))`
/// 4. Vec notation: `param!(FieldList, "fields", vec![vec!["field1"], vec!["field2"]])`
/// 5. Body shorthand: `param!(Body &payload)` (uses an empty key for raw curl `-d` semantics)
///
/// # Examples
///
/// ```ignore
/// // Shorthand with single value (ident style)
/// param!(FieldList, "fl", "family-name")
///
/// // Shorthand with single tuple (ident style)
/// param!(QueryPair, "filter", ("status", "inactive"))
///
/// // Shorthand with multiple tuples (ident style)
/// param!(QueryPair, "q", (
///     ("affiliation-org-name", "Lyrasis"),
///     ("ror-org-id", "\"https://ror.org/01qz5mb56\""),
/// ))
///
/// // Path style (explicit ParamStyle reference)
/// param!(ParamStyle::FieldList, "fl", "family-name")
///
/// // KeyValuePair (query string key-value pair)
/// param!(KeyValuePair, "per_page", "100")
///
/// param!(ParamStyle::KeyValuePair, "page", "2")
///
/// // Header parameter
/// param!(Header, "PRIVATE-TOKEN", &token)
///
/// // Body shorthand (raw body, no key)
/// param!(Body &payload)
///
/// // Body parameter (with key name)
/// param!(Body, "body", &payload)
///
/// // Vec notation
/// param!(QueryPair, "q", vec![
///     vec!["affiliation-org-name", "Lyrasis"],
/// ])
/// ```
#[macro_export]
macro_rules! param {
    // Body shorthand without key: param!(Body &payload)
    (Body $val:expr) => {
        $crate::io::api::Param::of_type($crate::io::api::ParamStyle::Body)
            .values(vec![vec![Some($val)]])
            .with_key("")
    };
    // Body shorthand without key (comma form): param!(Body, &payload)
    (Body, $val:expr) => {
        $crate::io::api::Param::of_type($crate::io::api::ParamStyle::Body)
            .values(vec![vec![Some($val)]])
            .with_key("")
    };
    // Ident with multiple tuples: param!(QueryPair, "q", (("a", "b"), ("c", "d")))
    ($style:ident, $name:expr, ( $( ($($val:expr),* $(,)?) ),+ $(,)? )) => {
        $crate::io::api::Param::of_type($crate::io::api::ParamStyle::$style)
            .values(vec![ $( vec![ $( Some($val) ),* ] ),* ])
            .with_key($name)
    };
    // Ident with single tuple: param!(QueryPair, "filter", ("status", "inactive"))
    ($style:ident, $name:expr, ($($val:expr),+ $(,)?)) => {
        $crate::io::api::Param::of_type($crate::io::api::ParamStyle::$style)
            .values(vec![vec![ $( Some($val) ),* ]])
            .with_key($name)
    };
    // Ident with vec notation: param!(FieldList, "fields", vec![vec!["f1"], vec!["f2"]])
    ($style:ident, $name:expr, vec![ $( vec![ $($val:expr),* $(,)? ] ),* $(,)? ]) => {
        $crate::io::api::Param::of_type($crate::io::api::ParamStyle::$style)
            .values(vec![ $( vec![ $( Some($val) ),* ] ),* ])
            .with_key($name)
    };
    // Ident with single value: param!(FieldList, "fl", "family-name")
    ($style:ident, $name:expr, $val:expr) => {
        $crate::io::api::Param::of_type($crate::io::api::ParamStyle::$style)
            .values(vec![vec![Some($val)]])
            .with_key($name)
    };
    // Path with multiple tuples: param!(ParamStyle::QueryPair, "q", (("a", "b"), ("c", "d")))
    ($style:path, $name:expr, ( $( ($($val:expr),* $(,)?) ),+ $(,)? )) => {
        $crate::io::api::Param::of_type($style)
            .values(vec![ $( vec![ $( Some($val) ),* ] ),* ])
            .with_key($name)
    };
    // Path with single tuple: param!(ParamStyle::QueryPair, "filter", ("status", "inactive"))
    ($style:path, $name:expr, ($($val:expr),+ $(,)?)) => {
        $crate::io::api::Param::of_type($style)
            .values(vec![vec![ $( Some($val) ),* ]])
            .with_key($name)
    };
    // Path with vec notation: param!(ParamStyle::FieldList, "fields", vec![vec!["f1"]])
    ($style:path, $name:expr, vec![ $( vec![ $($val:expr),* $(,)? ] ),* $(,)? ]) => {
        $crate::io::api::Param::of_type($style)
            .values(vec![ $( vec![ $( Some($val) ),* ] ),* ])
            .with_key($name)
    };
    // Path with single value: param!(ParamStyle::FieldList, "fl", "family-name")
    ($style:path, $name:expr, $val:expr) => {
        $crate::io::api::Param::of_type($style)
            .values(vec![vec![Some($val)]])
            .with_key($name)
    };
}
/// Generate a validator function that delegates to a method on the input value
///
/// Creates a public function `fn(value: &str) -> Result<(), ValidationError>` that
/// calls the given method on the input and validates the boolean result.
///
/// # Syntax
///
/// ```ignore
/// // Full form: separate function name, method name, error code, and message
/// method_validator!(
///     /// Doc comment
///     function_name,
///     method_name,
///     "error_code",
///     "Error message"
/// );
///
/// // Without message: generates "Provide a valid {code}"
/// method_validator!(
///     /// Doc comment
///     function_name,
///     method_name,
///     "error_code"
/// );
///
/// // Shorthand with message: uses function name as method name
/// method_validator!(
///     /// Doc comment
///     function_name,
///     "error_code",
///     "Error message"
/// );
///
/// // Minimal: uses function name as method name, generates default message
/// method_validator!(
///     /// Doc comment
///     function_name,
///     "error_code"
/// );
/// ```
#[macro_export]
macro_rules! method_validator {
    ($(#[$meta:meta])* $fn_name:ident, $method:ident, $code:literal, $message:literal) => {
        #[doc = concat!("Check if value is a valid ", $code)]
        $(#[$meta])*
        pub fn $fn_name(value: &str) -> Result<(), ::validator::ValidationError> {
            match value.$method() {
                | true => Ok(()),
                | _ => Err(::validator::ValidationError::new($code).with_message($message.into())),
            }
        }
    };
    ($(#[$meta:meta])* $fn_name:ident, $method:ident, $code:literal) => {
        #[doc = concat!("Check if value is a valid ", $code)]
        $(#[$meta])*
        pub fn $fn_name(value: &str) -> Result<(), ::validator::ValidationError> {
            match value.$method() {
                | true => Ok(()),
                | _ => Err(::validator::ValidationError::new($code)
                    .with_message(concat!("Provide valid ", $code).into())),
            }
        }
    };
    ($(#[$meta:meta])* $fn_name:ident, $code:literal, $message:literal) => {
        #[doc = concat!("Check if value is a valid ", $code)]
        $(#[$meta])*
        pub fn $fn_name(value: &str) -> Result<(), ::validator::ValidationError> {
            match value.$fn_name() {
                | true => Ok(()),
                | _ => Err(::validator::ValidationError::new($code).with_message($message.into())),
            }
        }
    };
    ($(#[$meta:meta])* $fn_name:ident, $code:literal) => {
        #[doc = concat!("Check if value is a valid ", $code)]
        $(#[$meta])*
        pub fn $fn_name(value: &str) -> Result<(), ::validator::ValidationError> {
            match value.$fn_name() {
                | true => Ok(()),
                | _ => Err(::validator::ValidationError::new($code)
                    .with_message(concat!("Provide valid ", $code, " value").into())),
            }
        }
    };
}
/// Generate a validator function that matches a value against a regex
///
/// Creates a public function `fn(value: &str) -> Result<(), ValidationError>` that
/// matches the input against the given regex expression.
///
/// # Syntax
/// ```ignore
/// regex_validator!(
///     /// Doc comment
///     function_name,
///     REGEX_CONSTANT,
///     "error_code",
///     "Error message"
/// );
/// ```
#[macro_export]
macro_rules! regex_validator {
    ($(#[$meta:meta])* $fn_name:ident, $regex:expr, $code:literal, $message:literal) => {
        #[doc = concat!("Check if value is a valid ", $code)]
        $(#[$meta])*
        pub fn $fn_name(value: &str) -> Result<(), ::validator::ValidationError> {
            match $regex.is_match(value) {
                | Ok(value) if value => Ok(()),
                | _ => Err(::validator::ValidationError::new($code).with_message($message.into())),
            }
        }
    };
    ($(#[$meta:meta])* $fn_name:ident, $regex:expr, $code:literal) => {
        #[doc = concat!("Check if value is a valid ", $code)]
        $(#[$meta])*
        pub fn $fn_name(value: &str) -> Result<(), ::validator::ValidationError> {
            match $regex.is_match(value) {
                | Ok(value) if value => Ok(()),
                | _ => Err(::validator::ValidationError::new($code)
                    .with_message(concat!("Provide valid ", $code).into())),
            }
        }
    };
}
/// Generate a list validator function from an existing scalar validator function
///
/// Creates a public function `fn(value: &[String]) -> Result<(), ValidationError>` that
/// validates each value with the provided scalar validator and returns the first indexed error.
///
/// # Syntax
/// ```ignore
/// list_validator!(
///     /// Doc comment
///     list_function_name,
///     scalar_function_name,
///     "error_code",
///     "Error message"
/// );
///
/// list_validator!(
///     /// Doc comment
///     list_function_name,
///     scalar_function_name,
///     "error_code"
/// );
/// ```
#[macro_export]
macro_rules! list_validator {
    ($(#[$meta:meta])* $fn_name:ident, $validator:ident, $code:literal, $message:literal) => {
        #[doc = concat!("Check if all values are valid ", $code, " entries")]
        $(#[$meta])*
        pub fn $fn_name(value: &[String]) -> Result<(), ::validator::ValidationError> {
            value
                .iter()
                .position(|x| $validator(x).is_err())
                .map(|index| {
                    let mut err = ::validator::ValidationError::new($code).with_message($message.to_string().into());
                    err.add_param("index".into(), &index);
                    err
                })
                .map_or(Ok(()), Err)
        }
    };
    ($(#[$meta:meta])* $fn_name:ident, $validator:ident, $code:literal) => {
        #[doc = concat!("Check if all values are valid ", $code, " entries")]
        $(#[$meta])*
        pub fn $fn_name(value: &[String]) -> Result<(), ::validator::ValidationError> {
            value
                .iter()
                .position(|x| $validator(x).is_err())
                .map(|index| {
                    let mut err = ::validator::ValidationError::new($code)
                        .with_message(concat!("Every ", $code, " should be valid").to_string().into());
                    err.add_param("index".into(), &index);
                    err
                })
                .map_or(Ok(()), Err)
        }
    };
}
