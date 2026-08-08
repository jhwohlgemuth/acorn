use crate::args;
use crate::cmd;
use crate::io::command_exists;
use crate::prelude::CommandOutput;
use crate::prelude::OsString;
use crate::test::utils::{echo_args, echo_bin, exit_args, exit_bin, pwsh_is_healthy, sh_echo, stderr_args, stderr_bin};
use crate::util::cmd::redact;
#[cfg(feature = "shell-lint")]
use crate::util::cmd::Severity as CmdSeverity;
#[cfg(feature = "shell-lint")]
use crate::util::constants::env::SHELL_LINT_MIN_SEVERITY;
#[cfg(feature = "shell-lint")]
use shuck_linter::{lint, AnalysisRequest, LinterSettings, Severity as LintSeverity, ShellDialect};
#[cfg(feature = "shell-lint")]
use shuck_parser::parser::Parser;
#[cfg(feature = "shell-lint")]
use temp_env::with_vars;

#[test]
fn test_redact() {
    let cases: &[(&[&str], &str)] = &[
        (&[], ""),
        (&["hello", "world"], "hello world"),
        (&["hello world", "foo"], "\"hello world\" foo"),
        (&["--api-key=mysecret"], "--api-key=[REDACTED]"),
        (&["--password=mysecret"], "--password=[REDACTED]"),
        (&["--registration-token=mysecret"], "--registration-token=[REDACTED]"),
        (&["--secret=mysecret"], "--secret=[REDACTED]"),
        (&["--token=mysecret"], "--token=[REDACTED]"),
        (&["--token", "mysecret"], "--token [REDACTED]"),
        (&["--api-key", "key123", "--token", "tok456"], "--api-key [REDACTED] --token [REDACTED]"),
        (
            &["--output", "file.txt", "--token=secret", "--verbose"],
            "--output file.txt --token=[REDACTED] --verbose",
        ),
        (&["--token"], "--token"),
        (&["--token="], "--token=[REDACTED]"),
        (&["--username", "admin", "--host=localhost"], "--username admin --host=localhost"),
        (&["--token=first", "--password=second"], "--token=[REDACTED] --password=[REDACTED]"),
        (&["--not-token=value", "--my-secret=value"], "--not-token=value --my-secret=value"),
    ];
    for (args, expected) in cases {
        let oss_args: Vec<OsString> = args.iter().map(OsString::from).collect();
        assert_eq!(redact(&oss_args).join(" "), *expected, "failed for args: {args:?}");
    }
}
mod args_macro {
    use super::*;

    #[test]
    fn test_basic() {
        let result = args!["hello", "world"];
        assert_eq!(result, vec![OsString::from("hello"), OsString::from("world")]);
    }
    #[test]
    fn test_empty() {
        let result: Vec<OsString> = args![];
        assert!(result.is_empty());
    }
    #[test]
    fn test_single() {
        let result = args!["--version"];
        assert_eq!(result, vec![OsString::from("--version")]);
    }
    #[test]
    fn test_spread_vec() {
        let extra = vec!["--gpus".to_string(), "all".to_string()];
        let result = args!["run", "--detach", ..extra, "image"];
        let expected: Vec<OsString> = ["run", "--detach", "--gpus", "all", "image"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_spread_and_trailing_comma() {
        let extra = vec!["-v"];
        let result = args!["cmd", ..extra, "arg",];
        let expected: Vec<OsString> = ["cmd", "-v", "arg"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_spread_at_end() {
        let extra = vec!["--gpus", "all"];
        let result = args!["run", ..extra];
        let expected: Vec<OsString> = ["run", "--gpus", "all"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_spread_only() {
        let extra = vec!["--gpus", "all"];
        let result = args![..extra];
        let expected: Vec<OsString> = ["--gpus", "all"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_str_refs() {
        let a = "hello";
        let b = "world";
        let result = args![a, b];
        let expected: Vec<OsString> = ["hello", "world"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_trailing_comma() {
        let result = args!["a", "b",];
        let expected: Vec<OsString> = ["a", "b"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_with_variables() {
        let name = "test".to_string();
        let result = args!["--name", name, "--verbose"];
        let expected: Vec<OsString> = ["--name", "test", "--verbose"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_tuple_key_value() {
        let result = args![("--name", "Bill")];
        let expected: Vec<OsString> = ["--name", "Bill"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_tuple_mixed() {
        let result = args!["run", ("--name", "container"), "--detach"];
        let expected: Vec<OsString> = ["run", "--name", "container", "--detach"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_tuple_with_variables() {
        let key = "--name";
        let value = "Bill".to_string();
        let result = args![(key, value)];
        let expected: Vec<OsString> = ["--name", "Bill"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_tuple_trailing_comma() {
        let result = args![("--name", "Bill"),];
        let expected: Vec<OsString> = ["--name", "Bill"].map(OsString::from).to_vec();
        assert_eq!(result, expected);
    }
}
mod cmd_macro {
    use super::*;

    #[cfg(feature = "shell-lint")]
    #[test]
    fn test_configured_shell_lint_min_severity() {
        let default = with_vars([(SHELL_LINT_MIN_SEVERITY, None::<&str>)], CmdSeverity::minimum);
        assert_eq!(default, CmdSeverity::Hint);
        let warning = with_vars([(SHELL_LINT_MIN_SEVERITY, Some("warning"))], CmdSeverity::minimum);
        assert_eq!(warning, CmdSeverity::Warning);
        let warning_alias = with_vars([(SHELL_LINT_MIN_SEVERITY, Some(" warn "))], CmdSeverity::minimum);
        assert_eq!(warning_alias, CmdSeverity::Warning);
        let error = with_vars([(SHELL_LINT_MIN_SEVERITY, Some("ERROR"))], CmdSeverity::minimum);
        assert_eq!(error, CmdSeverity::Error);
        let invalid = with_vars([(SHELL_LINT_MIN_SEVERITY, Some("unknown"))], CmdSeverity::minimum);
        assert_eq!(invalid, CmdSeverity::Hint);
    }
    #[cfg(feature = "shell-lint")]
    #[test]
    fn test_diagnostics_at_or_above_severity() {
        let source = "if true; then echo hello";
        let parse_result = Parser::with_dialect(source, ShellDialect::Bash.parser_dialect()).parse();
        let settings = LinterSettings::default().with_shell(ShellDialect::Bash);
        let base = lint(AnalysisRequest::from_parse_result(&parse_result, source, &settings))
            .first()
            .cloned()
            .expect("expected at least one diagnostic for malformed bash source");
        let mut hint = base.clone();
        hint.severity = LintSeverity::Hint;
        let mut warning = base.clone();
        warning.severity = LintSeverity::Warning;
        let mut error = base;
        error.severity = LintSeverity::Error;
        let diagnostics = vec![hint, warning, error];
        let hint_or_above = CmdSeverity::at_or_above(diagnostics.clone(), CmdSeverity::Hint);
        assert_eq!(hint_or_above.len(), 3);
        let warning_or_above = CmdSeverity::at_or_above(diagnostics.clone(), CmdSeverity::Warning);
        assert_eq!(warning_or_above.len(), 2);
        assert!(warning_or_above.iter().all(|diagnostic| diagnostic.severity >= LintSeverity::Warning));
        let error_or_above = CmdSeverity::at_or_above(diagnostics, CmdSeverity::Error);
        assert_eq!(error_or_above.len(), 1);
        assert!(error_or_above.iter().all(|diagnostic| diagnostic.severity >= LintSeverity::Error));
    }
    #[test]
    fn test_error() {
        assert!(cmd!("not-a-real-command-xyz", ["arg"]).is_err());
        assert!(cmd!(sh "not-a-real-command-xyz arg").is_err());
    }
    #[test]
    fn test_output() {
        let output = cmd!(sh sh_echo("sh-output")).expect("sh string");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "sh-output");
        let output = cmd!(sh sh_echo("one two three")).expect("sh multi-arg");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "one two three");
        let name = "world";
        let output = cmd!(sh sh_echo(&format!("hello-{name}"))).expect("sh format!");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "hello-world");
        let output = if cfg!(unix) {
            cmd!("echo", ["hello"]).expect("array literal")
        } else {
            cmd!("cmd", ["/c", "echo", "hello"]).expect("array literal")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "hello");
        let args = echo_args("world");
        let output = cmd!(echo_bin(), args).expect("args variable");
        assert!(output.status.success());
        assert_eq!(output.stdout(), "world");
        let output = if cfg!(unix) {
            cmd!("echo" "cli-style").expect("cli-style")
        } else {
            cmd!("cmd" "/c" "echo" "cli-style").expect("cli-style")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "cli-style");
        let output = cmd!(stderr_bin(), stderr_args("oops")).expect("stderr");
        if cfg!(unix) {
            assert_eq!(output.stderr(), "oops");
        } else {
            assert!(output.stderr().contains("oops") || output.stdout().contains("oops"));
        }
        let output = cmd!(exit_bin(), exit_args(42)).expect("exit code");
        assert!(!output.status.success());
    }
    #[test]
    fn test_sh_literal_splat_output() {
        let args = ["hello", "world"];
        let output = if cfg!(unix) {
            cmd!(sh "echo {args...}").expect("array splat")
        } else {
            cmd!(sh "cmd /c echo {args...}").expect("array splat")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "hello world");
        let args = vec!["vector".to_string(), "splat".to_string()];
        let output = if cfg!(unix) {
            cmd!(sh "echo {args...}").expect("vector splat")
        } else {
            cmd!(sh "cmd /c echo {args...}").expect("vector splat")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "vector splat");
        let arg1: Option<&str> = Some("optional");
        let arg2: Option<&str> = None;
        let output = if cfg!(unix) {
            cmd!(sh "echo {arg1...} {arg2...}").expect("option splat")
        } else {
            cmd!(sh "cmd /c echo {arg1...} {arg2...}").expect("option splat")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "optional");
    }
    #[test]
    fn test_sh_literal_splat_empty_iterable() {
        let args: Option<&str> = None;
        let output = if cfg!(unix) {
            cmd!(sh "true {args...}").expect("empty splat")
        } else {
            cmd!(sh "cmd /c rem {args...}").expect("empty splat")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "");
    }
    #[test]
    fn test_sh_literal_splat_status_try_and_dir() {
        let args = ["status"];
        let status = if cfg!(unix) {
            cmd!(sh status "echo {args...}").expect("splat status")
        } else {
            cmd!(sh status "cmd /c echo {args...}").expect("splat status")
        };
        assert!(status.success());
        let args = ["try"];
        let value = if cfg!(unix) {
            cmd!(try sh "echo {args...}").expect("splat try")
        } else {
            cmd!(try sh "cmd /c echo {args...}").expect("splat try")
        };
        assert_eq!(value, "try");
        let cwd = std::env::current_dir().expect("current directory");
        let args = ["dir"];
        let output = if cfg!(unix) {
            cmd!(sh "echo {args...}"; dir: cwd).expect("splat dir")
        } else {
            cmd!(sh "cmd /c echo {args...}"; dir: cwd).expect("splat dir")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "dir");
    }
    #[test]
    fn test_sh_literal_splat_single_quotes_are_literal() {
        let output = if cfg!(unix) {
            cmd!(sh "echo '{args...}'").expect("quoted splat")
        } else {
            cmd!(sh "cmd /c echo '{args...}'").expect("quoted splat")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "{args...}");
    }
    #[test]
    fn test_sh_literal_var_output() {
        let msg = "hello";
        let output = if cfg!(unix) {
            cmd!(sh "echo {msg}").expect("single var")
        } else {
            cmd!(sh "cmd /c echo {msg}").expect("single var")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "hello");
        let name = "acorn".to_string();
        let output = if cfg!(unix) {
            cmd!(sh "echo {name}").expect("var String")
        } else {
            cmd!(sh "cmd /c echo {name}").expect("var String")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "acorn");
    }
    #[test]
    fn test_sh_literal_var_status_and_try() {
        let msg = "status-check";
        let status = if cfg!(unix) {
            cmd!(sh status "echo {msg}").expect("single var")
        } else {
            cmd!(sh status "cmd /c echo {msg}").expect("single var")
        };
        assert!(status.success());
        let msg = "try-check";
        let value = if cfg!(unix) {
            cmd!(try sh "echo {msg}").expect("try var")
        } else {
            cmd!(try sh "cmd /c echo {msg}").expect("try var")
        };
        assert_eq!(value, "try-check");
    }
    #[test]
    fn test_literal_direct_exec_basic() {
        let output = if cfg!(unix) {
            cmd!("echo", ["direct"]).expect("array form works")
        } else {
            cmd!("cmd", ["/c", "echo", "direct"]).expect("array form works")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "direct");
        let output = if cfg!(unix) {
            cmd!("echo" "cli-style").expect("cli-style works")
        } else {
            cmd!("cmd" "/c" "echo" "cli-style").expect("cli-style works")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "cli-style");
    }
    #[test]
    fn test_literal_direct_exec_sh_words() {
        let msg = "interpolated";
        let output = if cfg!(unix) {
            cmd!("echo {msg}").expect("bare literal var")
        } else {
            cmd!("cmd /c echo {msg}").expect("bare literal var")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "interpolated");
        let args = ["hello", "world"];
        let output = if cfg!(unix) {
            cmd!("echo {args...}").expect("bare literal splat")
        } else {
            cmd!("cmd /c echo {args...}").expect("bare literal splat")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "hello world");
    }
    #[test]
    fn test_literal_direct_exec_status_try() {
        let msg = "status-test";
        let status = if cfg!(unix) {
            cmd!(status "echo {msg}").expect("bare status var")
        } else {
            cmd!(status "cmd /c echo {msg}").expect("bare status var")
        };
        assert!(status.success());
        let msg = "try-test";
        let value = if cfg!(unix) {
            cmd!(try "echo {msg}").expect("bare try var")
        } else {
            cmd!(try "cmd /c echo {msg}").expect("bare try var")
        };
        assert_eq!(value, "try-test");
    }
    #[test]
    fn test_literal_mixed_var_and_splat() {
        let greeting = "Hello";
        let names = ["Alice", "Bob"];
        let output = if cfg!(unix) {
            cmd!("echo {greeting} {names...}").expect("mixed var and splat")
        } else {
            cmd!("cmd /c echo {greeting} {names...}").expect("mixed var and splat")
        };
        assert!(output.status.success());
        assert_eq!(output.stdout(), "Hello Alice Bob");
    }
    #[cfg(feature = "shell-lint")]
    #[test]
    fn test_shell_lint_blocks_invalid_bash() {
        if command_exists("bash") {
            let result = cmd!(bash "if true; then echo hello");
            assert!(result.is_err());
        }
    }
    #[test]
    fn test_shell_selection_output() {
        if command_exists("bash") {
            match cmd!(status bash "echo shell-bash-check") {
                | Ok(status) if status.success() => {
                    let output = cmd!(bash "echo shell-bash-output").expect("bash output");
                    assert!(output.status.success());
                    assert_eq!(output.stdout(), "shell-bash-output");
                }
                | _ => {}
            }
        }
        if pwsh_is_healthy() {
            match cmd!(status pwsh "Write-Output 'shell-pwsh-check'") {
                | Ok(status) if status.success() => {
                    let output = cmd!(pwsh "Write-Output 'shell-pwsh-output'").expect("pwsh output");
                    assert!(output.status.success());
                    assert_eq!(output.stdout(), "shell-pwsh-output");
                }
                | _ => {}
            }
        }
    }
    #[test]
    fn test_shell_selection_status_and_try() {
        if command_exists("bash") {
            match cmd!(bash status "echo shell-bash-check") {
                | Ok(status) if status.success() => {
                    let status = cmd!(bash status "echo shell-bash-status").expect("bash status");
                    assert!(status.success());

                    let value = cmd!(try bash "echo shell-bash-try").expect("bash try");
                    assert_eq!(value, "shell-bash-try");
                }
                | _ => {}
            }
        }
        if pwsh_is_healthy() {
            match cmd!(pwsh status "Write-Output 'shell-pwsh-check'") {
                | Ok(status) if status.success() => {
                    let status = cmd!(pwsh status "Write-Output 'shell-pwsh-status'").expect("pwsh status");
                    assert!(status.success());

                    let value = cmd!(try pwsh "Write-Output 'shell-pwsh-try'").expect("pwsh try");
                    assert_eq!(value, "shell-pwsh-try");
                }
                | _ => {}
            }
        }
    }
    #[test]
    fn test_status() {
        let status = cmd!(sh status sh_echo("sh-status")).expect("sh string");
        assert!(status.success());
        let arg = "check";
        let status = cmd!(sh status sh_echo(arg)).expect("sh format!");
        assert!(status.success());
        let status = if cfg!(unix) {
            cmd!(status "echo", ["hello"]).expect("array literal")
        } else {
            cmd!(status "cmd", ["/c", "echo", "hello"]).expect("array literal")
        };
        assert!(status.success());
        let args = echo_args("hello");
        let status = cmd!(status echo_bin(), args).expect("args variable");
        assert!(status.success());
        let status = if cfg!(unix) {
            cmd!(status "echo" "status-check").expect("cli-style")
        } else {
            cmd!(status "cmd" "/c" "echo" "status-check").expect("cli-style")
        };
        assert!(status.success());
        let status = cmd!(status exit_bin(), exit_args(42)).expect("exit code");
        assert!(!status.success());
    }
}
