#![allow(clippy::expect_used, clippy::panic)]
//! Managed process execution integration tests.
extern crate alloc;
use acorn_cmd::{
    cmd, command_string, parse_sh, CancellationToken, CaptureLimits, CommandError, CommandSpec, ExecutionOptions, OutputStream, ProcessTermination,
};
use alloc::sync::Arc;
use core::time::Duration;
use std::{
    env::{consts::EXE_SUFFIX, temp_dir},
    fs::{read, remove_file},
    io,
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, OnceLock},
    thread,
};

fn assert_descendant_stopped(path: &Path) {
    thread::sleep(Duration::from_millis(700));
    assert!(read(path).is_err(), "descendant survived managed execution");
}
fn helper() -> &'static Path {
    static HELPER: OnceLock<PathBuf> = OnceLock::new();
    HELPER
        .get_or_init(|| {
            let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/process_helper.rs");
            let output = temp_dir().join(format!("acorn-cmd-process-helper-{}{EXE_SUFFIX}", std::process::id()));
            let status = Command::new("rustc")
                .args(["--edition=2021", "-o"])
                .arg(&output)
                .arg(source)
                .status()
                .expect("compile process helper");
            assert!(status.success(), "process helper should compile");
            output
        })
        .as_path()
}
fn marker(name: &str) -> PathBuf {
    temp_dir().join(format!("acorn-cmd-{name}-{}", std::process::id()))
}
fn remove_marker(path: &Path) {
    match remove_file(path) {
        | Ok(()) => {}
        | Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        | Err(error) => panic!("remove marker {}: {error}", path.display()),
    }
}
#[test]
fn test_active_cancellation_terminates_descendants() {
    let path = marker("cancel-descendant");
    let helper = helper().to_path_buf();
    remove_marker(&path);
    let cancellation = CancellationToken::new();
    let canceller = cancellation.clone();
    let thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(75));
        canceller.cancel();
    });
    let options = ExecutionOptions::new()
        .with_cancellation(cancellation)
        .with_timeout(Duration::from_secs(5));
    let result = CommandSpec::new(helper)
        .args(["descendant".into(), path.as_os_str().to_os_string(), "500".into(), "hold-pipes".into()])
        .execute(&options)
        .expect("cancel managed command");
    thread.join().expect("join canceller");
    assert_eq!(result.termination(), ProcessTermination::Cancelled);
    assert!(result.status().is_some());
    assert_descendant_stopped(&path);
}
#[test]
fn test_bounded_capture_streams_every_raw_byte() {
    let delivered = Arc::new(Mutex::new((Vec::new(), Vec::new())));
    let sink_delivered = Arc::clone(&delivered);
    let options =
        ExecutionOptions::new()
            .with_capture_limits(CaptureLimits::new(Some(32), None))
            .with_sink(move |stream: OutputStream, bytes: &[u8]| match sink_delivered.lock() {
                | Ok(mut delivered) => {
                    match stream {
                        | OutputStream::Stderr => delivered.1.extend_from_slice(bytes),
                        | OutputStream::Stdout => delivered.0.extend_from_slice(bytes),
                    }
                    Ok(())
                }
                | Err(_) => Err(io::Error::other("sink mutex poisoned")),
            });
    let result = CommandSpec::new(helper())
        .args(["dual", "131072"])
        .execute(&options)
        .expect("capture dual streams");
    assert_eq!(result.termination(), ProcessTermination::Exited);
    assert_eq!(result.stdout().bytes().len(), 32);
    assert_eq!(result.stderr().bytes().len(), 32);
    assert_eq!(result.stdout().bytes().last(), Some(&0xff));
    assert_eq!(result.stderr().bytes().last(), Some(&0xfe));
    assert_eq!(result.stdout().truncated_bytes(), 131_041);
    assert_eq!(result.stderr().truncated_bytes(), 131_041);
    let delivered = delivered.lock().expect("delivered output");
    assert_eq!(delivered.0.len(), 131_073);
    assert_eq!(delivered.1.len(), 131_073);
    assert!(delivered.0.get(..131_072).is_some_and(|bytes| bytes.iter().all(|byte| *byte == b'o')));
    assert!(delivered.1.get(..131_072).is_some_and(|bytes| bytes.iter().all(|byte| *byte == b'e')));
}
#[test]
fn test_capture_line_limit_keeps_complete_tail() {
    let result = CommandSpec::new(helper())
        .args(["args", "a", "b", "c", "d"])
        .execute(&ExecutionOptions::new().with_capture_limits(CaptureLimits::new(None, Some(2))))
        .expect("capture line-limited output");
    assert_eq!(result.stdout().bytes(), b"1:c\n1:d\n");
    assert_eq!(result.stdout().truncated_bytes(), 8);
    assert_eq!(result.stdout().truncated_lines(), 2);
}
#[test]
fn test_checked_execute_retains_nonzero_result() {
    let error = CommandSpec::new(helper())
        .args(["exit", "42"])
        .checked_execute(&ExecutionOptions::new())
        .expect_err("nonzero exit should fail checked execution");
    assert!(matches!(error, CommandError::NonZeroExit(_)));
    let result = error.result().expect("checked result");
    assert_eq!(result.status().and_then(|status| status.code()), Some(42));
    assert_eq!(result.termination(), ProcessTermination::Exited);
}
#[test]
fn test_command_rendering_round_trips_boundaries() {
    let arguments = ["args", "", "two words", "quote'and\\slash"].map(std::ffi::OsString::from).to_vec();
    let rendered = command_string(helper(), &arguments);
    let (program, parsed) = parse_sh(&rendered).expect("parse rendered command");
    assert_eq!(Path::new(&program), helper());
    assert_eq!(parsed, ["args", "", "two words", "quote'and\\slash"]);
}
#[test]
fn test_compile_time_and_runtime_words_agree() {
    let helper = helper();
    let compile_time = cmd!("{helper} args '' 'two words' quote\\ and\\\\slash '☘'").expect("compile-time words");
    let runtime = CommandSpec::new(helper)
        .args(["args", "", "two words", "quote and\\slash", "☘"])
        .output()
        .expect("runtime words");
    assert_eq!(compile_time.stdout, runtime.stdout);
}
#[test]
fn test_missing_executable_and_working_directory_are_spawn_errors() {
    let missing_program = format!("acorn-cmd-missing-program-{}", std::process::id());
    let missing_directory = temp_dir().join(format!("acorn-cmd-missing-directory-{}", std::process::id()));
    let program_error = CommandSpec::new(missing_program)
        .execute(&ExecutionOptions::new())
        .expect_err("missing program should not spawn");
    let directory_error = CommandSpec::new(helper())
        .current_dir(missing_directory)
        .execute(&ExecutionOptions::new())
        .expect_err("missing working directory should not spawn");
    assert!(matches!(program_error, CommandError::Spawn { .. }));
    assert!(matches!(directory_error, CommandError::Spawn { .. }));
}
#[test]
fn test_natural_exit_terminates_detached_descendant() {
    let path = marker("natural-descendant");
    remove_marker(&path);
    let result = CommandSpec::new(helper())
        .args([
            "descendant".into(),
            path.as_os_str().to_os_string(),
            "500".into(),
            "no-pipes".into(),
            "exit-leader".into(),
        ])
        .execute(&ExecutionOptions::new())
        .expect("execute descendant helper");
    assert_eq!(result.termination(), ProcessTermination::Exited);
    assert_eq!(result.stdout().bytes(), b"ready\n");
    assert_descendant_stopped(&path);
}
#[test]
fn test_output_methods_remain_unlimited() {
    let output = CommandSpec::new(helper())
        .args(["dual", "1048600"])
        .output()
        .expect("capture output without managed limits");
    assert_eq!(output.stdout.len(), 1_048_601);
    assert_eq!(output.stderr.len(), 1_048_601);
}
#[test]
fn test_pre_spawn_cancellation_does_not_start_command() {
    let path = marker("pre-cancel");
    remove_marker(&path);
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let result = CommandSpec::new(helper())
        .args(["touch".into(), path.as_os_str().to_os_string()])
        .execute(&ExecutionOptions::new().with_cancellation(cancellation).with_timeout(Duration::ZERO))
        .expect("pre-spawn cancellation result");
    assert_eq!(result.termination(), ProcessTermination::Cancelled);
    assert!(result.status().is_none());
    assert_eq!(result.duration(), Duration::ZERO);
    assert!(!path.exists());
}
#[test]
fn test_runtime_parser_rejects_empty_and_malformed_input() {
    assert!(parse_sh("  ").is_err());
    assert!(parse_sh("echo 'unterminated").is_err());
}
#[test]
fn test_sink_failure_terminates_descendants() {
    let path = marker("sink-descendant");
    remove_marker(&path);
    let options = ExecutionOptions::new().with_sink(|_: OutputStream, _: &[u8]| Err(io::Error::other("stop")));
    let error = CommandSpec::new(helper())
        .args(["descendant".into(), path.as_os_str().to_os_string(), "500".into(), "hold-pipes".into()])
        .execute(&options)
        .expect_err("sink should fail");
    assert!(matches!(error, CommandError::Sink { .. }));
    assert!(error.context().is_some());
    assert_descendant_stopped(&path);
}
#[test]
fn test_sink_panic_terminates_descendants() {
    let path = marker("sink-panic-descendant");
    remove_marker(&path);
    let options = ExecutionOptions::new().with_sink(|_: OutputStream, _: &[u8]| -> io::Result<()> {
        panic!("sink panic fixture");
    });
    let error = CommandSpec::new(helper())
        .args(["descendant".into(), path.as_os_str().to_os_string(), "500".into(), "hold-pipes".into()])
        .execute(&options)
        .expect_err("sink panic should become a capture failure");
    assert!(matches!(error, CommandError::Capture { .. }));
    assert_descendant_stopped(&path);
}
#[test]
fn test_timeout_terminates_descendants() {
    let path = marker("timeout-descendant");
    remove_marker(&path);
    let options = ExecutionOptions::new().with_timeout(Duration::from_millis(75));
    let result = CommandSpec::new(helper())
        .args(["descendant".into(), path.as_os_str().to_os_string(), "500".into(), "hold-pipes".into()])
        .execute(&options)
        .expect("timeout managed command");
    assert_eq!(result.termination(), ProcessTermination::TimedOut);
    assert!(result.status().is_some());
    assert_descendant_stopped(&path);
}
