//! Macros

/// Execute a command and capture its output.
///
/// Simplifies `Command::new(binary).args(args).output()` patterns.
/// Thread safe — `Command` is `Send + Sync`.
///
/// Use the `status` prefix to call `.status()` instead of `.output()`.
/// Use the `sh` prefix to parse a single string as a shell command (splits on whitespace).
///
/// # Syntax
///
/// ```ignore
/// // Parse a single string (supports format! interpolation)
/// cmd!(sh "git diff-tree --no-commit-id --name-only")
/// cmd!(sh format!("git diff --name-only {}", branch))
///
/// // CLI-style with string literals (returns Result<Output, io::Error>)
/// cmd!("git" "diff-tree" "--no-commit-id" "--name-only")
///
/// // Capture output with array or variable
/// cmd!("git", ["branch", "--show-current"])
/// cmd!("git", args)
///
/// // Check exit status only (returns Result<ExitStatus, io::Error>)
/// cmd!(sh status "vale --config .vale.ini sync")
/// cmd!(status "vale" "--config" ".vale.ini" "sync")
/// cmd!(status "vale", ["--config", path, "sync"])
/// cmd!(status "vale", args)
/// ```
///
/// # Examples
///
/// ```ignore
/// use acorn::cmd;
/// use acorn::prelude::CommandOutput;
///
/// // Parse a full command string — great for interpolated values
/// let branch = "main";
/// match cmd!(sh format!("git diff --name-only {branch}")) {
///     Ok(output) if output.status.success() => {
///         println!("{}", output.stdout());
///     }
///     _ => {},
/// }
///
/// // CLI-style — space-separated string literals
/// match cmd!("git" "branch" "--show-current") {
///     Ok(output) if output.status.success() => {
///         println!("{}", output.stdout());
///     }
///     Ok(output) => eprintln!("{}", output.stderr()),
///     Err(why) => eprintln!("Error: {}", why),
/// }
///
/// // Array-style — for dynamic or mixed-type args
/// let args = vec!["diff", "--name-only", "main"];
/// match cmd!("git", args) {
///     Ok(output) if output.status.success() => {
///         println!("{}", output.stdout());
///     }
///     _ => {},
/// }
///
/// // Status-only check
/// match cmd!(sh status "vale --config .vale.ini sync") {
///     Ok(status) if status.success() => println!("synced"),
///     _ => eprintln!("sync failed"),
/// }
/// ```
#[macro_export]
macro_rules! cmd {
    // Parse string and capture output: cmd!(sh "git diff --name-only")
    (sh $cmd:expr) => {{
        let cmd_str = $cmd.to_string();
        let mut parts = cmd_str.split_whitespace();
        match parts.next() {
            | Some(binary) => {
                let args: Vec<&str> = parts.collect();
                $crate::prelude::Command::new(binary).args(args).output()
            }
            | None => Err($crate::prelude::io::Error::other("empty command string")),
        }
    }};
    // Parse string and check status: cmd!(sh status "vale --config .vale.ini sync")
    (sh status $cmd:expr) => {{
        let cmd_str = $cmd.to_string();
        let mut parts = cmd_str.split_whitespace();
        match parts.next() {
            | Some(binary) => {
                let args: Vec<&str> = parts.collect();
                $crate::prelude::Command::new(binary).args(args).status()
            }
            | None => Err($crate::prelude::io::Error::other("empty command string")),
        }
    }};
    // Status with array literal: cmd!(status binary, [arg1, arg2])
    (status $binary:expr, [ $($arg:expr),* $(,)? ]) => {{
        $crate::prelude::Command::new($binary).args([$($arg),*]).status()
    }};
    // Status with args variable: cmd!(status binary, args)
    (status $binary:expr, $args:expr) => {{
        $crate::prelude::Command::new($binary).args($args).status()
    }};
    // Status CLI-style: cmd!(status "vale" "--config" ".vale.ini" "sync")
    (status $binary:literal $($arg:literal)*) => {{
        $crate::prelude::Command::new($binary).args([$($arg),*]).status()
    }};
    // Output with array literal: cmd!(binary, [arg1, arg2])
    ($binary:expr, [ $($arg:expr),* $(,)? ]) => {{
        $crate::prelude::Command::new($binary).args([$($arg),*]).output()
    }};
    // Output with args variable: cmd!(binary, args)
    ($binary:expr, $args:expr) => {{
        $crate::prelude::Command::new($binary).args($args).output()
    }};
    // CLI-style: cmd!("git" "diff-tree" "--no-commit-id" "--name-only")
    ($binary:literal $($arg:literal)*) => {{
        $crate::prelude::Command::new($binary).args([$($arg),*]).output()
    }};
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
                "=> {} {}",
                $crate::util::Label::skip(),
                format!($msg, $($rest)*)
            )
        );
    };
    ($msg:literal) => {
        tracing::warn!("{}", format!("=> {} {}", $crate::util::Label::skip(), $msg));
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
/// # Value Syntaxes
///
/// The macro supports four different value syntaxes:
/// 1. Single value: `param!(FieldList, "fl", "family-name")`
/// 2. Single tuple: `param!(QueryPair, "filter", ("status", "inactive"))`
/// 3. Multiple tuples: `param!(QueryPair, "q", (("key1", "val1"), ("key2", "val2")))`
/// 4. Vec notation: `param!(FieldList, "fields", vec![vec!["field1"], vec!["field2"]])`
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
/// // Vec notation
/// param!(QueryPair, "q", vec![
///     vec!["affiliation-org-name", "Lyrasis"],
/// ])
/// ```
#[macro_export]
macro_rules! param {
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
