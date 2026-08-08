use acorn::io::InputOutput;
use acorn::prelude::PathBuf;
use acorn::schema::pid::patent::{CountryCode, KindCode};
use acorn::schema::pid::{Patent, PersistentIdentifier, PersistentIdentifierParse, ARK, DOI, ISBN, ORCID, ROR};
use acorn::schema::research_activity::ResearchActivity;
use acorn::schema::{self, ContactPoint};
use acorn::util::StringConversion;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Python bindings for ACORN
#[pymodule]
#[pyo3(name = "acorn")]
fn acorn_py(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let python = module.py();
    // Add classes to schema module
    let schema = PyModule::new(python, "schema")?;
    schema.add_class::<PyResearchActivity>()?;
    // Add classes to schema.pid module
    let pid = PyModule::new(python, "pid")?;
    pid.add_class::<PyDOI>()?;
    pid.add_class::<PyISBN>()?;
    pid.add_class::<PyPatent>()?;
    pid.add_class::<PyORCID>()?;
    pid.add_class::<PyROR>()?;
    // Add functions to schema.validate module
    let validate = PyModule::new(python, "validate")?;
    validate.add_function(wrap_pyfunction!(format_ark, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_doi, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_orcid, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_phone_number, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_ror, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_ark, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_doi, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_ip6, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_orcid, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_patent_identifier, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_raid, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_ror, &validate)?)?;
    // Configure submodules
    // Without this, `import acorn.schema.validate` will fail
    let sys = python.import("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("acorn.schema", schema)?;
    modules.set_item("acorn.schema.pid", pid)?;
    modules.set_item("acorn.schema.validate", validate)?;
    Ok(())
}
#[pyclass(name = "ContactPoint", from_py_object)]
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
#[pyclass(name = "DOI", from_py_object)]
#[derive(Clone)]
struct PyDOI {
    identifier: String,
    schema_uri: String,
    prefix: String,
    suffix: String,
}
#[pyclass(name = "ISBN", from_py_object)]
#[derive(Clone)]
struct PyISBN {
    prefix_element: String,
    registration_group: String,
    publisher: String,
    title: String,
    check_digit: String,
}
#[pyclass(name = "Metadata", from_py_object)]
#[derive(Clone)]
struct PyMetadata {
    identifier: String,
}
#[pyclass(name = "ORCID", from_py_object)]
#[derive(Clone)]
struct PyORCID {
    identifier: String,
    schema_uri: String,
    check_digit: String,
}
#[pyclass(name = "Patent", from_py_object)]
#[derive(Clone)]
struct PyPatent {
    country_code: Option<String>,
    kind_code: String,
    serial_number: String,
    year: Option<String>,
}
#[pyclass(name = "ResearchActivity", from_py_object)]
#[derive(Clone)]
struct PyResearchActivity {
    metadata: PyMetadata,
    title: String,
    subtitle: Option<String>,
    sections: PySections,
    contact: PyContactPoint,
}
#[pyclass(name = "ROR", from_py_object)]
#[derive(Clone)]
struct PyROR {
    identifier: String,
    schema_uri: String,
    check_digit: String,
}
#[pyclass(name = "Sections", from_py_object)]
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
impl PyDOI {
    #[staticmethod]
    fn find_all(value: String) -> Vec<PyDOI> {
        DOI::find_all(&value).iter().map(|d| PyDOI::from_string(d.clone().to_string())).collect()
    }
    #[staticmethod]
    fn from_string(value: String) -> PyDOI {
        let pid = DOI::from_string(&value);
        PyDOI {
            identifier: pid.identifier(),
            prefix: pid.prefix().unwrap_or_default(),
            suffix: pid.suffix().unwrap_or_default(),
            schema_uri: pid.schema_uri(),
        }
    }
    #[getter]
    fn identifier(&self) -> String {
        self.identifier.clone()
    }
    #[getter]
    fn prefix(&self) -> String {
        self.prefix.clone()
    }
    #[getter]
    fn schema_uri(&self) -> String {
        self.schema_uri.clone()
    }
    #[getter]
    fn suffix(&self) -> String {
        self.suffix.clone()
    }
}
#[pymethods]
impl PyISBN {
    #[staticmethod]
    fn find_all(value: String) -> Vec<PyISBN> {
        ISBN::find_all(&value)
            .iter()
            .map(|p| PyISBN::from_string(p.clone().to_string()))
            .collect()
    }
    #[staticmethod]
    fn from_string(value: String) -> PyISBN {
        let pid = ISBN::from_string(&value);
        let ISBN {
            prefix_element,
            registration_group,
            publisher,
            title,
            check_digit,
        } = pid;
        PyISBN {
            prefix_element: prefix_element.unwrap_or_default(),
            registration_group: registration_group.unwrap_or_default(),
            publisher: publisher.unwrap_or_default(),
            title: title.unwrap_or_default(),
            check_digit: check_digit.unwrap_or_default(),
        }
    }
    #[getter]
    fn prefix_element(&self) -> String {
        self.prefix_element.clone()
    }
    #[getter]
    fn registration_group(&self) -> String {
        self.registration_group.clone()
    }
    #[getter]
    fn publisher(&self) -> String {
        self.publisher.clone()
    }
    #[getter]
    fn title(&self) -> String {
        self.title.clone()
    }
    #[getter]
    fn check_digit(&self) -> String {
        self.check_digit.clone()
    }
    fn __str__(&self) -> String {
        let Self {
            prefix_element,
            registration_group,
            publisher,
            title,
            check_digit,
        } = self;
        [prefix_element, registration_group, publisher, title, check_digit]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>()
            .join("-")
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
impl PyORCID {
    #[staticmethod]
    fn find_all(value: String) -> Vec<PyORCID> {
        ORCID::find_all(&value)
            .iter()
            .map(|p| PyORCID::from_string(p.clone().to_string()))
            .collect()
    }
    #[staticmethod]
    fn from_string(value: String) -> PyORCID {
        let pid = ORCID::from_string(&value);
        let ORCID {
            identifier,
            schema_uri,
            check_digit,
        } = pid;
        PyORCID {
            identifier: identifier.unwrap_or_default(),
            schema_uri: schema_uri.unwrap_or_default(),
            check_digit: check_digit.unwrap_or_default(),
        }
    }
    #[getter]
    fn identifier(&self) -> String {
        self.identifier.clone()
    }
    #[getter]
    fn schema_uri(&self) -> String {
        self.schema_uri.clone()
    }
    #[getter]
    fn check_digit(&self) -> String {
        self.check_digit.clone()
    }
}
#[pymethods]
impl PyPatent {
    #[staticmethod]
    fn find_all(value: String) -> Vec<PyPatent> {
        Patent::find_all(&value)
            .iter()
            .map(|p| PyPatent::from_string(p.clone().to_string()))
            .collect()
    }
    #[staticmethod]
    fn from_string(value: String) -> PyPatent {
        let pid = Patent::parse(&value);
        match pid {
            | Some(value) => match value {
                | Patent::Granted {
                    country_code,
                    kind_code,
                    serial_number,
                } => PyPatent {
                    country_code: country_code.clone().map(|x| x.to_string()),
                    kind_code: kind_code.to_string(),
                    year: None,
                    serial_number,
                },
                | Patent::Application { serial_number, year } => PyPatent {
                    country_code: None,
                    kind_code: "".to_string(),
                    serial_number,
                    year,
                },
                | Patent::Publication {
                    country_code,
                    kind_code,
                    serial_number,
                    year,
                } => PyPatent {
                    country_code: country_code.clone().map(|x| x.to_string()),
                    kind_code: kind_code.to_string(),
                    serial_number,
                    year,
                },
            },
            | None => PyPatent {
                country_code: None,
                kind_code: "".to_string(),
                serial_number: String::new(),
                year: None,
            },
        }
    }
    #[getter]
    fn country_code(&self) -> Option<String> {
        self.country_code.clone()
    }
    #[getter]
    fn kind_code(&self) -> String {
        self.kind_code.clone()
    }
    #[getter]
    fn serial_number(&self) -> String {
        self.serial_number.clone()
    }
    #[getter]
    fn year(&self) -> Option<String> {
        self.year.clone()
    }
    fn is_granted(&self) -> bool {
        let code = self.kind_code();
        KindCode::from(code).is_granted()
    }
    fn __str__(&self) -> String {
        let code = self.country_code().unwrap_or_else(|| CountryCode::default().to_string());
        format!("{} {} {}", code, self.serial_number(), self.kind_code())
    }
}
#[pymethods]
impl PyResearchActivity {
    #[staticmethod]
    fn read(path: PathBuf) -> PyResult<PyResearchActivity> {
        match ResearchActivity::read(path.clone()) {
            | Ok(data) => {
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
            | Err(why) => Err(PyValueError::new_err(format!(
                "Failed to read research activity data from {} — {why}",
                path.to_absolute_path(),
            ))),
        }
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
#[pymethods]
impl PyROR {
    #[staticmethod]
    fn find_all(value: String) -> Vec<PyROR> {
        ROR::find_all(&value).iter().map(|d| PyROR::from_string(d.clone().to_string())).collect()
    }
    #[staticmethod]
    fn from_string(value: String) -> PyROR {
        let pid = ROR::from_string(&value);
        let ROR {
            identifier,
            schema_uri,
            check_digit,
        } = pid;
        PyROR {
            identifier: identifier.unwrap_or_default(),
            schema_uri: schema_uri.unwrap_or_default(),
            check_digit: check_digit.unwrap_or_default(),
        }
    }
    #[getter]
    fn identifier(&self) -> String {
        self.identifier.clone()
    }
    #[getter]
    fn schema_uri(&self) -> String {
        self.schema_uri.clone()
    }
    #[getter]
    fn check_digit(&self) -> String {
        self.check_digit.clone()
    }
}
/// Convert a ARK into a standard format
/// ### Example
/// ```python
/// from acorn.schema.validate import format_ark
///
/// valid = "https://n2t.net/ark:12148/btv1b8449691v/f29"
/// assert format_ark(valid) == valid
/// assert format_ark("ark:12148/btv1b8449691v/f29") == "ark:12148/btv1b8449691v/f29"
/// ```
#[pyfunction]
fn format_ark(value: &str) -> String {
    ARK::format(value)
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
    DOI::format(value)
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
    ORCID::format(value)
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
        | Err(err) => Err(PyValueError::new_err(err.to_string())),
    }
}
/// Convert a ROR into a standard format
/// ### Example
/// ```python
/// from acorn.schema.validate import format_ror
///
/// assert format_ror("https://ror.org/01qz5mb56") == "https://ror.org/01qz5mb56"
/// ```
#[pyfunction]
fn format_ror(value: &str) -> String {
    ROR::format(value)
}
/// Check if value is a valid ARK
///
/// See <https://arks.org/> for more information on ARKs
///
/// ### Example
/// ```python
/// from acorn.schema.validate import is_ark
///
/// assert is_ark("https://n2t.net/ark:12148/btv1b8449691v/f29")
/// ```
#[pyfunction]
fn is_ark(value: &str) -> bool {
    match schema::validate::is_ark(value) {
        | Ok(_) => true,
        | Err(_) => false,
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
#[pyfunction]
fn is_patent_identifier(value: &str) -> bool {
    match schema::validate::is_patent_identifier(value) {
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
