use acorn::io::InputOutput;
use acorn::util::StringConversion;
use acorn_schema::schema::pid::patent::{CountryCode, KindCode};
use acorn_schema::schema::pid::{
    Arxiv, Patent, PersistentIdentifier, PersistentIdentifierConvert, PersistentIdentifierParse, ARK, DOI, ISBN, ORCID, ROR, SWHID,
};
use acorn_schema::schema::research_activity::ResearchActivity;
use acorn_schema::schema::{self, ContactPoint};
use acorn_schema::validation::rules;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::path::PathBuf;

macro_rules! impl_pid_wrapper {
    ($wrapper:ident, $pid_type:ty, |$value:ident| $from_string:block, [$($field:ident: $field_type:ty),+ $(,)?] $(, {$($extra:item)*})?) => {
        #[pymethods]
        impl $wrapper {
            #[staticmethod]
            fn find_all(value: String) -> Vec<$wrapper> {
                <$pid_type>::find_all(&value)
                    .iter()
                    .map(|identifier| Self::from_string(identifier.to_string()))
                    .collect()
            }
            #[staticmethod]
            fn from_string(value: String) -> $wrapper {
                let $value = value;
                $from_string
            }
            $(
                #[getter]
                fn $field(&self) -> $field_type {
                    self.$field.clone()
                }
            )+
            $($($extra)*)?
        }
    };
}

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
    pid.add_class::<PyArxiv>()?;
    pid.add_class::<PyDOI>()?;
    pid.add_class::<PyISBN>()?;
    pid.add_class::<PyPatent>()?;
    pid.add_class::<PyORCID>()?;
    pid.add_class::<PyROR>()?;
    pid.add_class::<PySWHID>()?;
    // Add functions to schema.validate module
    let validate = PyModule::new(python, "validate")?;
    validate.add_function(wrap_pyfunction!(format_ark, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_arxiv, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_doi, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_orcid, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_phone_number, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_ror, &validate)?)?;
    validate.add_function(wrap_pyfunction!(format_swhid, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_ark, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_arxiv, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_doi, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_ip, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_orcid, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_patent_identifier, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_raid, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_ror, &validate)?)?;
    validate.add_function(wrap_pyfunction!(is_swhid, &validate)?)?;
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
#[pyclass(name = "Arxiv", from_py_object)]
#[derive(Clone)]
struct PyArxiv {
    archive: String,
    identifier: String,
    schema_uri: String,
    version: String,
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
#[pyclass(name = "SWHID", from_py_object)]
#[derive(Clone)]
struct PySWHID {
    identifier: String,
    core_identifier: String,
    object_type: String,
    schema_uri: String,
    url: String,
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
impl_pid_wrapper!(
    PyDOI,
    DOI,
    |value| {
        let pid = DOI::from_string(&value);
        PyDOI {
            identifier: pid.identifier(),
            prefix: pid.prefix().unwrap_or_default(),
            suffix: pid.suffix().unwrap_or_default(),
            schema_uri: pid.schema_uri(),
        }
    },
    [identifier: String, prefix: String, schema_uri: String, suffix: String]
);
impl_pid_wrapper!(
    PyArxiv,
    Arxiv,
    |value| {
        let pid = Arxiv::from_string(value);
        PyArxiv {
            archive: pid.archive.clone().unwrap_or_default(),
            identifier: pid.identifier(),
            schema_uri: pid.schema_uri(),
            version: pid.version.clone().unwrap_or_default(),
        }
    },
    [archive: String, identifier: String, schema_uri: String, version: String]
);
impl_pid_wrapper!(
    PyISBN,
    ISBN,
    |value| {
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
    },
    [prefix_element: String, registration_group: String, publisher: String, title: String, check_digit: String],
    {
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
);
#[pymethods]
impl PyMetadata {
    #[getter]
    fn identifier(&self) -> String {
        self.identifier.clone()
    }
}
impl_pid_wrapper!(
    PyORCID,
    ORCID,
    |value| {
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
    },
    [identifier: String, schema_uri: String, check_digit: String]
);
impl_pid_wrapper!(
    PyPatent,
    Patent,
    |value| {
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
    },
    [country_code: Option<String>, kind_code: String, serial_number: String, year: Option<String>],
    {
        fn is_granted(&self) -> bool {
            let code = self.kind_code();
            KindCode::from(code).is_granted()
        }
        fn __str__(&self) -> String {
            let code = self.country_code().unwrap_or_else(|| CountryCode::default().to_string());
            format!("{} {} {}", code, self.serial_number(), self.kind_code())
        }
    }
);
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
impl_pid_wrapper!(
    PyROR,
    ROR,
    |value| {
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
    },
    [identifier: String, schema_uri: String, check_digit: String]
);
impl_pid_wrapper!(
    PySWHID,
    SWHID,
    |value| {
        let pid = SWHID::from_string(value);
        PySWHID {
            core_identifier: pid.core_identifier(),
            identifier: pid.identifier(),
            object_type: pid.object_type.as_str().to_string(),
            schema_uri: pid.schema_uri(),
            url: pid.url(),
        }
    },
    [core_identifier: String, identifier: String, object_type: String, schema_uri: String, url: String],
    {
        fn __str__(&self) -> String {
            self.identifier.clone()
        }
    }
);
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
/// Convert an arXiv identifier into its canonical format.
#[pyfunction]
fn format_arxiv(value: &str) -> String {
    Arxiv::format(value)
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
/// Convert a Software Hash Identifier into its canonical format.
#[pyfunction]
fn format_swhid(value: &str) -> String {
    SWHID::format(value)
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
    value.is_ark()
}
/// Check if a value is a valid arXiv identifier.
#[pyfunction]
fn is_arxiv(value: &str) -> bool {
    value.is_arxiv()
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
    value.is_doi()
}
/// Check if value is a valid IP address (IPv4 or IPv6)
///
/// ### Example
/// ```python
/// from acorn.schema.validate import is_ip
///
/// assert is_ip("192.168.1.1")
/// assert is_ip("::1")
/// ```
#[pyfunction]
fn is_ip(value: &str) -> bool {
    rules::ipv4(value).is_ok() || rules::ipv6(value).is_ok()
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
    value.is_orcid()
}
#[pyfunction]
fn is_patent_identifier(value: &str) -> bool {
    Patent::is_valid(value)
}
/// Check if value is a valid RAiD
#[pyfunction]
fn is_raid(value: &str) -> bool {
    value.is_raid()
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
    value.is_ror()
}
/// Check if a value is a valid Software Hash Identifier.
#[pyfunction]
fn is_swhid(value: &str) -> bool {
    value.is_swhid()
}
