use acorn_lib::schema::validate;
use pyo3::prelude::*;

/// Check if value is a valid ORCiD
///
/// See <https://orcid.org/> for more information on ORCiDs
#[pyfunction]
fn is_orcid(value: &str) -> bool {
    match validate::is_orcid(value) {
        | Ok(_) => true,
        | Err(_) => false,
    }
}
#[pymodule]
mod acorn {
    #[pymodule_export]
    use super::is_orcid;
}
