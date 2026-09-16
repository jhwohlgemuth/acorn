//! International Standard Name Identifier public SRU API.
use crate::io::api::{self, Configuration, Endpoint, Param, Params, RemoteResource, ResponseContent};
use crate::io::ApiResult;
use crate::param;
use crate::util::constants::app::DEFAULT_ISNI_DOMAIN;
use crate::util::constants::env::{ISNI_API_TOKEN, ISNI_SERVER_HOST};
use acorn_core::options::{ApiExtension, ApiOptions};
use acorn_schema::namespaces::DEFAULT_ISNI_SCHEMA_URI;
use acorn_schema::pid::{normalize, PersistentIdentifier, PersistentIdentifierParse, ISNI};
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

const MAXIMUM_RECORDS: &str = "2";
const RECORD_SCHEMA: &str = "isni-b";
const SRU_OPERATION: &str = "searchRetrieve";
const SRU_VERSION: &str = "1.1";

/// ISNI API options.
pub type Options = ApiOptions<Extension, Param>;

/// Kind of public identity represented by an ISNI record.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IdentityKind {
    /// Organization or group.
    Organisation,
    /// Person or fictional identity.
    PersonOrFiction,
}
/// ISNI-specific API option defaults.
#[derive(Clone, Debug, Default)]
pub struct Extension;
/// Public location supplied by the ISNI registry.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    /// Country code when supplied.
    pub country_code: Option<String>,
    /// Country or location name when supplied.
    pub country_name: Option<String>,
}
/// Public name supplied by the ISNI registry.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Name {
    /// Person forename.
    pub forename: Option<String>,
    /// Organization main name.
    pub main_name: Option<String>,
    /// Registry name-use classification.
    pub name_use: Option<String>,
    /// Person surname.
    pub surname: Option<String>,
}
/// Public ISNI identity metadata retained by ACORN.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    /// Canonical compact ISNI.
    pub canonical_isni: String,
    /// Registry data-confidence value.
    pub confidence: Option<String>,
    /// Public identity kind.
    pub identity_kind: IdentityKind,
    /// Public locations supplied by registry sources.
    pub locations: Vec<Location>,
    /// Deprecated identifiers merged into the canonical record.
    pub merged_isnis: Vec<String>,
    /// Public names in registry source order.
    pub names: Vec<Name>,
    /// Organization classifications.
    pub organization_types: Vec<String>,
    /// Source identifiers associated with the record.
    pub sources: Vec<SourceIdentifier>,
}
/// Complete ISNI resolver result persisted as discovery provenance.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    /// Unmodified SRU XML response.
    pub raw_xml: String,
    /// Registry record selected for the request.
    pub record: Record,
    /// Normalized identifier requested from the registry.
    pub requested_isni: String,
}
/// Registry source identifier.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceIdentifier {
    /// Registry source code.
    #[serde(alias = "sourceCode")]
    pub source: Option<String>,
    /// Identifier within the source.
    #[serde(alias = "sourceIdentifier")]
    pub value: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostic {
    message: Option<String>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostics {
    #[serde(default, rename = "diagnostic")]
    values: Vec<Diagnostic>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnquiryRecord {
    data_confidence: Option<String>,
    #[serde(default, rename = "mergedISNI")]
    merged_isnis: Vec<String>,
    organisation: Option<Organisation>,
    #[serde(rename = "isniStatus", alias = "status")]
    status: Option<String>,
    #[serde(rename = "isniUnformatted")]
    canonical_isni: Option<String>,
    person_or_fiction: Option<PersonOrFiction>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Organisation {
    #[serde(default, rename = "location")]
    locations: Vec<Location>,
    #[serde(default, rename = "organisationName")]
    names: Vec<Name>,
    #[serde(default, rename = "organisationType")]
    organization_types: Vec<String>,
    #[serde(default, rename = "source")]
    sources: Vec<SourceIdentifier>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersonOrFiction {
    #[serde(default, rename = "location")]
    locations: Vec<Location>,
    #[serde(default, rename = "personalName")]
    names: Vec<Name>,
    #[serde(default, rename = "source")]
    sources: Vec<SourceIdentifier>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecordData {
    response_record: EnquiryRecord,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Records {
    #[serde(default, rename = "record")]
    values: Vec<SruRecord>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchRetrieveResponse {
    diagnostics: Option<Diagnostics>,
    number_of_records: usize,
    records: Option<Records>,
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SruRecord {
    record_data: RecordData,
}
impl ApiExtension for Extension {
    fn default_domain() -> String {
        DEFAULT_ISNI_DOMAIN.to_string()
    }
    fn env_domain_var() -> &'static str {
        ISNI_SERVER_HOST
    }
    fn env_token_var() -> &'static str {
        ISNI_API_TOKEN
    }
}
impl EnquiryRecord {
    fn canonical_identifier(&self) -> String {
        self.canonical_isni
            .as_deref()
            .filter(|value| ISNI::is_valid(*value))
            .map(ISNI::from_string)
            .map(|value| value.identifier())
            .unwrap_or_default()
    }
    fn contains(&self, requested: &str) -> bool {
        self.canonical_identifier() == requested
            || self
                .merged_isnis
                .iter()
                .map(ISNI::from_string)
                .map(|value| value.identifier())
                .any(|value| value == requested)
    }
    fn is_assigned(&self) -> bool {
        self.status.as_deref().is_some_and(|value| value.eq_ignore_ascii_case("assigned"))
    }
    fn into_record(self) -> ApiResult<Record> {
        let canonical_isni = self.canonical_identifier();
        match (canonical_isni.is_empty(), self.person_or_fiction, self.organisation) {
            | (true, _, _) => Err(eyre!("ISNI registry response is missing a canonical identifier")),
            | (false, Some(person), _) => Ok(Record {
                canonical_isni,
                confidence: self.data_confidence,
                identity_kind: IdentityKind::PersonOrFiction,
                locations: person.locations,
                merged_isnis: normalize::<ISNI>(self.merged_isnis),
                names: person.names,
                organization_types: Vec::new(),
                sources: person.sources,
            }),
            | (false, None, Some(organisation)) => Ok(Record {
                canonical_isni,
                confidence: self.data_confidence,
                identity_kind: IdentityKind::Organisation,
                locations: organisation.locations,
                merged_isnis: normalize::<ISNI>(self.merged_isnis),
                names: organisation.names,
                organization_types: organisation.organization_types,
                sources: organisation.sources,
            }),
            | (false, None, None) => Err(eyre!("ISNI registry response is missing person or organization metadata")),
        }
    }
}
impl Record {
    /// Select a deterministic public display name.
    pub fn display_name(&self) -> Option<String> {
        let complete_person_name = |name: &Name| {
            name.forename
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .zip(name.surname.as_deref().map(str::trim).filter(|value| !value.is_empty()))
                .map(|(forename, surname)| format!("{forename} {surname}"))
        };
        self.names
            .iter()
            .filter(|name| name.name_use.as_deref().is_some_and(|value| value.eq_ignore_ascii_case("public")))
            .find_map(complete_person_name)
            .or_else(|| self.names.iter().find_map(complete_person_name))
            .or_else(|| {
                self.names
                    .iter()
                    .filter_map(|name| name.main_name.as_deref().map(str::trim))
                    .find(|value| !value.is_empty())
                    .map(str::to_string)
            })
    }
    /// Return the canonical resolver URL.
    pub fn url(&self) -> String {
        format!("{DEFAULT_ISNI_SCHEMA_URI}/{}", self.canonical_isni)
    }
}
impl SearchRetrieveResponse {
    fn select(self, requested: &str) -> ApiResult<EnquiryRecord> {
        let diagnostic = self
            .diagnostics
            .into_iter()
            .flat_map(|value| value.values)
            .find_map(|value| value.message)
            .filter(|value| !value.trim().is_empty());
        match diagnostic {
            | Some(message) => Err(eyre!("ISNI registry diagnostic — {message}")),
            | None => {
                let matches = self
                    .records
                    .into_iter()
                    .flat_map(|records| records.values)
                    .map(|record| record.record_data.response_record)
                    .filter(|record| record.is_assigned() && record.contains(requested))
                    .collect::<Vec<_>>();
                match (self.number_of_records, matches.len()) {
                    | (0, _) | (_, 0) => Err(eyre!("ISNI registry did not return an assigned record for {requested}")),
                    | (_, 1) => matches.into_iter().next().ok_or_else(|| eyre!("ISNI registry record selection failed")),
                    | _ => Err(eyre!("ISNI registry returned multiple assigned records for {requested}")),
                }
            }
        }
    }
}
/// Resolve one exact ISNI record through the public SRU API.
pub async fn record(options: &Options) -> ApiResult<Resolution> {
    let raw = options.identifier().unwrap_or_default();
    let requested = ISNI::from_string(raw).identifier();
    match ISNI::is_valid(raw) && !requested.is_empty() {
        | false => Err(eyre!("Invalid ISNI identifier: {raw}")),
        | true => {
            let query = format!("pica.isn={requested}");
            let params = Params::new()
                .with(param!(KeyValuePair, "version", SRU_VERSION))
                .with(param!(KeyValuePair, "operation", SRU_OPERATION))
                .with(param!(KeyValuePair, "query", query.as_str()))
                .with(param!(KeyValuePair, "recordSchema", RECORD_SCHEMA))
                .with(param!(KeyValuePair, "maximumRecords", MAXIMUM_RECORDS))
                .with_custom(options.params())
                .build();
            match Endpoint::from_template("isni").map(|endpoint| endpoint.with_domain(&options.domain)) {
                | Ok(endpoint) => match endpoint.invoke("record", Some(params)).await {
                    | Ok(ResponseContent::Xml(raw_xml)) => api::parse_xml::<SearchRetrieveResponse>(&raw_xml)
                        .map_err(|why| eyre!("ISNI registry returned invalid resolver metadata — {why}"))
                        .and_then(|response| response.select(&requested))
                        .and_then(EnquiryRecord::into_record)
                        .map(|record| Resolution {
                            raw_xml,
                            record,
                            requested_isni: requested,
                        }),
                    | Ok(_) => Err(eyre!("ISNI registry returned a non-XML response")),
                    | Err(why) => Err(why),
                },
                | Err(why) => Err(why),
            }
        }
    }
}
