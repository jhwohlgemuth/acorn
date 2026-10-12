//! Dublin Core crosswalks through the shared research-output model.
use super::CrosswalkSchema::{DataCite as DATACITE, DublinCore as DUBLIN_CORE, RadOutput as RAD_OUTPUT};
use super::{via_datacite, ConversionWarning, Crosswalk, CrosswalkError};
use crate::pid::{PersistentIdentifier, PersistentIdentifierParse, DOI};
use crate::research_activity::common::{ControlledSubject, StructuredRights, TypedIdentifier, TypedRelatedResource};
use crate::research_activity::output::{Contributor, ResearchOutput};
use crate::research_activity::{ResearchActivity, ResearchActivityMetadata};
use crate::standard::dublin_core::{
    DublinCoreDescription, DublinCoreDocument, DublinCoreLiteral, DublinCoreNamespace, DublinCoreResource, DublinCoreValue, DCTERMS_NAMESPACE,
    ELEMENTS_NAMESPACE, TYPE_NAMESPACE,
};
use crate::standard::relation::Relation;
use crate::standard::{
    datacite::{self, Description, DescriptionType, Publisher, ResourceTypeGeneral},
    invenio,
};
#[cfg(feature = "std")]
use crate::standard::{dcat, huwise};
use crate::validation::Validate;
use crate::{ContactPoint, OneOrMany, Website};
use acorn_core::prelude::{format, vec, String, ToString, Vec};
use alloc::collections::BTreeMap;
use core::{fmt, iter::once};

/// Required local context for converting Dublin Core into research activity data.
#[derive(Clone, Debug, Default)]
pub struct DublinCoreImportOptions {
    /// Required local contact data, which Dublin Core does not model as a RAD contact point.
    pub contact: Option<ContactPoint>,
    /// Required local RAD identifier.
    pub identifier: Option<String>,
}
struct DublinCoreResourceType<'a>(&'a ResourceTypeGeneral);
impl ConversionWarning {
    fn dublin_core_import(document: &DublinCoreDocument, primary: &DublinCoreDescription) -> Vec<Self> {
        let additional = (document.descriptions.len() > 1).then(|| {
            Self::omitted(
                DUBLIN_CORE,
                RAD_OUTPUT,
                "descriptions[1..]",
                "one root resource is selected deterministically",
            )
        });
        let extensions = primary
            .properties
            .keys()
            .filter(|predicate| !predicate.starts_with(DCTERMS_NAMESPACE) && !predicate.starts_with(ELEMENTS_NAMESPACE))
            .map(|predicate| Self::no_equivalent(DUBLIN_CORE, RAD_OUTPUT, predicate.clone()));
        additional.into_iter().chain(extensions).collect()
    }
}
impl Crosswalk<DublinCoreDocument> for datacite::Record {
    fn crosswalk(&self) -> Result<(DublinCoreDocument, Vec<ConversionWarning>), CrosswalkError> {
        <Self as Crosswalk<ResearchOutput>>::crosswalk(self).and_then(|(output, warnings)| {
            <ResearchOutput as Crosswalk<DublinCoreDocument>>::crosswalk(&output).map(|(document, next)| {
                let warnings = warnings.into_iter().map(|warning| warning.rebase(DATACITE, &[])).chain(next).collect();
                (document, warnings)
            })
        })
    }
}
impl DublinCoreDescription {
    fn first_text(&self, name: &str) -> Option<String> {
        self.literals(name).next().map(|value| value.value.clone())
    }
    fn literals<'a>(&'a self, name: &str) -> impl Iterator<Item = &'a DublinCoreLiteral> {
        self.values(name).filter_map(|value| match value {
            | DublinCoreValue::Literal(value) => Some(value),
            | DublinCoreValue::Resource(_) => None,
        })
    }
    fn related(&self) -> Vec<TypedRelatedResource> {
        Relation::ALL
            .into_iter()
            .flat_map(|relation| {
                let name = relation.to_string();
                self.values(&name)
                    .map(|value| {
                        let relation_type = relation.relation_type();
                        TypedRelatedResource {
                            category: None,
                            identifier: TypedIdentifier {
                                scheme: None,
                                scheme_uri: None,
                                value: match value {
                                    | DublinCoreValue::Literal(value) => value.value.clone(),
                                    | DublinCoreValue::Resource(value) => value.identifier(),
                                },
                            },
                            relation_type: relation_type.to_string(),
                            relation_type_information: (relation_type == "Other")
                                .then(|| format!("{}:{name}", DublinCoreNamespace::MetadataTerms.prefix())),
                            resource_type_general: None,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
    fn required_text(&self, name: &str) -> Result<String, CrosswalkError> {
        self.first_text(name)
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| CrosswalkError::MissingRequiredField(format!("{}:{name}", DublinCoreNamespace::MetadataTerms.prefix())))
    }
    fn texts(&self, name: &str) -> Vec<String> {
        self.literals(name).map(|value| value.value.clone()).collect()
    }
    fn values<'a>(&'a self, name: &str) -> impl Iterator<Item = &'a DublinCoreValue> {
        self.properties
            .get(&format!("{DCTERMS_NAMESPACE}{name}"))
            .into_iter()
            .flatten()
            .chain(self.properties.get(&format!("{ELEMENTS_NAMESPACE}{name}")).into_iter().flatten())
    }
}
impl Crosswalk<datacite::Record> for DublinCoreDocument {
    fn crosswalk(&self) -> Result<(datacite::Record, Vec<ConversionWarning>), CrosswalkError> {
        <Self as Crosswalk<ResearchOutput>>::crosswalk(self).and_then(|(output, warnings)| {
            <ResearchOutput as Crosswalk<datacite::Record>>::crosswalk(&output).map(|(record, next)| {
                let warnings = warnings
                    .into_iter()
                    .chain(next.into_iter().map(|warning| warning.rebase(DUBLIN_CORE, &[])))
                    .collect();
                (record, warnings)
            })
        })
    }
}
impl Crosswalk<ResearchOutput> for DublinCoreDocument {
    fn crosswalk(&self) -> Result<(ResearchOutput, Vec<ConversionWarning>), CrosswalkError> {
        self.primary().and_then(|description| {
            description.required_text("title").map(|title| {
                let identifier = description.subject.as_iri().map(ToString::to_string);
                let identifiers = description.texts("identifier");
                let doi = identifier
                    .iter()
                    .chain(identifiers.iter())
                    .find_map(|value| DOI::is_valid(value).then(|| DOI::format(value)));
                let publication_year = description
                    .texts("issued")
                    .into_iter()
                    .chain(description.texts("date"))
                    .find_map(|value| value.get(..4).and_then(|year| year.parse::<i32>().ok()));
                let publisher = description.first_text("publisher").map(|name| Publisher {
                    name,
                    publisher_identifier: None,
                    publisher_identifier_scheme: None,
                    scheme_uri: None,
                });
                let types = description
                    .types
                    .iter()
                    .filter_map(|value| value.strip_prefix(TYPE_NAMESPACE))
                    .collect::<Vec<_>>();
                let resource_type_general = types.first().map(|value| ResourceTypeGeneral::from(*value));
                let creators = self.contributors(description, "creator");
                let contributors = self.contributors(description, "contributor");
                let alternate_identifiers = identifiers
                    .into_iter()
                    .filter(|value| !DOI::is_valid(value))
                    .map(|value| TypedIdentifier {
                        scheme: None,
                        scheme_uri: None,
                        value,
                    })
                    .collect::<Vec<_>>();
                let related_identifiers = description.related();
                let descriptions: Vec<Description> = description.clone().into();
                let subjects = description
                    .values("subject")
                    .map(|value| match value {
                        | DublinCoreValue::Literal(value) => ControlledSubject {
                            scheme_uri: None,
                            subject: value.value.clone(),
                            subject_scheme: None,
                            value_uri: None,
                        },
                        | DublinCoreValue::Resource(value) => ControlledSubject {
                            scheme_uri: None,
                            subject: value.identifier(),
                            subject_scheme: None,
                            value_uri: value.as_iri().map(ToString::to_string),
                        },
                    })
                    .collect::<Vec<_>>();
                let rights_list: Vec<StructuredRights> = description.clone().into();
                let websites = description.subject.as_iri().map(|url| {
                    vec![Website {
                        description: "Dublin Core described resource".to_string(),
                        url: url.to_string(),
                    }]
                });
                let output = ResearchOutput::init()
                    .title(title)
                    .maybe_identifier(identifier)
                    .maybe_doi(doi)
                    .maybe_kind(types.first().map(|value| (*value).to_string()))
                    .maybe_publication_year(publication_year)
                    .maybe_publisher(publisher)
                    .maybe_resource_type_general(resource_type_general)
                    .maybe_resource_type(types.first().map(|value| (*value).to_string()))
                    .maybe_language(description.first_text("language").filter(|value| value.len() == 2))
                    .maybe_alternate_identifiers(nonempty(alternate_identifiers))
                    .maybe_related_identifiers(nonempty(related_identifiers))
                    .maybe_rights_list(nonempty(rights_list))
                    .maybe_descriptions(nonempty(descriptions))
                    .maybe_subjects(nonempty(subjects))
                    .maybe_formats(nonempty(description.texts("format")))
                    .maybe_sizes(nonempty(description.texts("extent")))
                    .maybe_creators(nonempty(creators))
                    .maybe_contributors(nonempty(contributors))
                    .maybe_websites(websites)
                    .build();
                (output, ConversionWarning::dublin_core_import(self, description))
            })
        })
    }
}
impl DublinCoreDocument {
    fn contributors(&self, description: &DublinCoreDescription, name: &str) -> Vec<Contributor> {
        description
            .values(name)
            .map(|value| {
                let name = match value {
                    | DublinCoreValue::Literal(value) => value.value.clone(),
                    | DublinCoreValue::Resource(resource) => self
                        .descriptions
                        .iter()
                        .find(|candidate| &candidate.subject == resource)
                        .and_then(|candidate| candidate.first_text("title"))
                        .unwrap_or_else(|| resource.identifier()),
                };
                Contributor::init().name(name).build()
            })
            .collect()
    }
    fn primary(&self) -> Result<&DublinCoreDescription, CrosswalkError> {
        self.descriptions
            .iter()
            .find(|description| description.values("title").next().is_some())
            .or_else(|| self.descriptions.first())
            .ok_or_else(|| CrosswalkError::MissingRequiredField("descriptions".to_string()))
    }
    /// Convert Dublin Core into research activity data using explicit local identity context.
    pub fn to_research_activity(&self, options: &DublinCoreImportOptions) -> Result<(ResearchActivity, Vec<ConversionWarning>), CrosswalkError> {
        options
            .identifier
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string)
            .ok_or_else(|| CrosswalkError::MissingRequiredField("options.identifier".to_string()))
            .and_then(|identifier| {
                options
                    .contact
                    .clone()
                    .ok_or_else(|| CrosswalkError::MissingRequiredField("options.contact".to_string()))
                    .map(|contact| (identifier, contact))
            })
            .and_then(|(identifier, contact)| {
                self.primary().and_then(|primary| {
                    primary.required_text("title").and_then(|title| {
                        <Self as Crosswalk<ResearchOutput>>::crosswalk(self).and_then(|(output, warnings)| {
                            let activity = ResearchActivity {
                                contact,
                                meta: ResearchActivityMetadata {
                                    identifier,
                                    outputs: Some(OneOrMany::One(output)),
                                    ..ResearchActivityMetadata::default()
                                },
                                subtitle: primary.first_text("alternative"),
                                title,
                                ..ResearchActivity::default()
                            };
                            activity
                                .validate()
                                .map(|_| (activity, warnings))
                                .map_err(|error| CrosswalkError::BuildFailed(error.to_string()))
                        })
                    })
                })
            })
    }
}
#[cfg(feature = "std")]
via_datacite!(DublinCoreDocument => dcat::Dataset);
#[cfg(feature = "std")]
via_datacite!(DublinCoreDocument => huwise::Dataset);
via_datacite!(DublinCoreDocument => invenio::Record);
impl DublinCoreResource {
    fn identifier(&self) -> String {
        match self {
            | Self::BlankNode(value) => format!("_:{value}"),
            | Self::Iri(value) => value.clone(),
        }
    }
}
impl From<ResearchOutput> for DublinCoreResource {
    fn from(output: ResearchOutput) -> Self {
        output
            .identifier
            .as_deref()
            .filter(|value| value.contains(':'))
            .map(Self::iri)
            .or_else(|| output.doi.map(|value| Self::iri(DOI::from_string(value).url())))
            .unwrap_or_else(|| Self::blank_node("resource"))
    }
}
impl fmt::Display for DublinCoreResourceType<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self.0 {
            | ResourceTypeGeneral::Audiovisual => "MovingImage",
            | ResourceTypeGeneral::Collection => "Collection",
            | ResourceTypeGeneral::Dataset => "Dataset",
            | ResourceTypeGeneral::Event => "Event",
            | ResourceTypeGeneral::Image => "Image",
            | ResourceTypeGeneral::InteractiveResource | ResourceTypeGeneral::ComputationalNotebook => "InteractiveResource",
            | ResourceTypeGeneral::PhysicalObject => "PhysicalObject",
            | ResourceTypeGeneral::Service => "Service",
            | ResourceTypeGeneral::Software | ResourceTypeGeneral::Workflow => "Software",
            | ResourceTypeGeneral::Sound => "Sound",
            | _ => "Text",
        };
        formatter.write_str(value)
    }
}
impl Crosswalk<DublinCoreDocument> for ResearchActivity {
    fn crosswalk(&self) -> Result<(DublinCoreDocument, Vec<ConversionWarning>), CrosswalkError> {
        self.to_datacite_project().and_then(|(record, warnings)| {
            <datacite::Record as Crosswalk<DublinCoreDocument>>::crosswalk(&record)
                .map(|(document, next)| (document, warnings.into_iter().chain(next).collect()))
        })
    }
}
impl Crosswalk<DublinCoreDocument> for ResearchOutput {
    fn crosswalk(&self) -> Result<(DublinCoreDocument, Vec<ConversionWarning>), CrosswalkError> {
        (!self.title.trim().is_empty())
            .then_some(())
            .ok_or_else(|| CrosswalkError::MissingRequiredField("title".to_string()))
            .map(|()| {
                let properties = once(property("title", DublinCoreValue::literal(&self.title, None, None)))
                    .chain(
                        self.doi
                            .iter()
                            .map(|value| property("identifier", DublinCoreValue::literal(value, None, None))),
                    )
                    .chain(
                        self.alternate_identifiers
                            .iter()
                            .flatten()
                            .map(|value| property("identifier", DublinCoreValue::literal(&value.value, None, None))),
                    )
                    .chain(
                        self.creators
                            .iter()
                            .flatten()
                            .map(|value| property("creator", DublinCoreValue::literal(&value.name, None, None))),
                    )
                    .chain(
                        self.contributors
                            .iter()
                            .flatten()
                            .map(|value| property("contributor", DublinCoreValue::literal(&value.name, None, None))),
                    )
                    .chain(
                        self.publisher
                            .iter()
                            .map(|value| property("publisher", DublinCoreValue::literal(&value.name, None, None))),
                    )
                    .chain(
                        self.publication_year
                            .iter()
                            .map(|value| property("issued", DublinCoreValue::literal(&value.to_string(), None, Some("W3CDTF")))),
                    )
                    .chain(
                        self.dates
                            .iter()
                            .flatten()
                            .map(|value| property(&value.date_type.to_string(), DublinCoreValue::literal(&value.date, None, Some("W3CDTF")))),
                    )
                    .chain(
                        self.language
                            .iter()
                            .map(|value| property("language", DublinCoreValue::literal(value, None, Some("RFC5646")))),
                    )
                    .chain(
                        self.formats
                            .iter()
                            .flatten()
                            .map(|value| property("format", DublinCoreValue::literal(value, None, None))),
                    )
                    .chain(
                        self.sizes
                            .iter()
                            .flatten()
                            .map(|value| property("extent", DublinCoreValue::literal(value, None, None))),
                    )
                    .chain(self.subjects.iter().flatten().map(|value| match &value.value_uri {
                        | Some(value) => property("subject", DublinCoreValue::Resource(DublinCoreResource::iri(value.clone()))),
                        | None => property("subject", DublinCoreValue::literal(&value.subject, None, None)),
                    }))
                    .chain(self.descriptions.iter().flatten().map(|value| {
                        let name = match value.description_type {
                            | Some(DescriptionType::Abstract) => "abstract",
                            | Some(DescriptionType::TableOfContents) => "tableOfContents",
                            | _ => "description",
                        };
                        property(name, DublinCoreValue::literal(&value.description, value.language.as_deref(), None))
                    }))
                    .chain(
                        self.rights_list
                            .iter()
                            .flatten()
                            .filter_map(|value| match (&value.rights_uri, &value.rights) {
                                | (Some(value), _) => Some(property("rights", DublinCoreValue::Resource(DublinCoreResource::iri(value.clone())))),
                                | (_, Some(value)) => Some(property("rights", DublinCoreValue::literal(value, None, None))),
                                | _ => None,
                            }),
                    )
                    .chain(self.related_identifiers.iter().flatten().map(|value| {
                        let predicate = Relation::from_relation_type(&value.relation_type).to_string();
                        let value = match value.identifier.value.contains(':') {
                            | true => DublinCoreValue::Resource(DublinCoreResource::iri(value.identifier.value.clone())),
                            | false => DublinCoreValue::literal(&value.identifier.value, None, None),
                        };
                        property(&predicate, value)
                    }))
                    .fold(BTreeMap::<String, Vec<DublinCoreValue>>::new(), |mut properties, (predicate, value)| {
                        properties.entry(predicate).or_default().push(value);
                        properties
                    });
                let types = self
                    .resource_type_general
                    .as_ref()
                    .map(DublinCoreResourceType)
                    .into_iter()
                    .map(|value| format!("{TYPE_NAMESPACE}{value}"))
                    .collect();
                let document = DublinCoreDocument::new(vec![DublinCoreDescription {
                    properties,
                    subject: self.clone().into(),
                    types,
                }]);
                (document, Vec::from(self.clone()))
            })
    }
}
impl From<ResearchOutput> for Vec<ConversionWarning> {
    fn from(output: ResearchOutput) -> Self {
        [
            output.access.map(|_| ConversionWarning::no_equivalent(RAD_OUTPUT, DUBLIN_CORE, "access")),
            output
                .funding
                .map(|_| ConversionWarning::no_equivalent(RAD_OUTPUT, DUBLIN_CORE, "funding")),
            output.geo_locations.map(|_| {
                ConversionWarning::simplified(
                    RAD_OUTPUT,
                    DUBLIN_CORE,
                    "geoLocations",
                    format!("use {}:spatial explicitly", DublinCoreNamespace::MetadataTerms.prefix()),
                )
            }),
            (!output.keywords.is_empty()).then(|| ConversionWarning::no_equivalent(RAD_OUTPUT, DUBLIN_CORE, "keywords")),
            output
                .related_items
                .map(|_| ConversionWarning::simplified(RAD_OUTPUT, DUBLIN_CORE, "relatedItems", "use a DCMI relation property")),
            output
                .version
                .map(|_| ConversionWarning::no_equivalent(RAD_OUTPUT, DUBLIN_CORE, "version")),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
impl From<DublinCoreDescription> for Vec<Description> {
    fn from(description: DublinCoreDescription) -> Self {
        [
            ("abstract", DescriptionType::Abstract),
            ("description", DescriptionType::Other),
            ("tableOfContents", DescriptionType::TableOfContents),
        ]
        .into_iter()
        .flat_map(|(name, kind)| {
            description.literals(name).map(move |value| Description {
                description: value.value.clone(),
                description_type: Some(kind.clone()),
                language: value.language.clone().filter(|value| value.len() == 2),
            })
        })
        .collect()
    }
}
impl From<DublinCoreDescription> for Vec<StructuredRights> {
    fn from(description: DublinCoreDescription) -> Self {
        description
            .values("rights")
            .chain(description.values("license"))
            .map(|value| match value {
                | DublinCoreValue::Literal(value) => StructuredRights::init().rights(value.value.clone()).build(),
                | DublinCoreValue::Resource(value) => StructuredRights::init().maybe_rights_uri(value.as_iri().map(ToString::to_string)).build(),
            })
            .collect()
    }
}
fn nonempty<T>(values: Vec<T>) -> Option<Vec<T>> {
    (!values.is_empty()).then_some(values)
}
fn property(name: &str, value: DublinCoreValue) -> (String, DublinCoreValue) {
    (format!("{DCTERMS_NAMESPACE}{name}"), value)
}
