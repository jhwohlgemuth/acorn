use acorn_lib::schema;
use pyo3::prelude::*;

/// Python bindings for ACORN
#[pymodule]
fn acorn(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let python = module.py();
    let schema = PyModule::new(python, "schema")?;
    let validate = PyModule::new(python, "validate")?;
    // Add functions from schema.validate module
    validate.add_function(wrap_pyfunction!(format_phone_number, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_doi, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_ip6, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_orcid, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_raid, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_ror, &validate)?)?;
    // Configure submodules
    // Without this, `import acorn.schema.validate` will fail
    let sys = python.import("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("acorn.schema", schema)?;
    modules.set_item("acorn.schema.validate", validate)?;
    Ok(())
}
/// Convert a phone number into a standard format
/// ### Example
/// ```python
/// from acorn.schema.validate import format_phone_number
///
/// assert format_phone_number("(123) 456-7890") == "123.456.7890"
/// ```
#[pyfunction]
fn format_phone_number(value: &str) -> PyResult<String> {
    match schema::validate::format_phone_number(value) {
        | Ok(value) => Ok(value),
        | Err(err) => Err(pyo3::exceptions::PyValueError::new_err(err.to_string())),
    }
}
/// Check if value is a valid DOI
/// ### Note
/// > `10.5555` is not a DOI prefix, but rather a handle prefix
///
/// See <https://doi.org/> for more information on DOIs
///
/// ### Example
/// ```python
/// from acorn.schema.validate import is_doi
///
/// assert is_doi("10.11578/dc.20250604.1")
/// ```
#[pyfunction]
fn is_doi(value: &str) -> bool {
    match schema::validate::is_doi(value) {
        | Ok(_) => true,
        | Err(_) => false,
    }
}
/// Check if value is a valid IP6 address (e.g. "2001:0db8:85a3:0000:0000:8a2e:0370:7334")
/// ### Example
/// ```python
/// from acorn.schema.validate import is_ip6
///
/// assert is_ip6("::")
/// ```
#[pyfunction]
fn is_ip6(value: &str) -> bool {
    match schema::validate::is_ip6(value) {
        | Ok(_) => true,
        | Err(_) => false,
    }
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
/// Check if value is a valid RAiD
#[pyfunction]
fn is_raid(value: &str) -> bool {
    match schema::validate::is_raid(value) {
        | Ok(_) => true,
        | Err(_) => false,
    }
}
/// Check if value is a valid Research Organization Registry (ROR) value
///
/// See <https://www.ror.org/> for more information
///
/// ### Example
/// ```python
/// from acorn.schema.validate import is_ror
///
/// assert is_ror("https://ror.org/01qz5mb56")
/// ```
#[pyfunction]
fn is_ror(value: &str) -> bool {
    match schema::validate::is_ror(value) {
        | Ok(_) => true,
        | Err(_) => false,
    }
}
