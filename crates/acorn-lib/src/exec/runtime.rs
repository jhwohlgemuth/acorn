//! Restricted Rhai engine profile
use super::parse_version_header;
use crate::analyzer::{Check, CheckCategory};
use crate::{check_err, check_ok};
use acorn_core::prelude::{String, Vec};
use acorn_core::util::constants::app::MAX_EXEC_BYTES;
use color_eyre::eyre::{eyre, Report, Result};
use std::path::Path;

/// Runtime contract for compiling and evaluating extension source.
pub trait Runtime {
    /// Compile source using the runtime's constrained profile.
    fn compile(&self, source: &str) -> Result<rhai::AST, Report>;
    /// Check source by evaluating it with a fresh scope and supplied context.
    fn check(&self, source: &str, context: rhai::Map) -> Result<rhai::Dynamic, Report>;
}
/// Restricted Rhai runtime with no registered host capabilities.
pub struct RhaiRuntime(rhai::Engine);
impl Default for RhaiRuntime {
    fn default() -> Self {
        Self::new()
    }
}
impl RhaiRuntime {
    /// Check source without evaluating it.
    pub fn check_safe(path: &Path, content: &str) -> Vec<Check> {
        let validation = Self::validate_source(path, content);
        match validation {
            | Ok(()) => vec![check_ok!(
                CheckCategory::Quality,
                uri: path.display().to_string(),
                message: "Rhai static check passed".to_string()
            )],
            | Err(why) => vec![check_err!(
                CheckCategory::Quality,
                uri: path.display().to_string(),
                message: why.to_string()
            )],
        }
    }
    /// Validate source against the static ABI and restricted engine profile without evaluating it.
    pub fn validate_source(path: &Path, content: &str) -> Result<(), Report> {
        match content.len() > MAX_EXEC_BYTES {
            | true => Err(eyre!("script `{}` exceeds {} bytes", path.display(), MAX_EXEC_BYTES)),
            | false => parse_version_header(content)
                .map_err(|why| eyre!("script `{}` header error — {why}", path.display()))
                .and_then(|_| {
                    Self::new()
                        .compile(content)
                        .map_err(|why| eyre!("script `{}` compile error — {why}", path.display()))
                })
                .and_then(|ast| {
                    let uses_eval = content.lines().any(|line| {
                        let line = line.trim();
                        !line.starts_with("//") && line.contains("eval(")
                    });
                    let uses_import = content.lines().any(|line| {
                        let line = line.trim();
                        !line.starts_with("//") && line.contains("import ")
                    });
                    match (uses_eval, uses_import) {
                        | (true, _) => Err(eyre!("script `{}` uses banned `eval` — disabled", path.display())),
                        | (_, true) => Err(eyre!("script `{}` uses banned `import` — disabled", path.display())),
                        | _ => verify_evaluate_entry_point(&ast).map_err(|why| eyre!("script `{}` entry error — {why}", path.display())),
                    }
                }),
        }
    }
    /// Create the restricted engine profile used for compilation and evaluation.
    pub fn new() -> Self {
        let mut engine = rhai::Engine::new_raw();
        engine.disable_symbol("eval");
        engine.set_max_operations(100_000);
        engine.set_max_string_size(64 * 1024);
        engine.set_max_array_size(1024);
        engine.set_max_map_size(1024);
        Self(engine)
    }
}
impl Runtime for RhaiRuntime {
    fn compile(&self, source: &str) -> Result<rhai::AST, Report> {
        self.0.compile(source).map_err(|why| eyre!("{why}"))
    }
    fn check(&self, source: &str, context: rhai::Map) -> Result<rhai::Dynamic, Report> {
        self.compile(source).and_then(|ast| {
            verify_evaluate_entry_point(&ast).and_then(|()| {
                self.0
                    .call_fn::<rhai::Dynamic>(&mut rhai::Scope::new(), &ast, "evaluate", (context,))
                    .map_err(|why| eyre!("{why}"))
            })
        })
    }
}
/// Check that the AST declares exactly one `evaluate` function with arity 1.
pub fn verify_evaluate_entry_point(ast: &rhai::AST) -> Result<(), Report> {
    let functions: Vec<(String, usize)> = ast.iter_functions().map(|f| (f.name.to_string(), f.params.len())).collect();
    let matches: Vec<_> = functions.iter().filter(|(name, _)| name == "evaluate").collect();
    match matches.as_slice() {
        | [] => Err(eyre!("missing required entry point `evaluate(context)` with arity 1")),
        | [(_, 1)] => Ok(()),
        | [(_, arity)] => Err(eyre!("`evaluate` must have exactly one parameter (context), found arity {arity}")),
        | _ => Err(eyre!("duplicate `evaluate` functions — exactly one is required")),
    }
}
