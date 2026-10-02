//! Compiles Pomsky pattern assets into Rust constants for `acorn_schema::assets::patterns`.

use fancy_regex::Regex;
use pomsky::{
    diagnose::Diagnostic,
    options::{CompileOptions, RegexFlavor},
    Expr,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const SOURCE_DIRECTORY: &str = "assets/pomsky";

fn main() {
    println!("cargo:rerun-if-changed={SOURCE_DIRECTORY}");
    if let Err(why) = generate() {
        eprintln!("Failed to compile ACORN Pomsky patterns: {why}");
        process::exit(1);
    }
}

fn compile(path: &Path) -> Result<(String, String), String> {
    fs::read_to_string(path)
        .map_err(|why| format!("cannot read {}: {why}", path.display()))
        .and_then(|source| {
            let options = CompileOptions {
                flavor: RegexFlavor::Java,
                ..CompileOptions::default()
            };
            let (compiled, diagnostics, _) = Expr::parse_and_compile(&source, options);
            match (compiled, diagnostics.is_empty()) {
                | (Some(pattern), true) => Regex::new(&pattern)
                    .map_err(|why| format!("{} emitted an invalid fancy-regex pattern {pattern:?}: {why:?}", path.display()))
                    .map(|_| pattern),
                | _ => Err(format_diagnostics(path, &diagnostics)),
            }
        })
        .and_then(|pattern| identifier(path).map(|identifier| (identifier, pattern)))
}

fn format_diagnostics(path: &Path, diagnostics: &[Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let span = diagnostic
                .span
                .range()
                .map(|range| format!("{}..{}", range.start, range.end))
                .unwrap_or_else(|| "unknown span".to_string());
            format!("{}:{span}: {}", path.display(), diagnostic.msg)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate() -> Result<(), String> {
    sources().and_then(|paths| {
        paths
            .iter()
            .try_fold(String::new(), |mut generated, path| {
                println!("cargo:rerun-if-changed={}", path.display());
                compile(path).map(|(identifier, pattern)| {
                    generated.push_str(&format!(
                        "/// Pattern compiled from `{}`.\npub const {identifier}: &str = {pattern:?};\n",
                        path.display()
                    ));
                    generated
                })
            })
            .and_then(write)
    })
}

fn identifier(path: &Path) -> Result<String, String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| {
            stem.chars()
                .enumerate()
                .all(|(index, character)| character.is_ascii_lowercase() || character.is_ascii_digit() && index > 0 || character == '_' && index > 0)
        })
        .map(|stem| format!("{}_PATTERN", stem.to_ascii_uppercase()))
        .ok_or_else(|| format!("Pomsky filename must be lowercase snake case: {}", path.display()))
}

fn sources() -> Result<Vec<PathBuf>, String> {
    fs::read_dir(SOURCE_DIRECTORY)
        .map_err(|why| format!("cannot read {SOURCE_DIRECTORY}: {why}"))
        .and_then(|entries| {
            entries
                .map(|entry| entry.map(|entry| entry.path()).map_err(|why| why.to_string()))
                .collect::<Result<Vec<_>, _>>()
        })
        .map(|mut paths| {
            paths.retain(|path| path.extension().is_some_and(|extension| extension == "pomsky"));
            paths.sort();
            paths
        })
        .and_then(|paths| match paths.is_empty() {
            | true => Err(format!("no .pomsky files found in {SOURCE_DIRECTORY}")),
            | false => Ok(paths),
        })
}

fn write(generated: String) -> Result<(), String> {
    env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| "Cargo did not provide OUT_DIR".to_string())
        .map(|directory| directory.join("patterns.rs"))
        .and_then(|path| fs::write(&path, generated).map_err(|why| format!("cannot write {}: {why}", path.display())))
}
