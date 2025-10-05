use acorn_lib::schema;
use pyo3::prelude::*;

/// Check if value is a valid ORCiD
///
/// See <https://orcid.org/> for more information on ORCiDs
#[pyfunction]
fn is_orcid(value: &str) -> bool {
    match schema::validate::is_orcid(value) {
        | Ok(_) => true,
        | Err(_) => false,
    }
}
#[pymodule]
fn acorn(module: &Bound<'_, PyModule>) -> PyResult<()> {
    add_validate_submodule(module)?;
    Ok(())
}
fn add_validate_submodule(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let name = "validate";
    let submodule = PyModule::new(module.py(), name)?;
    submodule.add_function(wrap_pyfunction!(is_orcid, &submodule)?)?;
    module.py().import("sys")?
        .getattr("modules")?
        .set_item(format!("acorn.{}", name), submodule)
}