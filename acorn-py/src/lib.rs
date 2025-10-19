use acorn_lib::schema::{self, ContactPoint, ResearchActivity};
use pyo3::prelude::*;
use std::path::PathBuf;

/// Python bindings for ACORN
#[pymodule]
fn acorn(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let python = module.py();
    let schema = PyModule::new(python, "schema")?;
    schema.add_class::<PyResearchActivity>()?;
    let validate = PyModule::new(python, "validate")?;
    // Add functions from schema.validate module
    validate.add_function(wrap_pyfunction!(format_doi, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_orcid, &validate)?)?;
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
#[pyclass(name = "ContactPoint")]
#[derive(Clone)]
struct PyContactPoint {
    job_title: String,
    given_name: String,
    family_name: String,
    email: String,
    telephone: String,
    url: String,
    organization: String,
}
#[pyclass(name = "Metadata")]
#[derive(Clone)]
struct PyMetadata {
    identifier: String,
}
#[pyclass(name = "ResearchActivity")]
struct PyResearchActivity {
    metadata: PyMetadata,
    title: String,
    subtitle: Option<String>,
    sections: PySections,
    contact: PyContactPoint,
}
#[pyclass(name = "Sections")]
#[derive(Clone)]
struct PySections {}
#[pymethods]
impl PyContactPoint {
    /// Job title (e.g., "Group Lead") of role that the contact fills related to the asscociated research activity
    #[getter]
    fn job_title(&self) -> String {
        self.job_title.clone()
    }
    /// First (given) name of contact point
    #[getter]
    fn given_name(&self) -> String {
        self.given_name.clone()
    }
    /// Last (family) name of contact point
    #[getter]
    fn family_name(&self) -> String {
        self.family_name.clone()
    }
    /// Email address of contact point
    #[getter]
    fn email(&self) -> String {
        self.email.clone()
    }
    /// Phone number of contact point
    #[getter]
    fn telephone(&self) -> String {
        self.telephone.clone()
    }
    /// Profile URL of contact point
    #[getter]
    fn url(&self) -> String {
        self.url.clone()
    }
    /// Organization of contact point
    #[getter]
    fn organization(&self) -> String {
        self.organization.clone()
    }
}
#[pymethods]
impl PyMetadata {
    #[getter]
    fn identifier(&self) -> String {
        self.identifier.clone()
    }
}
#[pymethods]
impl PyResearchActivity {
    #[staticmethod]
    fn read(path: PathBuf) -> PyResult<PyResearchActivity> {
        let data = match ResearchActivity::read(path.clone()) {
            | Some(value) => value,
            | None => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Failed to read research activity data from {}",
                    path.to_str().unwrap()
                )))
            }
        };
        let metadata = PyMetadata {
            identifier: data.meta.identifier.clone(),
        };
        let ResearchActivity { title, subtitle, .. } = data;
        let sections = PySections {};
        let ContactPoint {
            job_title,
            given_name,
            family_name,
            email,
            telephone,
            url,
            organization,
            ..
        } = data.contact;
        let contact = PyContactPoint {
            job_title,
            given_name,
            family_name,
            email,
            telephone,
            url,
            organization,
        };
        Ok(PyResearchActivity {
            metadata,
            title,
            subtitle,
            sections,
            contact,
        })
    }
    #[getter]
    fn metadata(&self) -> PyMetadata {
        self.metadata.clone()
    }
    #[getter]
    fn title(&self) -> String {
        self.title.clone()
    }
    #[getter]
    fn subtitle(&self) -> String {
        let value = match &self.subtitle {
            | Some(value) => value,
            | None => "",
        };
        value.to_string()
    }
    #[getter]
    fn sections(&self) -> PySections {
        self.sections.clone()
    }
    #[getter]
    fn contact(&self) -> PyContactPoint {
        self.contact.clone()
    }
    fn __repr__(&self) -> String {
        format!("<ResearchActivity: {}>", self.title)
    }
    fn __str__(&self) -> String {
        self.title.clone()
    }
}
/// Convert a DOI into a standard format
/// ### Example
/// ```python
/// from acorn.schema.validate import format_doi
///
/// assert format_doi("https://doi.org/10.1000/182") == "10.1000/182"
/// ```
#[pyfunction]
fn format_doi(value: &str) -> String {
    schema::pid::DOI::format(value)
}
/// Convert an ORCiD into a standard format
/// ### Example
/// ```python
/// from acorn.schema.validate import format_orcid
///
/// assert format_orcid("https://doi.org/10.1000/182") == "10.1000/182"
/// ```
#[pyfunction]
fn format_orcid(value: &str) -> String {
    schema::pid::ORCID::format(value)
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
