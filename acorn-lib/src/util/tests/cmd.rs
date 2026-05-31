use crate::cmd;
use crate::io::command_exists;
use crate::prelude::CommandOutput;
use crate::test_utils::{echo_args, echo_bin, exit_args, exit_bin, sh_echo, stderr_args, stderr_bin};
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

fn pwsh_is_healthy() -> bool {
    if !command_exists("pwsh") {
        return false;
    }
    match cmd!(pwsh "Write-Output 'shell-pwsh-health'") {
        | Ok(output) => {
            let stdout = output.stdout();
            let stderr = output.stderr();
            output.status.success()
                && stdout.contains("shell-pwsh-health")
                && !stdout.contains("Unhandled exception")
                && !stderr.contains("Unhandled exception")
        }
        | Err(_) => false,
    }
}

#[test]
fn test_cmd_error() {
    assert!(cmd!("not-a-real-command-xyz", ["arg"]).is_err());
    assert!(cmd!(sh "not-a-real-command-xyz arg").is_err());
}
#[test]
fn test_cmd_output() {
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
fn test_cmd_shell_selection_output() {
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
fn test_cmd_shell_selection_status_and_try() {
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
#[cfg(feature = "shell-lint")]
#[test]
fn test_cmd_shell_lint_blocks_invalid_bash() {
    if command_exists("bash") {
        let result = cmd!(bash "if true; then echo hello");
        assert!(result.is_err());
    }
}
#[cfg(feature = "shell-lint")]
#[test]
fn test_cmd_configured_shell_lint_min_severity() {
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
fn test_cmd_diagnostics_at_or_above_severity() {
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
fn test_cmd_status() {
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
