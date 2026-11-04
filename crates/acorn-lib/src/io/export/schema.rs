//! Shared schema export projection for direct and routed callers.
use crate::io::config::application::ApplicationConfiguration;
use crate::io::ApiResult;
use acorn_core::util::MimeType;
use acorn_schema::{
    modelkit::kitfile,
    pid::raid::Metadata,
    research_activity::ResearchActivity,
    standard::{cff, ckan, datacite, dcat, dublin_core::DublinCoreDocument, huwise, invenio, text, Standard},
};
use color_eyre::eyre::eyre;
use schemars::{schema_for, JsonSchema};

/// Generated schema serialization for modeled metadata standards.
pub trait StandardSchema {
    /// Serialize this standard as deterministic JSON or YAML schema text.
    fn schema(self, format: &MimeType) -> ApiResult<String>;
}
impl StandardSchema for Standard {
    fn schema(self, format: &MimeType) -> ApiResult<String> {
        fn serialize<T: JsonSchema>(format: &MimeType) -> ApiResult<String> {
            let schema = schema_for!(T);
            match format {
                | MimeType::Json => serde_json::to_string_pretty(&schema).map_err(|why| eyre!("Serialize JSON schema — {why}")),
                | MimeType::Yaml => serde_norway::to_string(&schema).map_err(|why| eyre!("Serialize YAML schema — {why}")),
                | _ => Err(eyre!("Schema export supports only JSON or YAML output")),
            }
        }
        match self {
            | Standard::ApplicationConfigurationResearchEnablement => serialize::<ApplicationConfiguration>(format),
            | Standard::CitationFileFormat => serialize::<cff::Cff>(format),
            | Standard::Ckan => serialize::<ckan::Package>(format),
            | Standard::Datacite => serialize::<datacite::Record>(format),
            | Standard::Dcat => serialize::<dcat::Dataset>(format),
            | Standard::Document | Standard::Text => serialize::<text::Text>(format),
            | Standard::DublinCore => serialize::<DublinCoreDocument>(format),
            | Standard::Huwise => serialize::<huwise::Dataset>(format),
            | Standard::Invenio => serialize::<invenio::Record>(format),
            | Standard::Kitfile | Standard::ModelKit => serialize::<kitfile::Kitfile>(format),
            | Standard::Oci => Err(eyre!("OCI checking has no single generated schema")),
            | Standard::Raid => serialize::<Metadata>(format),
            | Standard::ResearchActivityData => serialize::<ResearchActivity>(format),
        }
    }
}
