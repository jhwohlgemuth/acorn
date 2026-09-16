//! Restricted Rhai engine profile
use super::capability::InvocationCapability;
use super::manifest::ResourceLimits;
use super::parse_version_header;
use crate::analyzer::{Check, CheckCategory};
use crate::{check_err, check_ok};
use acorn_core::prelude::{String, Vec};
use acorn_core::util::constants::app::{
    MAX_EXEC_ARRAY_SIZE, MAX_EXEC_BYTES, MAX_EXEC_EXPR_DEPTH, MAX_EXEC_FUNCTIONS, MAX_EXEC_FUNCTION_DEPTH, MAX_EXEC_INTERNED_STRINGS,
    MAX_EXEC_MAP_SIZE, MAX_EXEC_MODULES, MAX_EXEC_OPERATIONS, MAX_EXEC_STRING_SIZE, MAX_EXEC_VARIABLES,
};
use color_eyre::eyre::{eyre, Report, Result};
use jiff::{SignedDuration, Timestamp};
use std::path::Path;

/// Runtime contract for compiling and evaluating extension source.
pub trait Runtime {
    /// Compile source using the runtime's constrained profile.
    fn compile(&self, source: &str) -> Result<rhai::AST, Report>;
    /// Check source by evaluating it with a fresh scope and supplied context.
    fn check(&self, source: &str, context: rhai::Map) -> Result<rhai::Dynamic, Report>;
    /// Evaluate a previously validated AST with a fresh scope.
    fn check_ast(&self, ast: &rhai::AST, context: rhai::Map) -> Result<rhai::Dynamic, Report>;
}
/// Restricted Rhai runtime with no registered host capabilities.
pub struct RhaiRuntime(rhai::Engine);
impl Default for RhaiRuntime {
    fn default() -> Self {
        Self::new()
    }
}
impl RhaiRuntime {
    /// Create a restricted runtime with one invocation's manifest-brokered host functions.
    pub fn with_capability(capability: &InvocationCapability, limits: &ResourceLimits) -> Self {
        let duration = SignedDuration::from_millis(i64::try_from(limits.max_runtime).unwrap_or(i64::MAX));
        let deadline = Timestamp::now().checked_add(duration).unwrap_or(Timestamp::MAX);
        let mut runtime = Self::new().set_max_operations(limits.max_operations);
        runtime
            .engine_mut()
            .on_progress(move |_| (Timestamp::now() >= deadline).then(|| "script wall-clock limit exceeded".into()));
        capability.register(runtime.engine_mut());
        runtime
    }
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
        Self::compile_source(path, content).map(|_| ())
    }
    /// Compile source against the static ABI and restricted engine profile.
    pub fn compile_source(path: &Path, content: &str) -> Result<rhai::AST, Report> {
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
                        | _ => verify_evaluate_entry_point(&ast)
                            .map_err(|why| eyre!("script `{}` entry error — {why}", path.display()))
                            .map(|()| ast),
                    }
                }),
        }
    }
    /// Create the restricted engine profile used for compilation and evaluation.
    pub fn new() -> Self {
        let mut engine = rhai::Engine::new_raw();
        engine.disable_symbol("eval");
        engine.set_max_array_size(MAX_EXEC_ARRAY_SIZE);
        engine.set_max_call_levels(MAX_EXEC_FUNCTION_DEPTH);
        engine.set_max_expr_depths(MAX_EXEC_EXPR_DEPTH, MAX_EXEC_FUNCTION_DEPTH);
        engine.set_max_functions(MAX_EXEC_FUNCTIONS);
        engine.set_max_map_size(MAX_EXEC_MAP_SIZE);
        engine.set_max_modules(MAX_EXEC_MODULES);
        engine.set_max_operations(MAX_EXEC_OPERATIONS);
        engine.set_max_string_size(MAX_EXEC_STRING_SIZE);
        engine.set_max_strings_interned(MAX_EXEC_INTERNED_STRINGS);
        engine.set_max_variables(MAX_EXEC_VARIABLES);
        Self(engine)
    }
    /// Return mutable access to the restricted Rhai engine.
    pub fn engine_mut(&mut self) -> &mut rhai::Engine {
        &mut self.0
    }
    /// Set the Rhai operation ceiling and return the updated runtime.
    pub fn set_max_operations(mut self, maximum: u64) -> Self {
        self.0.set_max_operations(maximum);
        self
    }
}
impl Runtime for RhaiRuntime {
    fn compile(&self, source: &str) -> Result<rhai::AST, Report> {
        self.0.compile(source).map_err(|why| eyre!("{why}"))
    }
    fn check(&self, source: &str, context: rhai::Map) -> Result<rhai::Dynamic, Report> {
        self.compile(source)
            .and_then(|ast| verify_evaluate_entry_point(&ast).map(|()| ast))
            .and_then(|ast| self.check_ast(&ast, context))
    }
    fn check_ast(&self, ast: &rhai::AST, context: rhai::Map) -> Result<rhai::Dynamic, Report> {
        self.0
            .call_fn::<rhai::Dynamic>(&mut rhai::Scope::new(), ast, "evaluate", (context,))
            .map_err(|why| eyre!("{why}"))
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
