//! CKAN package metadata and CKAN-to-DCAT conversion.
use crate::standard::crosswalk::CrosswalkError;
#[cfg(feature = "std")]
use crate::standard::{
    crosswalk::{ConversionWarning, Crosswalk},
    datacite::RelationType,
    dcat::{self, ContactPoint, Dataset, Distribution, DocumentRef, Kind, Publisher, Relationship},
};
#[cfg(feature = "std")]
use crate::validation::rules;
use crate::validation::Validate;
use crate::OneOrMany;
use acorn_core::prelude::alloc::{format, String, ToString, Vec};
use acorn_core::util::MimeType;
#[cfg(feature = "std")]
use acorn_core::Location;
use alloc::collections::BTreeMap;
#[cfg(feature = "std")]
use ammonia::Builder;
#[cfg(feature = "std")]
use quick_xml::escape::unescape;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// A CKAN Action API response containing one package.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct ActionResponse {
    /// Error payload returned for unsuccessful actions.
    pub error: Option<Value>,
    /// Package returned for a successful action.
    pub result: Option<Package>,
    /// Whether the action succeeded.
    pub success: bool,
}
/// A person or organization named by a CKAN metadata extension.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct Agent {
    /// Email address.
    pub email: Option<String>,
    /// Persistent identifier, such as an ORCID.
    pub identifier: Option<String>,
    /// Display name.
    pub name: Option<String>,
    /// Extension-defined agent type.
    #[serde(rename = "type")]
    pub type_: Option<String>,
}
/// An arbitrary CKAN package extra.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct Extra {
    /// Extra key.
    pub key: String,
    /// Extra value.
    pub value: Value,
}
/// A CKAN group attached to a package.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct Group {
    /// Group identifier.
    pub id: Option<String>,
    /// Machine-readable name.
    pub name: Option<String>,
    /// Human-readable title.
    pub title: Option<String>,
}
/// An expanded CKAN organization.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct Organization {
    /// Organization identifier.
    pub id: Option<String>,
    /// Machine-readable name.
    pub name: Option<String>,
    /// Human-readable title.
    pub title: Option<String>,
    /// Remaining organization properties supplied by CKAN or plugins.
    #[serde(flatten)]
    pub extensions: BTreeMap<String, Value>,
}
/// A CKAN package returned by `package_show`.
#[skip_serializing_none]
#[derive(Clone, Debug, eserde::Deserialize, JsonSchema, Serialize, Validate)]
pub struct Package {
    /// Legacy author name.
    pub author: Option<String>,
    /// Legacy author email.
    pub author_email: Option<String>,
    /// Extension-defined contacts.
    #[eserde(compat)]
    pub contact: Option<Vec<Agent>>,
    /// Extension-defined contributors.
    #[eserde(compat)]
    pub contributor: Option<Vec<Agent>>,
    /// Extension-defined creators.
    #[eserde(compat)]
    pub creator: Option<Vec<Agent>>,
    /// CKAN user that created the package record.
    pub creator_user_id: Option<String>,
    /// Arbitrary CKAN extras.
    #[eserde(compat)]
    pub extras: Option<Vec<Extra>>,
    /// Groups attached to the package.
    #[serde(default)]
    #[eserde(compat)]
    pub groups: Vec<Group>,
    /// Server-assigned package identifier.
    #[validate(nonempty)]
    pub id: String,
    /// Whether CKAN considers the package open.
    pub isopen: Option<bool>,
    /// Dataset publication date supplied by an extension.
    pub issued: Option<String>,
    /// CKAN license identifier.
    pub license_id: Option<String>,
    /// CKAN license label.
    pub license_title: Option<String>,
    /// CKAN license URL.
    pub license_url: Option<String>,
    /// Legacy maintainer name.
    pub maintainer: Option<String>,
    /// Legacy maintainer email.
    pub maintainer_email: Option<String>,
    /// CKAN metadata-record creation timestamp.
    pub metadata_created: Option<String>,
    /// CKAN metadata-record modification timestamp.
    pub metadata_modified: Option<String>,
    /// Dataset modification date supplied by an extension.
    pub modified: Option<String>,
    /// Machine-readable package slug.
    #[validate(nonempty)]
    pub name: String,
    /// Dataset description, which may contain HTML.
    pub notes: Option<String>,
    /// Server-calculated resource count.
    pub num_resources: Option<u64>,
    /// Server-calculated tag count.
    pub num_tags: Option<u64>,
    /// Expanded owning organization.
    #[eserde(compat)]
    pub organization: Option<Organization>,
    /// Owning organization identifier.
    pub owner_org: Option<String>,
    /// Whether access is restricted in CKAN.
    pub private: Option<bool>,
    /// Extension-defined publishers.
    #[eserde(compat)]
    pub publisher: Option<Vec<Agent>>,
    /// Extension-defined qualified relationships.
    #[eserde(compat)]
    pub qualified_relation: Option<Vec<QualifiedRelation>>,
    /// CKAN relationships where this package is the object.
    #[serde(default)]
    #[eserde(compat)]
    pub relationships_as_object: Vec<PackageRelationship>,
    /// CKAN relationships where this package is the subject.
    #[serde(default)]
    #[eserde(compat)]
    pub relationships_as_subject: Vec<PackageRelationship>,
    /// Resources distributed with the package.
    #[serde(default)]
    #[eserde(compat)]
    pub resources: Vec<Resource>,
    /// CKAN workflow state.
    pub state: Option<String>,
    /// Package tags.
    #[serde(default)]
    #[eserde(compat)]
    pub tags: Vec<Tag>,
    /// Human-readable package title.
    #[validate(nonempty)]
    pub title: String,
    /// CKAN package type.
    #[serde(rename = "type")]
    pub type_: Option<String>,
    /// Package landing-page URL.
    pub url: Option<String>,
    /// Remaining package properties supplied by CKAN or plugins.
    #[serde(flatten)]
    #[eserde(compat)]
    pub extensions: BTreeMap<String, Value>,
}
/// A CKAN package relationship.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct PackageRelationship {
    /// Relationship label.
    #[serde(rename = "type")]
    pub type_: Option<String>,
    /// Object package identifier or URL.
    pub object: Option<String>,
    /// Subject package identifier or URL.
    pub subject: Option<String>,
}
/// A qualified relationship supplied by a CKAN extension.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct QualifiedRelation {
    /// Extension relationship label.
    #[serde(default)]
    pub relation: String,
    /// Preferred extension role label.
    #[serde(default)]
    pub role: String,
    /// Related resource URI.
    pub uri: String,
}
/// A CKAN resource belonging to a package.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct Resource {
    /// Resource creation timestamp.
    pub created: Option<String>,
    /// Whether CKAN's datastore is active for this resource.
    pub datastore_active: Option<bool>,
    /// Resource description.
    pub description: Option<String>,
    /// Display format.
    pub format: Option<String>,
    /// CKAN checksum text.
    pub hash: Option<String>,
    /// Server-assigned resource identifier.
    pub id: Option<String>,
    /// Resource modification timestamp.
    pub last_modified: Option<String>,
    /// Resource metadata modification timestamp.
    pub metadata_modified: Option<String>,
    /// IANA media type.
    pub mimetype: Option<String>,
    /// Human-readable resource name.
    pub name: Option<String>,
    /// Extension-defined resource type.
    pub resource_type: Option<String>,
    /// Resource size in bytes.
    pub size: Option<u64>,
    /// Access URL.
    pub url: Option<String>,
    /// CKAN URL storage type.
    pub url_type: Option<String>,
    /// Remaining resource properties supplied by CKAN or plugins.
    #[serde(flatten)]
    pub extensions: BTreeMap<String, Value>,
}
/// A CKAN package tag.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub struct Tag {
    /// Display label.
    pub display_name: Option<String>,
    /// Tag identifier.
    pub id: Option<String>,
    /// Keyword value.
    pub name: Option<String>,
    /// CKAN workflow state.
    pub state: Option<String>,
    /// Optional vocabulary identifier.
    pub vocabulary_id: Option<String>,
}
impl CrosswalkError {
    fn indexed(self, index: usize, line: Option<usize>) -> Self {
        let label = line.map_or_else(
            || format!("record at index {index}"),
            |line| format!("record at index {index} (line {line})"),
        );
        Self::ParseFailed(format!("Failed to parse {label} — {self}"))
    }
}
#[cfg(feature = "std")]
impl TryFrom<Agent> for dcat::Agent {
    type Error = CrosswalkError;
    fn try_from(agent: Agent) -> Result<Self, Self::Error> {
        nonempty(agent.name.as_deref())
            .map(|name| Self {
                name: Some(name),
                homepage: None,
                email: nonempty(agent.email.as_deref()),
                identifier: nonempty(agent.identifier.as_deref()).map(|identifier| {
                    let normalized = match rules::orcid(&identifier) {
                        | Ok(()) if !identifier.starts_with("http") => format!("https://orcid.org/{identifier}"),
                        | _ => identifier,
                    };
                    OneOrMany::One(normalized)
                }),
            })
            .ok_or_else(|| CrosswalkError::MissingRequiredField("CKAN agent name".to_string()))
    }
}
#[cfg(feature = "std")]
impl Crosswalk<Dataset> for Package {
    fn crosswalk(&self) -> Result<(Dataset, Vec<ConversionWarning>), CrosswalkError> {
        let (description, description_warnings) = description(self.notes.as_deref());
        let (id, landing_page, identity_warnings) = identity(self.url.as_deref());
        let (issued, issued_warnings) = date(self.issued.as_deref(), "issued");
        let (modified, modified_warnings) = date(self.modified.as_deref(), "modified");
        let (creator, creator_warnings) = self.creators();
        let (publisher, publisher_warnings) = self.publisher();
        let (contact_point, contact_warnings) = self.contacts();
        let (license, license_warnings) = self.license();
        let (qualified_relation, relation, relationship_warnings) = self.relationships();
        let (distribution, distribution_warnings) = distributions(&self.resources);
        let warnings = description_warnings
            .into_iter()
            .chain(identity_warnings)
            .chain([ConversionWarning::omitted(
                "ckan",
                "dcat",
                "name",
                "CKAN slug is not asserted as a global identifier",
            )])
            .chain(issued_warnings)
            .chain(modified_warnings)
            .chain(optional_warning(
                &self.metadata_created,
                "metadata_created",
                "Metadata-record timestamp is not a dataset date",
            ))
            .chain(optional_warning(
                &self.metadata_modified,
                "metadata_modified",
                "Metadata-record timestamp is not a dataset date",
            ))
            .chain(creator_warnings)
            .chain(publisher_warnings)
            .chain(contact_warnings)
            .chain(license_warnings)
            .chain(relationship_warnings)
            .chain(distribution_warnings)
            .chain(self.administrative_warnings())
            .chain(self.metadata_warnings())
            .chain(self.extension_warnings())
            .collect();
        let dataset = Dataset::init()
            .maybe_id(id)
            .jsonld_type("dcat:Dataset")
            .title(OneOrMany::Many(vec![self.title.trim().to_string()]))
            .maybe_description(description.map(|value| OneOrMany::Many(vec![value])))
            .identifier(OneOrMany::Many(vec![self.id.clone()]))
            .maybe_issued(issued)
            .maybe_modified(modified)
            .maybe_publisher(publisher)
            .maybe_creator(creator)
            .maybe_contact_point(contact_point)
            .maybe_keywords(self.keywords())
            .maybe_license(license)
            .maybe_landing_page(landing_page)
            .maybe_relation(relation)
            .maybe_qualified_relation(qualified_relation)
            .maybe_distribution(distribution)
            .build();
        dataset
            .validate()
            .map(|()| (dataset, warnings))
            .map_err(|error| CrosswalkError::BuildFailed(format!("Converted DCAT dataset is invalid — {error}")))
    }
}
impl Package {
    /// Parse one or many CKAN packages from JSON, JSON Lines, YAML, or ZON.
    pub fn parse(content: &str, mime: MimeType) -> Result<OneOrMany<Self>, CrosswalkError> {
        match mime {
            | MimeType::Json => serde_json::from_str(content)
                .map_err(|error| CrosswalkError::ParseFailed(error.to_string()))
                .and_then(parse_document_value),
            | MimeType::JsonLines => parse_json_lines(content),
            | MimeType::Yaml => serde_norway::from_str(content)
                .map_err(|error| CrosswalkError::ParseFailed(error.to_string()))
                .and_then(parse_document_value),
            | MimeType::Zon => super::zon::decode_value(content)
                .map_err(|error| CrosswalkError::ParseFailed(error.to_string()))
                .and_then(parse_document_value),
            | _ => Err(CrosswalkError::ParseFailed(
                "CKAN content must be JSON, JSON Lines, YAML, or ZON".to_string(),
            )),
        }
    }
}
#[cfg(feature = "std")]
impl Package {
    fn administrative_warnings(&self) -> Vec<ConversionWarning> {
        [
            (self.creator_user_id.is_some(), "creator_user_id"),
            (!self.groups.is_empty(), "groups"),
            (self.isopen.is_some(), "isopen"),
            (self.num_resources.is_some(), "num_resources"),
            (self.num_tags.is_some(), "num_tags"),
            (self.owner_org.is_some(), "owner_org"),
            (self.private.is_some(), "private"),
            (self.state.is_some(), "state"),
            (self.type_.is_some(), "type"),
        ]
        .into_iter()
        .filter(|(present, _)| *present)
        .map(|(_, field)| ConversionWarning::no_equivalent("ckan", "dcat", field))
        .collect()
    }
    fn contacts(&self) -> (Option<OneOrMany<ContactPoint>>, Vec<ConversionWarning>) {
        let fallback =
            self.contact
                .as_ref()
                .filter(|values| !values.is_empty())
                .cloned()
                .unwrap_or_else(|| match (&self.maintainer, &self.maintainer_email) {
                    | (None, None) => Vec::new(),
                    | (name, email) => vec![Agent {
                        email: email.clone(),
                        identifier: None,
                        name: name.clone(),
                        type_: None,
                    }],
                });
        let (contacts, warnings): (Vec<_>, Vec<_>) = fallback
            .iter()
            .enumerate()
            .map(|(index, contact)| {
                let name = nonempty(contact.name.as_deref());
                let email = nonempty(contact.email.as_deref()).filter(|email| rules::email(email).is_ok());
                match (name, email) {
                    | (Some(name), Some(email)) => (
                        Some(ContactPoint::Kind(Kind {
                            jsonld_type: Some("vcard:Kind".to_string()),
                            fn_: name,
                            has_email: format!("mailto:{email}"),
                            tel: None,
                            organization_name: None,
                        })),
                        None,
                    ),
                    | (None, Some(email)) => (Some(ContactPoint::Uri(format!("mailto:{email}"))), None),
                    | _ => (
                        None,
                        Some(ConversionWarning::omitted(
                            "ckan",
                            "dcat",
                            format!("contact[{index}]"),
                            "Contact requires a valid email address",
                        )),
                    ),
                }
            })
            .unzip();
        let values = contacts.into_iter().flatten().collect::<Vec<_>>();
        (option_many(values), warnings.into_iter().flatten().collect())
    }
    fn creators(&self) -> (Option<Vec<dcat::Agent>>, Vec<ConversionWarning>) {
        let source = self.creator.as_ref().filter(|values| !values.is_empty()).cloned().unwrap_or_else(|| {
            nonempty(self.author.as_deref())
                .map(|name| {
                    vec![Agent {
                        email: self.author_email.clone(),
                        identifier: None,
                        name: Some(name),
                        type_: None,
                    }]
                })
                .unwrap_or_default()
        });
        let agents = source
            .iter()
            .cloned()
            .filter_map(|agent| dcat::Agent::try_from(agent).ok())
            .collect::<Vec<_>>();
        let warnings = source
            .iter()
            .enumerate()
            .flat_map(|(index, agent)| {
                let missing_name = agent
                    .name
                    .as_deref()
                    .filter(|name| name.trim().is_empty())
                    .map(|_| ConversionWarning::omitted("ckan", "dcat", format!("creator[{index}].name"), "Creator name is empty"));
                let type_warning = agent
                    .type_
                    .as_ref()
                    .map(|_| ConversionWarning::no_equivalent("ckan", "dcat", format!("creator[{index}].type")));
                missing_name.into_iter().chain(type_warning)
            })
            .chain(
                self.contributor
                    .iter()
                    .flatten()
                    .enumerate()
                    .map(|(index, _)| ConversionWarning::no_equivalent("ckan", "dcat", format!("contributor[{index}]"))),
            )
            .collect();
        ((!agents.is_empty()).then_some(agents), warnings)
    }
}
#[cfg(feature = "std")]
impl Package {
    fn extension_warnings(&self) -> Vec<ConversionWarning> {
        let mut fields = self
            .extensions
            .iter()
            .filter(|(_, value)| meaningful(value))
            .map(|(key, _)| key.clone())
            .chain(
                self.extras
                    .iter()
                    .flatten()
                    .filter(|extra| meaningful(&extra.value))
                    .map(|extra| format!("extras.{}", extra.key)),
            )
            .collect::<Vec<_>>();
        fields.sort_by_cached_key(|field| field.to_ascii_lowercase());
        fields
            .into_iter()
            .map(|field| ConversionWarning::no_equivalent("ckan", "dcat", field))
            .collect()
    }
    fn keywords(&self) -> Option<OneOrMany<String>> {
        option_many(
            self.tags
                .iter()
                .filter_map(|tag| nonempty(tag.name.as_deref()))
                .fold(Vec::new(), |mut values, value| {
                    if !values.iter().any(|existing: &String| existing == &value) {
                        values.push(value);
                    }
                    values
                }),
        )
    }
    fn license(&self) -> (Option<String>, Vec<ConversionWarning>) {
        let license = self
            .license_url
            .as_deref()
            .map(str::trim)
            .filter(|url| rules::uri(url).is_ok())
            .map(ToString::to_string);
        let warnings = self
            .license_url
            .as_ref()
            .filter(|_| license.is_none())
            .map(|_| ConversionWarning::omitted("ckan", "dcat", "license_url", "License URL is invalid"))
            .into_iter()
            .chain(optional_warning(
                &self.license_id,
                "license_id",
                "License identifier is not promoted to an IRI",
            ))
            .chain(optional_warning(
                &self.license_title,
                "license_title",
                "License label is not promoted to an IRI",
            ))
            .collect();
        (license, warnings)
    }
    fn metadata_warnings(&self) -> Vec<ConversionWarning> {
        let custom_creators = self.creator.as_ref().is_some_and(|values| !values.is_empty());
        let custom_contacts = self.contact.as_ref().is_some_and(|values| !values.is_empty());
        let custom_publishers = self.publisher.as_ref().is_some_and(|values| !values.is_empty());
        let legacy = [
            (custom_creators && nonempty(self.author.as_deref()).is_some(), "author"),
            (custom_creators && nonempty(self.author_email.as_deref()).is_some(), "author_email"),
            (custom_contacts && nonempty(self.maintainer.as_deref()).is_some(), "maintainer"),
            (
                custom_contacts && nonempty(self.maintainer_email.as_deref()).is_some(),
                "maintainer_email",
            ),
        ]
        .into_iter()
        .filter(|(present, _)| *present)
        .map(|(_, field)| ConversionWarning::omitted("ckan", "dcat", field, "Structured extension metadata takes precedence"));
        let contacts = self.contact.iter().flatten().enumerate().flat_map(|(index, agent)| {
            [
                (nonempty(agent.identifier.as_deref()).is_some(), "identifier"),
                (nonempty(agent.type_.as_deref()).is_some(), "type"),
            ]
            .into_iter()
            .filter(move |(present, _)| *present)
            .map(move |(_, field)| ConversionWarning::no_equivalent("ckan", "dcat", format!("contact[{index}].{field}")))
        });
        let publishers = self.publisher.iter().flatten().enumerate().filter_map(|(index, agent)| {
            nonempty(agent.type_.as_deref()).map(|_| ConversionWarning::no_equivalent("ckan", "dcat", format!("publisher[{index}].type")))
        });
        let tags = self.tags.iter().enumerate().flat_map(|(index, tag)| {
            let display_name_unmapped = match (nonempty(tag.display_name.as_deref()), nonempty(tag.name.as_deref())) {
                | (Some(display), Some(name)) => display != name,
                | (Some(_), None) => true,
                | _ => false,
            };
            [
                (display_name_unmapped, "display_name"),
                (nonempty(tag.id.as_deref()).is_some(), "id"),
                (nonempty(tag.state.as_deref()).is_some(), "state"),
                (nonempty(tag.vocabulary_id.as_deref()).is_some(), "vocabulary_id"),
            ]
            .into_iter()
            .filter(move |(present, _)| *present)
            .map(move |(_, field)| ConversionWarning::no_equivalent("ckan", "dcat", format!("tags[{index}].{field}")))
        });
        let organization = self.organization.iter().flat_map(|organization| {
            let name_unmapped = custom_publishers
                || match (nonempty(organization.title.as_deref()), nonempty(organization.name.as_deref())) {
                    | (Some(title), Some(name)) => title != name,
                    | _ => false,
                };
            let title_unmapped = custom_publishers && nonempty(organization.title.as_deref()).is_some();
            let mut fields = organization
                .extensions
                .iter()
                .filter(|(_, value)| meaningful(value))
                .map(|(key, _)| format!("organization.{key}"))
                .collect::<Vec<_>>();
            fields.extend(
                [
                    (nonempty(organization.id.as_deref()).is_some(), "organization.id"),
                    (name_unmapped, "organization.name"),
                    (title_unmapped, "organization.title"),
                ]
                .into_iter()
                .filter(|(present, _)| *present)
                .map(|(_, field)| field.to_string()),
            );
            fields.sort_by_cached_key(|field| field.to_ascii_lowercase());
            fields.into_iter().map(|field| ConversionWarning::no_equivalent("ckan", "dcat", field))
        });
        legacy.chain(contacts).chain(publishers).chain(tags).chain(organization).collect()
    }
    fn publisher(&self) -> (Option<Publisher>, Vec<ConversionWarning>) {
        let source = self
            .publisher
            .as_ref()
            .and_then(|values| values.first())
            .cloned()
            .and_then(|agent| dcat::Agent::try_from(agent).ok())
            .or_else(|| {
                self.organization.as_ref().and_then(|organization| {
                    nonempty(organization.title.as_deref())
                        .or_else(|| nonempty(organization.name.as_deref()))
                        .map(|name| dcat::Agent {
                            name: Some(name),
                            homepage: None,
                            email: None,
                            identifier: None,
                        })
                })
            });
        let warnings = self
            .publisher
            .iter()
            .flatten()
            .enumerate()
            .skip(1)
            .map(|(index, _)| ConversionWarning::omitted("ckan", "dcat", format!("publisher[{index}]"), "DCAT Dataset has one publisher field"))
            .collect();
        (source.map(Publisher::Agent), warnings)
    }
    fn relationships(&self) -> (Option<Vec<Relationship>>, Option<OneOrMany<String>>, Vec<ConversionWarning>) {
        let qualified = self.qualified_relation.iter().flatten().map(QualifiedRelation::convert);
        let core = self
            .relationships_as_subject
            .iter()
            .chain(self.relationships_as_object.iter())
            .map(PackageRelationship::convert);
        let converted = qualified.chain(core).collect::<Vec<_>>();
        let relationships = converted
            .iter()
            .filter_map(|(relationship, _, _)| relationship.clone())
            .collect::<Vec<_>>();
        let relations = converted.iter().filter_map(|(_, relation, _)| relation.clone()).collect();
        let warnings = converted.into_iter().filter_map(|(_, _, warning)| warning).collect();
        ((!relationships.is_empty()).then_some(relationships), option_many(relations), warnings)
    }
}
#[cfg(feature = "std")]
impl PackageRelationship {
    fn convert(&self) -> (Option<Relationship>, Option<String>, Option<ConversionWarning>) {
        let uri = self.object.as_deref().or(self.subject.as_deref()).map(str::trim);
        let uri = uri.filter(|uri| rules::uri(uri).is_ok());
        let role = self.type_.as_deref().map(RelationType::from);
        let field = "relationships";
        match (uri, role) {
            | (Some(uri), Some(role)) if role != RelationType::Other => (
                Some(Relationship {
                    relation: uri.to_string(),
                    had_role: role,
                }),
                None,
                None,
            ),
            | (Some(uri), _) => (
                None,
                Some(uri.to_string()),
                Some(ConversionWarning::approximated(
                    "ckan",
                    "dcat",
                    field,
                    "Unknown relationship role was retained as a generic relation",
                )),
            ),
            | _ => (
                None,
                None,
                Some(ConversionWarning::omitted(
                    "ckan",
                    "dcat",
                    field,
                    "Relationship target is not an absolute HTTP(S) URI",
                )),
            ),
        }
    }
}
#[cfg(feature = "std")]
impl QualifiedRelation {
    fn convert(&self) -> (Option<Relationship>, Option<String>, Option<ConversionWarning>) {
        let role = nonempty(Some(self.role.as_str())).unwrap_or_else(|| self.relation.trim().to_string());
        match (rules::uri(&self.uri).is_ok(), RelationType::from(role.as_str())) {
            | (false, _) => (
                None,
                None,
                Some(ConversionWarning::omitted(
                    "ckan",
                    "dcat",
                    "qualified_relation.uri",
                    "Relationship URI is invalid",
                )),
            ),
            | (true, RelationType::Other) => (
                None,
                Some(self.uri.clone()),
                Some(ConversionWarning::approximated(
                    "ckan",
                    "dcat",
                    "qualified_relation",
                    "Unknown relationship role was retained as a generic relation",
                )),
            ),
            | (true, role) => (
                Some(Relationship {
                    relation: self.uri.clone(),
                    had_role: role,
                }),
                None,
                None,
            ),
        }
    }
}
#[cfg(feature = "std")]
fn date(value: Option<&str>, field: &str) -> (Option<String>, Vec<ConversionWarning>) {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        | None => (None, Vec::new()),
        | Some(value) if valid_date(value) => (Some(value.to_string()), Vec::new()),
        | Some(_) => (
            None,
            vec![ConversionWarning::omitted(
                "ckan",
                "dcat",
                field,
                "Value is not a supported ISO 8601 date",
            )],
        ),
    }
}
#[cfg(feature = "std")]
fn description(value: Option<&str>) -> (Option<String>, Vec<ConversionWarning>) {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or((None, Vec::new()), |value| {
            let has_markup = value.contains('<') && value.contains('>');
            let prepared = value
                .replace("</p>", "\n\n")
                .replace("</div>", "\n")
                .replace("<br>", "\n")
                .replace("<br/>", "\n")
                .replace("<br />", "\n");
            let cleaned = Builder::empty().clean(&prepared).to_string();
            let decoded = unescape(&cleaned).map_or(cleaned.clone(), |text| text.into_owned());
            let normalized = decoded
                .replace("\r\n", "\n")
                .replace('\r', "\n")
                .split("\n\n")
                .map(|paragraph| paragraph.split_whitespace().collect::<Vec<_>>().join(" "))
                .filter(|paragraph| !paragraph.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");
            let warnings = has_markup
                .then(|| ConversionWarning::simplified("ckan", "dcat", "notes", "HTML markup was converted to plain text"))
                .into_iter()
                .collect();
            (Some(normalized), warnings)
        })
}
#[cfg(feature = "std")]
fn distribution(resource: &Resource, index: usize) -> (Option<Distribution>, Vec<ConversionWarning>) {
    let field = |name: &str| format!("resources[{index}].{name}");
    match resource.url.as_deref().map(str::trim).filter(|url| rules::uri(url).is_ok()) {
        | None => (
            None,
            vec![ConversionWarning::omitted(
                "ckan",
                "dcat",
                field("url"),
                "Resource URL is missing or invalid",
            )],
        ),
        | Some(url) => {
            let service = resource.datastore_active == Some(true)
                || resource
                    .resource_type
                    .as_deref()
                    .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "api" | "service" | "datastore"))
                || resource
                    .url_type
                    .as_deref()
                    .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "api" | "service" | "datastore"));
            let mut extension_fields = resource
                .extensions
                .iter()
                .filter(|(_, value)| meaningful(value))
                .map(|(key, _)| field(key))
                .collect::<Vec<_>>();
            extension_fields.sort_by_cached_key(|field| field.to_ascii_lowercase());
            let (issued, issued_warnings) = date(resource.created.as_deref(), &field("created"));
            let (modified, modified_warnings) = date(resource.last_modified.as_deref(), &field("last_modified"));
            let warnings = service
                .then(|| {
                    ConversionWarning::simplified(
                        "ckan",
                        "dcat",
                        field("url"),
                        "Service access is represented by a distribution reference because Dataset output cannot embed a DataService",
                    )
                })
                .into_iter()
                .chain(optional_warning(
                    &resource.id,
                    &field("id"),
                    "CKAN resource UUID has no DCAT Distribution identifier field",
                ))
                .chain(optional_warning(
                    &resource.metadata_modified,
                    &field("metadata_modified"),
                    "Metadata-record timestamp is not a distribution date",
                ))
                .chain(optional_warning(&resource.hash, &field("hash"), "Checksum algorithm is not explicit"))
                .chain(
                    extension_fields
                        .into_iter()
                        .map(|field| ConversionWarning::no_equivalent("ckan", "dcat", field)),
                )
                .chain(issued_warnings)
                .chain(modified_warnings)
                .collect();
            let id = Location::from(url).is_non_loopback_http_or_https().then(|| url.to_string());
            let title = nonempty(resource.name.as_deref()).map(OneOrMany::One);
            let description = nonempty(resource.description.as_deref()).map(OneOrMany::One);
            let access_service = service.then(|| OneOrMany::One(url.to_string()));
            let download_url = resource
                .url_type
                .as_deref()
                .is_some_and(|value| value.eq_ignore_ascii_case("upload"))
                .then(|| OneOrMany::One(url.to_string()));
            (
                Some(
                    Distribution::init()
                        .maybe_id(id)
                        .jsonld_type("dcat:Distribution")
                        .maybe_title(title)
                        .maybe_description(description)
                        .maybe_issued(issued)
                        .maybe_modified(modified)
                        .access_url(OneOrMany::One(url.to_string()))
                        .maybe_access_service(access_service)
                        .maybe_download_url(download_url)
                        .maybe_byte_size(resource.size)
                        .maybe_media_type(nonempty(resource.mimetype.as_deref()))
                        .maybe_format(nonempty(resource.format.as_deref()))
                        .build(),
                ),
                warnings,
            )
        }
    }
}
#[cfg(feature = "std")]
fn distributions(resources: &[Resource]) -> (Option<Vec<Distribution>>, Vec<ConversionWarning>) {
    let (values, warnings): (Vec<_>, Vec<_>) = resources
        .iter()
        .enumerate()
        .map(|(index, resource)| distribution(resource, index))
        .unzip();
    let values = values.into_iter().flatten().collect::<Vec<_>>();
    ((!values.is_empty()).then_some(values), warnings.into_iter().flatten().collect())
}
#[cfg(feature = "std")]
fn identity(url: Option<&str>) -> (Option<String>, Option<OneOrMany<DocumentRef>>, Vec<ConversionWarning>) {
    match url.map(str::trim).filter(|url| rules::uri(url).is_ok()) {
        | None => {
            let warnings = url
                .filter(|url| !url.trim().is_empty())
                .map(|_| ConversionWarning::omitted("ckan", "dcat", "url", "Package URL is not an absolute HTTP(S) URI"))
                .into_iter()
                .collect();
            (None, None, warnings)
        }
        | Some(url) if Location::from(url).is_non_loopback_http_or_https() => (
            Some(url.to_string()),
            Some(OneOrMany::Many(vec![DocumentRef::Uri(url.to_string())])),
            Vec::new(),
        ),
        | Some(url) => (
            None,
            Some(OneOrMany::Many(vec![DocumentRef::Uri(url.to_string())])),
            vec![ConversionWarning::approximated(
                "ckan",
                "dcat",
                "url",
                "Local URL is retained as landingPage but not asserted as @id",
            )],
        ),
    }
}
#[cfg(feature = "std")]
fn meaningful(value: &Value) -> bool {
    match value {
        | Value::Null => false,
        | Value::Bool(value) => *value,
        | Value::Number(_) => true,
        | Value::String(value) => !value.trim().is_empty(),
        | Value::Array(values) => !values.is_empty(),
        | Value::Object(values) => !values.is_empty(),
    }
}
#[cfg(feature = "std")]
fn nonempty(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(ToString::to_string)
}
#[cfg(feature = "std")]
fn option_many<T>(values: Vec<T>) -> Option<OneOrMany<T>> {
    (!values.is_empty()).then_some(OneOrMany::Many(values))
}
#[cfg(feature = "std")]
fn optional_warning<T>(value: &Option<T>, field: &str, details: &str) -> Vec<ConversionWarning> {
    value
        .is_some()
        .then(|| ConversionWarning::omitted("ckan", "dcat", field.to_string(), details.to_string()))
        .into_iter()
        .collect()
}
fn parse_document_value(value: Value) -> Result<OneOrMany<Package>, CrosswalkError> {
    match value {
        | Value::Array(values) => values
            .into_iter()
            .enumerate()
            .map(|(index, value)| parse_package_value(value).map_err(|error| error.indexed(index, None)))
            .collect::<Result<Vec<_>, _>>()
            .map(OneOrMany::Many),
        | value => parse_package_value(value).map(OneOrMany::One),
    }
}
fn parse_json_lines(content: &str) -> Result<OneOrMany<Package>, CrosswalkError> {
    match content.is_empty() {
        | true => Err(CrosswalkError::ParseFailed("JSON Lines input cannot be empty".to_string())),
        | false => content
            .lines()
            .enumerate()
            .map(|(index, line)| {
                let line_number = index.saturating_add(1);
                serde_json::from_str(line)
                    .map_err(|error| CrosswalkError::ParseFailed(error.to_string()))
                    .and_then(parse_package_value)
                    .map_err(|error| error.indexed(index, Some(line_number)))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(OneOrMany::Many),
    }
}
fn parse_package_value(value: Value) -> Result<Package, CrosswalkError> {
    let package = match value.get("success").and_then(Value::as_bool) {
        | Some(false) => Err(CrosswalkError::ParseFailed("CKAN Action API response reported failure".to_string())),
        | Some(true) => value
            .get("result")
            .cloned()
            .ok_or_else(|| CrosswalkError::ParseFailed("Successful CKAN Action API response is missing result".to_string())),
        | None => Ok(value),
    };
    package.and_then(|package| {
        serde_json::to_string(&package)
            .map_err(|error| CrosswalkError::ParseFailed(error.to_string()))
            .and_then(|json| {
                eserde::json::from_str::<Package>(&json).map_err(|errors| {
                    CrosswalkError::ParseFailed(
                        errors
                            .iter()
                            .map(|error| format!("{}: {}", error.path().map_or("root".into(), |path| path.to_string()), error.message()))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
                })
            })
            .and_then(|package| {
                package
                    .validate()
                    .map(|()| package)
                    .map_err(|errors| CrosswalkError::ParseFailed(errors.to_string()))
            })
    })
}
#[cfg(feature = "std")]
fn valid_date(value: &str) -> bool {
    let year = value.len() == 4 && value.bytes().all(|byte| byte.is_ascii_digit());
    let month = value.len() == 7
        && value.as_bytes().get(4) == Some(&b'-')
        && value.get(0..4).is_some_and(|part| part.bytes().all(|byte| byte.is_ascii_digit()))
        && value
            .get(5..7)
            .and_then(|part| part.parse::<u8>().ok())
            .is_some_and(|month| (1..=12).contains(&month));
    year || month || rules::date(value).is_ok() || rules::timestamp(value).is_ok()
}
