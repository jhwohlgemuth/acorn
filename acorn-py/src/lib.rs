use acorn_lib::schema;
use pyo3::prelude::*;

/// Python bindings for ACORN
#[pymodule]
fn acorn(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let python = module.py();
    let schema = PyModule::new(python, "schema")?;
    let validate = PyModule::new(python, "validate")?;
    validate.add_function(wrap_pyfunction!(is_orcid, &validate)?)?;
    let sys = python.import("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("acorn.schema", schema)?;
    modules.set_item("acorn.schema.validate", validate)?;
    Ok(())
}
/// Check if value is a valid ORCiD
///
/// See <https://orcid.org/> for more information on ORCiDs
///
/// ### Example
/// ```python
/// from acorn.schema.validate import is_orcid
///
/// assert is_orcid("https://orcid.org/0000-0002-2057-9115")
/// ```
#[pyfunction]
fn is_orcid(value: &str) -> bool {
    match schema::validate::is_orcid(value) {
        | Ok(_) => true,
        | Err(_) => false,
    }
}
