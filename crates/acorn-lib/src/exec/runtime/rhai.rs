//! Capability-free Rhai engine limits shared by extensions and prompt scripts
use acorn_core::util::constants::app::{
    MAX_EXEC_ARRAY_SIZE, MAX_EXEC_EXPR_DEPTH, MAX_EXEC_FUNCTIONS, MAX_EXEC_FUNCTION_DEPTH, MAX_EXEC_INTERNED_STRINGS, MAX_EXEC_MAP_SIZE,
    MAX_EXEC_MODULES, MAX_EXEC_OPERATIONS, MAX_EXEC_STRING_SIZE, MAX_EXEC_VARIABLES,
};

pub(crate) fn restricted_engine() -> ::rhai::Engine {
    let mut engine = ::rhai::Engine::new_raw();
    engine.disable_symbol("eval");
    engine.disable_symbol("import");
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
    engine
}
