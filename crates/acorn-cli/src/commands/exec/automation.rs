//! Trusted local Rhai automation runtime.
use acorn_cmd::{CommandOutput, CommandSpec};
use acorn_host::fs::file_checksum;
use color_eyre::eyre::{eyre, Report, Result};
use rhai::{Array, Dynamic, Engine, EvalAltResult, Position};
use std::{
    env::{self, consts},
    fs::{self, read_to_string, write},
    path::{Path, PathBuf},
};

type RhaiResult<T> = core::result::Result<T, Box<EvalAltResult>>;
fn automation_engine(arguments: &[String]) -> Engine {
    let script_arguments = arguments.to_vec();
    let mut engine = Engine::new();
    engine.disable_symbol("eval");
    engine.register_fn("array_append", array_append);
    engine.register_fn("array_length", array_length);
    engine.register_fn("argument", move |index: i64, fallback: &str| {
        usize::try_from(index)
            .ok()
            .and_then(|index| script_arguments.get(index))
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    });
    engine.register_fn("copy_file", copy_file);
    engine.register_fn("contains_text", |value: &str, needle: &str| value.contains(needle));
    engine.register_fn("create_directory", create_directory);
    engine.register_fn("directory_exists", |path: &str| Path::new(path).is_dir());
    engine.register_fn("environment", |name: &str, fallback: &str| {
        env::var(name).unwrap_or_else(|_| fallback.to_string())
    });
    engine.register_fn("fail", fail);
    engine.register_fn("file_exists", |path: &str| Path::new(path).is_file());
    engine.register_fn("file_size", file_size);
    engine.register_fn("has_sha256_digest", has_sha256_digest);
    engine.register_fn("hexadecimal", hexadecimal);
    engine.register_fn("host_platform", host_platform);
    engine.register_fn("join_path", join_path);
    engine.register_fn("json", json);
    engine.register_fn("print", |message: &str| println!("{message}"));
    engine.register_fn("run", run_command);
    engine.register_fn("sha256", sha256);
    engine.register_fn("strip_prefix", strip_prefix);
    engine.register_fn("tree_matches", tree_matches);
    engine.register_fn("write_text", write_text);
    engine
}
fn array_append(mut left: Array, right: Array) -> Array {
    left.extend(right);
    left
}
fn array_length(values: Array) -> i64 {
    i64::try_from(values.len()).unwrap_or(i64::MAX)
}
fn copy_file(source: &str, destination: &str) -> RhaiResult<i64> {
    fs::copy(source, destination)
        .and_then(|bytes| i64::try_from(bytes).map_err(|_| std::io::Error::other("copied file is too large")))
        .map_err(io_error)
}
fn create_directory(path: &str) -> RhaiResult<()> {
    fs::create_dir_all(path).map_err(io_error)
}
fn fail(message: &str) -> RhaiResult<()> {
    Err(rhai_error(message))
}
fn file_size(path: &str) -> RhaiResult<i64> {
    fs::metadata(path)
        .and_then(|metadata| i64::try_from(metadata.len()).map_err(|_| std::io::Error::other("file is too large")))
        .map_err(io_error)
}
fn has_sha256_digest(value: &str) -> bool {
    value
        .rsplit_once("@sha256:")
        .is_some_and(|(_, digest)| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
fn hexadecimal(value: &str, length: i64) -> bool {
    usize::try_from(length).is_ok_and(|length| value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
fn host_platform() -> String {
    match (consts::OS, consts::ARCH) {
        | ("linux", "x86_64") => "linux-x86_64".to_string(),
        | ("linux", "aarch64") => "linux-arm64".to_string(),
        | ("macos", "aarch64") => "macos-arm64".to_string(),
        | ("windows", "x86_64") => "windows-x86_64".to_string(),
        | ("windows", "aarch64") => "windows-arm64".to_string(),
        | (system, architecture) => format!("{system}-{architecture}"),
    }
}
fn io_error(why: std::io::Error) -> Box<EvalAltResult> {
    rhai_error(&why.to_string())
}
fn join_path(base: &str, child: &str) -> String {
    Path::new(base).join(child).to_string_lossy().to_string()
}
fn json(value: Dynamic) -> RhaiResult<String> {
    rhai::serde::from_dynamic::<serde_json::Value>(&value)
        .map_err(|why| rhai_error(&why.to_string()))
        .and_then(|value| serde_json::to_string_pretty(&value).map_err(|why| rhai_error(&why.to_string())))
}
fn rhai_error(message: &str) -> Box<EvalAltResult> {
    EvalAltResult::ErrorRuntime(Dynamic::from(message.to_string()), Position::NONE).into()
}
/// Evaluate a repository-maintained automation script with explicit host functions.
pub(super) fn run(script: &Path, arguments: &[String]) -> Result<(), Report> {
    read_to_string(script)
        .map_err(|why| eyre!("Failed to read automation script `{}` — {why}", script.display()))
        .and_then(|source| {
            automation_engine(arguments)
                .eval::<Dynamic>(&source)
                .map_err(|why| eyre!("Automation script `{}` failed — {why}", script.display()))
        })
        .map(|_| ())
}
fn run_command(program: &str, arguments: Array) -> RhaiResult<String> {
    arguments
        .into_iter()
        .map(|argument| {
            argument
                .try_cast::<String>()
                .ok_or_else(|| rhai_error("command arguments must be strings"))
        })
        .collect::<RhaiResult<Vec<_>>>()
        .and_then(|arguments| {
            CommandSpec::new(program)
                .args(arguments)
                .output()
                .map_err(io_error)
                .and_then(|output| match output.status.success() {
                    | true => Ok(output.stdout()),
                    | false => Err(rhai_error(&format!("`{program}` exited with {}; {}", output.status, output.stderr()))),
                })
        })
}
fn sha256(path: &str) -> RhaiResult<String> {
    file_checksum(PathBuf::from(path), None)
        .map(|checksum| checksum.checksum_value)
        .map_err(|why| rhai_error(&why.to_string()))
}
fn strip_prefix(value: &str, prefix: &str) -> String {
    value.strip_prefix(prefix).unwrap_or(value).to_string()
}
fn tree_matches(tree: &str, names: Array) -> RhaiResult<Array> {
    names
        .into_iter()
        .map(|name| name.try_cast::<String>().ok_or_else(|| rhai_error("dependency names must be strings")))
        .collect::<RhaiResult<Vec<_>>>()
        .map(|names| {
            tree.lines()
                .filter(|line| names.iter().any(|name| line.trim_start().starts_with(&format!("{name} v"))))
                .map(|line| Dynamic::from(line.to_string()))
                .collect()
        })
}
fn write_text(path: &str, content: &str) -> RhaiResult<()> {
    write(path, content).map_err(io_error)
}

#[cfg(test)]
mod tests;
