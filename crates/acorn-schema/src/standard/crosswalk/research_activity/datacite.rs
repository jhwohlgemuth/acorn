//! DataCite Project crosswalk for ACORN research activities.
//!
//! The projection is intentionally one-way and opt-in. It requires authored
//! deposit metadata and reports every local-only or simplified RAD value.
use crate::pid::raid::CreditRole;
use crate::pid::{PersistentIdentifierParse, DOI, PID};
use crate::research_activity::output::{Affiliation, Contributor, ResearchOutput};
use crate::research_activity::{ActivityDateType, ActivityOrganization, ResearchActivity, StructuredRights, TypedIdentifier, TypedRelatedResource};
use crate::standard::crosswalk::CrosswalkSchema::{DataCite as DATACITE, Rad as RAD};
use crate::standard::crosswalk::{Conversion, ConversionWarning, Crosswalk, CrosswalkError};
use crate::standard::datacite::{
    self, ContributorType, Description, DescriptionType, GeoLocation, NameType, Publisher, RelatedItem, RelatedItemContributor, RelatedItemCreator,
    RelatedItemIdentifier, RelatedItemTitle, RelationType, ResourceTypeGeneral, Title, TitleType,
};
use crate::util::constants::app::HANDLE_RESOLVER_URI;
use acorn_core::prelude::alloc::{format, vec, String, ToString, Vec};
use core::iter::once;

const ACORN_IDENTIFIER_TYPE: &str = "ACORN";
const OUTPUT_FIELD_REBASE: [(&str, &str); 9] = [
    ("alternateIdentifiers", "meta.handle"),
    ("contributors", "meta.organizations"),
    ("creators", "meta.contributors"),
    ("dates", "meta.typedDates"),
    ("funding", "meta.funders"),
    ("relatedIdentifiers", "meta.relatedResources"),
    ("rightsList", "meta.rights"),
    ("subjects", "meta.subjects"),
    ("websites", "meta.websites"),
];
const PUBLISHER_ROLE: &str = "Publisher";
type ProjectParts = (String, i32, Publisher);

impl ActivityOrganization {
    fn has_role(&self, expected: &str) -> bool {
        self.roles
            .iter()
            .any(|role| role.role.as_deref().is_some_and(|role| role.eq_ignore_ascii_case(expected)))
    }
}
impl Crosswalk<datacite::Record> for ResearchActivity {
    fn crosswalk(&self) -> Result<(datacite::Record, Vec<ConversionWarning>), CrosswalkError> {
        self.to_datacite_project()
    }
}
impl ResearchActivity {
    /// Export this activity as a deposit-ready DataCite Project record.
    ///
    /// The source must contain exactly one DOI, one `Issued` date, and one
    /// organization with a `Publisher` role. No deposit metadata is inferred.
    pub fn to_datacite_project(&self) -> Result<(datacite::Record, Vec<ConversionWarning>), CrosswalkError> {
        self.project_parts().and_then(|(doi, publication_year, publisher)| {
            let creators = self.project_creators();
            let contributors = self.project_organization_contributors();
            let related = self.project_related();
            let related_items = self.project_related_items();
            let rights = self.project_rights();
            let output = ResearchOutput {
                identifier: Some(self.meta.identifier.clone()),
                doi: Some(doi),
                title: self.title.clone(),
                kind: None,
                publication_year: Some(publication_year),
                dates: self.meta.typed_dates.clone().map(|values| values.into_vec()),
                publisher: Some(publisher),
                resource_type_general: Some(ResourceTypeGeneral::Project),
                resource_type: Some("Research Project".to_string()),
                language: None,
                alternate_identifiers: Some(self.project_alternate_identifiers()),
                related_identifiers: (!related.value.is_empty()).then_some(related.value),
                related_items: related_items.value,
                version: None,
                rights_list: (!rights.is_empty()).then_some(rights),
                descriptions: self.project_descriptions(),
                subjects: self.meta.subjects.clone().map(|values| values.into_vec()),
                formats: None,
                sizes: None,
                geo_locations: self.project_geo_locations(),
                creators: Some(creators.value),
                contributors: (!contributors.value.is_empty()).then_some(contributors.value),
                funding: self.meta.funders.clone().map(|values| values.into_vec()),
                websites: self.meta.websites.clone().map(|values| values.into_vec()),
                keywords: self.meta.keywords.clone(),
                access: None,
            };
            <ResearchOutput as Crosswalk<datacite::Record>>::crosswalk(&output).map(|(mut record, warnings)| {
                if let Some(subtitle) = &self.subtitle {
                    record.attributes.titles.get_or_insert_default().push(Title {
                        title: subtitle.clone(),
                        title_type: Some(TitleType::Subtitle),
                    });
                }
                let warnings = creators
                    .warnings
                    .into_iter()
                    .chain(contributors.warnings)
                    .chain(related.warnings)
                    .chain(related_items.warnings)
                    .chain(self.project_local_warnings())
                    .chain(warnings.into_iter().map(|warning| warning.rebase(RAD, &OUTPUT_FIELD_REBASE)))
                    .collect();
                (record, warnings)
            })
        })
    }
    fn project_alternate_identifiers(&self) -> Vec<TypedIdentifier> {
        once(TypedIdentifier {
            value: self.meta.identifier.clone(),
            scheme: Some(ACORN_IDENTIFIER_TYPE.to_string()),
            scheme_uri: None,
        })
        .chain(self.meta.handle.as_ref().into_iter().flat_map(|values| {
            values.iter().map(|value| TypedIdentifier {
                value: value.clone(),
                scheme: Some("Handle".to_string()),
                scheme_uri: Some(HANDLE_RESOLVER_URI.to_string()),
            })
        }))
        .collect()
    }
    fn project_creators(&self) -> Conversion<Vec<Contributor>> {
        let contact_name = format!("{}, {}", self.contact.family_name, self.contact.given_name);
        let contact_orcid = self.contact.identifier.clone();
        let contact_affiliations = [Some(self.contact.organization.as_str()), self.contact.affiliation.as_deref()]
            .into_iter()
            .flatten()
            .filter(|value| !value.trim().is_empty())
            .map(|name| Affiliation {
                name: name.to_string(),
                ror: None,
            })
            .collect();
        let converted = self
            .meta
            .contributors
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter())
            .enumerate()
            .map(|(index, contributor)| {
                let orcid_index = contributor
                    .identifiers
                    .iter()
                    .position(|identifier| identifier.scheme.as_deref().is_some_and(|scheme| PID::from(scheme) == PID::ORCID));
                let orcid = orcid_index
                    .and_then(|orcid_index| contributor.identifiers.get(orcid_index))
                    .map(|identifier| identifier.value.clone());
                let roles = contributor
                    .roles
                    .iter()
                    .filter_map(|role| role.role.as_deref().and_then(|role| CreditRole::try_from(role).ok()))
                    .collect::<Vec<_>>();
                let warnings = contributor
                    .identifiers
                    .iter()
                    .enumerate()
                    .filter(|(identifier_index, _)| Some(*identifier_index) != orcid_index)
                    .map(|(identifier_index, _)| {
                        ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.contributors[{index}].identifiers[{identifier_index}]"))
                    })
                    .chain(
                        contributor
                            .affiliations
                            .iter()
                            .map(|_| ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.contributors[{index}].affiliations[]"))),
                    )
                    .chain(contributor.roles.iter().flat_map(|role| {
                        [
                            Some(ConversionWarning::no_equivalent(
                                RAD,
                                DATACITE,
                                format!("meta.contributors[{index}].roles[].agent"),
                            )),
                            role.role.as_ref().and_then(|role| {
                                CreditRole::try_from(role.as_str())
                                    .is_err()
                                    .then(|| ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.contributors[{index}].roles[].role")))
                            }),
                            role.start_date
                                .as_ref()
                                .map(|_| ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.contributors[{index}].roles[].startDate"))),
                            role.end_date
                                .as_ref()
                                .map(|_| ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.contributors[{index}].roles[].endDate"))),
                            role.role_scheme_uri.as_ref().map(|_| {
                                ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.contributors[{index}].roles[].roleSchemeUri"))
                            }),
                            role.contact
                                .map(|_| ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.contributors[{index}].roles[].contact"))),
                            role.leader
                                .map(|_| ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.contributors[{index}].roles[].leader"))),
                        ]
                        .into_iter()
                        .flatten()
                    }))
                    .collect::<Vec<_>>();
                let duplicate = orcid.as_deref().is_some_and(|orcid| Some(orcid) == contact_orcid.as_deref())
                    || contributor.name.eq_ignore_ascii_case(&contact_name);
                (
                    index,
                    Contributor {
                        name: contributor.name.clone(),
                        name_type: Some(NameType::Personal),
                        orcid,
                        ror: None,
                        contributor_type: None,
                        roles,
                        corresponding: None,
                        position: None,
                        affiliations: Vec::new(),
                    },
                    warnings,
                    duplicate,
                )
            })
            .collect::<Vec<_>>();
        let contact = Contributor {
            name: contact_name,
            name_type: Some(NameType::Personal),
            orcid: contact_orcid,
            ror: None,
            contributor_type: None,
            roles: converted
                .iter()
                .filter(|(_, _, _, duplicate)| *duplicate)
                .flat_map(|(_, contributor, _, _)| contributor.roles.iter().cloned())
                .collect(),
            corresponding: None,
            position: None,
            affiliations: contact_affiliations,
        };
        let contributors = converted
            .iter()
            .filter(|(_, _, _, duplicate)| !duplicate)
            .map(|(_, contributor, _, _)| contributor.clone())
            .collect::<Vec<_>>();
        let warnings = converted
            .into_iter()
            .flat_map(|(index, _, warnings, duplicate)| {
                warnings.into_iter().chain(duplicate.then(|| {
                    ConversionWarning::simplified(
                        RAD,
                        DATACITE,
                        format!("meta.contributors[{index}]"),
                        "the contributor duplicates the primary contact and was merged into that DataCite creator",
                    )
                }))
            })
            .collect();
        Conversion::new(once(contact).chain(contributors).collect(), warnings)
    }
    fn project_descriptions(&self) -> Option<Vec<Description>> {
        self.sections.as_ref().map(|sections| {
            let description = |description: String, description_type: Option<DescriptionType>| {
                Description::init()
                    .description(description)
                    .maybe_description_type(description_type)
                    .build()
            };
            [
                Some(description(sections.mission.clone(), Some(DescriptionType::Abstract))),
                Some(description(sections.challenge.clone(), Some(DescriptionType::Other))),
                (!sections.approach.is_empty()).then(|| description(sections.approach.join("\n"), Some(DescriptionType::Methods))),
                (!sections.impact.is_empty()).then(|| description(sections.impact.join("\n"), Some(DescriptionType::Other))),
                sections
                    .achievement
                    .as_ref()
                    .map(|values| description(values.join("\n"), Some(DescriptionType::Other))),
                sections
                    .capabilities
                    .as_ref()
                    .map(|values| description(values.join("\n"), Some(DescriptionType::TechnicalInfo))),
                Some(description(
                    format!("{}\n{}", sections.research.focus, sections.research.areas.join("\n")),
                    Some(DescriptionType::Other),
                )),
            ]
            .into_iter()
            .flatten()
            .collect()
        })
    }
    fn project_geo_locations(&self) -> Option<Vec<GeoLocation>> {
        self.meta.spatial_coverage.as_ref().map(|values| {
            values
                .iter()
                .map(|coverage| GeoLocation {
                    geo_location_point: None,
                    geo_location_box: None,
                    geo_location_place: coverage
                        .place
                        .iter()
                        .find_map(|place| place.text.clone())
                        .or_else(|| Some(coverage.id.clone())),
                    geo_location_polygon: None,
                })
                .collect()
        })
    }
    fn project_local_warnings(&self) -> Vec<ConversionWarning> {
        let top_level = [
            self.context.as_ref().map(|_| "@context"),
            self.research_activity_type.as_ref().map(|_| "@type"),
            self.aspect.as_ref().map(|_| "aspect"),
            self.notes.as_ref().map(|_| "notes"),
            self.meta.context.as_ref().map(|_| "meta.@context"),
            self.meta.metadata_type.as_ref().map(|_| "meta.@type"),
            self.meta.enrichment.as_ref().map(|_| "meta.enrichment"),
            self.meta.classification.as_ref().map(|_| "meta.classification"),
            Some("meta.archive"),
            Some("meta.draft"),
            Some("meta.status"),
            self.meta.additional_type.as_ref().map(|_| "meta.additionalType"),
            self.meta.media.as_ref().map(|_| "meta.media"),
            (!self.meta.technology.is_empty()).then_some("meta.technology"),
            self.meta.partners.as_ref().map(|_| "meta.partners"),
            self.meta.resources.as_ref().map(|_| "meta.resources"),
            self.meta.infrastructure.as_ref().map(|_| "meta.infrastructure"),
            self.meta.languages.as_ref().map(|_| "meta.languages"),
            self.meta.ror.as_ref().map(|_| "meta.ror"),
        ];
        let contact = [
            self.contact.context.as_ref().map(|_| "contact.@context"),
            self.contact.contact_point_type.as_ref().map(|_| "contact.@type"),
            Some("contact.jobTitle"),
            Some("contact.email"),
            Some("contact.telephone"),
            Some("contact.url"),
            self.meta
                .access
                .as_ref()
                .and_then(|rights| rights.access.as_ref())
                .map(|_| "meta.access.access"),
        ];
        let structural = self.sections.as_ref().map(|_| {
            ConversionWarning::simplified(
                RAD,
                DATACITE,
                "sections",
                "RAD prose sections are flattened into typed DataCite descriptions",
            )
        });
        let logbook = self.logbook.as_ref().into_iter().flat_map(|logbook| {
            logbook
                .entries
                .iter()
                .enumerate()
                .map(|(index, _)| ConversionWarning::no_equivalent(RAD, DATACITE, format!("logbook.entries[{index}]")))
        });
        let spatial = self.meta.spatial_coverage.as_ref().into_iter().flat_map(|values| {
            values
                .iter()
                .enumerate()
                .filter(|(_, coverage)| {
                    coverage.schema_uri.is_some() || coverage.place.len() > 1 || coverage.place.iter().any(|place| place.language.is_some())
                })
                .map(|(index, _)| {
                    ConversionWarning::simplified(
                        RAD,
                        DATACITE,
                        format!("meta.spatialCoverage[{index}]"),
                        "the first authored place is retained without the RAiD place schema or language",
                    )
                })
        });
        let patents = self.meta.patents.as_ref().into_iter().flat_map(|values| {
            values.iter().enumerate().map(|(index, _)| {
                ConversionWarning::approximated(
                    RAD,
                    DATACITE,
                    format!("meta.patents[{index}]"),
                    "the patent identifier is represented as a DataCite URN because DataCite has no Patent identifier type",
                )
            })
        });
        top_level
            .into_iter()
            .chain(contact)
            .flatten()
            .map(|field| ConversionWarning::no_equivalent(RAD, DATACITE, field))
            .chain(structural)
            .chain(logbook)
            .chain(patents)
            .chain(spatial)
            .collect()
    }
    fn project_organization_contributors(&self) -> Conversion<Vec<Contributor>> {
        let converted = self
            .meta
            .organizations
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter())
            .enumerate()
            .map(|(index, organization)| {
                let selected = organization
                    .roles
                    .iter()
                    .enumerate()
                    .filter(|(_, role)| role.role.as_deref().is_none_or(|role| !role.eq_ignore_ascii_case(PUBLISHER_ROLE)))
                    .find_map(|(role_index, role)| {
                        role.role
                            .as_deref()
                            .and_then(|role| ContributorType::try_from(role).ok())
                            .map(|kind| (role_index, kind))
                    });
                let ror_index = organization
                    .identifiers
                    .iter()
                    .position(|identifier| identifier.scheme.as_deref().is_some_and(|scheme| PID::from(scheme) == PID::ROR));
                let ror = ror_index
                    .and_then(|ror_index| organization.identifiers.get(ror_index))
                    .map(|identifier| identifier.value.clone());
                let value = selected.as_ref().map(|(_, contributor_type)| Contributor {
                    name: organization.name.clone(),
                    name_type: Some(NameType::Organizational),
                    orcid: None,
                    ror,
                    contributor_type: Some(contributor_type.clone()),
                    roles: Vec::new(),
                    corresponding: None,
                    position: None,
                    affiliations: Vec::new(),
                });
                let identifier_warnings = organization
                    .identifiers
                    .iter()
                    .enumerate()
                    .filter(|(identifier_index, _)| Some(*identifier_index) != ror_index)
                    .map(|(identifier_index, _)| {
                        ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.organizations[{index}].identifiers[{identifier_index}]"))
                    });
                let role_warnings = organization.roles.iter().enumerate().flat_map(|(role_index, role)| {
                    let is_selected = selected.as_ref().is_some_and(|(selected_index, _)| *selected_index == role_index);
                    let is_publisher = role.role.as_deref().is_some_and(|role| role.eq_ignore_ascii_case(PUBLISHER_ROLE));
                    [
                        Some(ConversionWarning::no_equivalent(
                            RAD,
                            DATACITE,
                            format!("meta.organizations[{index}].roles[{role_index}].organization"),
                        )),
                        (!is_selected && !is_publisher).then(|| {
                            ConversionWarning::simplified(
                                RAD,
                                DATACITE,
                                format!("meta.organizations[{index}].roles[{role_index}]"),
                                "only one DataCite contributor type is retained",
                            )
                        }),
                        role.role_scheme_uri.as_ref().map(|_| {
                            ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.organizations[{index}].roles[{role_index}].roleSchemeUri"))
                        }),
                        role.start_date.as_ref().map(|_| {
                            ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.organizations[{index}].roles[{role_index}].startDate"))
                        }),
                        role.end_date.as_ref().map(|_| {
                            ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.organizations[{index}].roles[{role_index}].endDate"))
                        }),
                    ]
                    .into_iter()
                    .flatten()
                });
                let omitted = (selected.is_none() && !organization.has_role(PUBLISHER_ROLE))
                    .then(|| ConversionWarning::no_equivalent(RAD, DATACITE, format!("meta.organizations[{index}]")));
                (value, identifier_warnings.chain(role_warnings).chain(omitted).collect::<Vec<_>>())
            })
            .collect::<Vec<_>>();
        let values = converted.iter().filter_map(|(value, _)| value.clone()).collect();
        Conversion::new(values, converted.into_iter().flat_map(|(_, warnings)| warnings).collect())
    }
    fn project_parts(&self) -> Result<ProjectParts, CrosswalkError> {
        self.project_doi().and_then(|doi| {
            self.project_publication_year()
                .and_then(|publication_year| self.project_publisher().map(|publisher| (doi, publication_year, publisher)))
        })
    }
    fn project_doi(&self) -> Result<String, CrosswalkError> {
        let dois = self
            .meta
            .doi
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter())
            .filter(|value| !DOI::format(value).is_empty())
            .map(DOI::format)
            .collect::<Vec<_>>();
        match dois.as_slice() {
            | [doi] => Ok(doi.clone()),
            | [] => Err(CrosswalkError::MissingRequiredField("meta.doi[project]".to_string())),
            | _ => Err(CrosswalkError::TransformationFailed {
                field: "meta.doi".to_string(),
                reason: "DataCite Project export requires exactly one authored project DOI".to_string(),
            }),
        }
    }
    fn project_publisher(&self) -> Result<Publisher, CrosswalkError> {
        let publishers = self
            .meta
            .organizations
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter())
            .filter(|organization| organization.has_role(PUBLISHER_ROLE))
            .collect::<Vec<_>>();
        match publishers.as_slice() {
            | [publisher] => {
                let identifier = publisher
                    .identifiers
                    .iter()
                    .find(|identifier| identifier.scheme.as_deref().is_some_and(|scheme| PID::from(scheme) == PID::ROR))
                    .map(|identifier| identifier.value.clone());
                Ok(Publisher {
                    name: publisher.name.clone(),
                    publisher_identifier: identifier.clone(),
                    publisher_identifier_scheme: identifier.as_ref().map(|_| "ROR".to_string()),
                    scheme_uri: identifier.map(|_| "https://ror.org".to_string()),
                })
            }
            | [] => Err(CrosswalkError::MissingRequiredField("meta.organizations[role=Publisher]".to_string())),
            | _ => Err(CrosswalkError::TransformationFailed {
                field: "meta.organizations".to_string(),
                reason: "DataCite Project export requires exactly one organization with a Publisher role".to_string(),
            }),
        }
    }
    fn project_publication_year(&self) -> Result<i32, CrosswalkError> {
        let dates = self
            .meta
            .typed_dates
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter())
            .filter(|date| date.date_type.is_issued())
            .collect::<Vec<_>>();
        match dates.as_slice() {
            | [date] => date
                .date
                .split('-')
                .next()
                .and_then(|year| year.parse::<i32>().ok())
                .ok_or_else(|| CrosswalkError::TransformationFailed {
                    field: "meta.typedDates[Issued]".to_string(),
                    reason: format!("cannot extract a publication year from '{}'", date.date),
                }),
            | [] => Err(CrosswalkError::MissingRequiredField("meta.typedDates[Issued]".to_string())),
            | _ => Err(CrosswalkError::TransformationFailed {
                field: "meta.typedDates".to_string(),
                reason: "DataCite Project export requires exactly one Issued date".to_string(),
            }),
        }
    }
    fn project_related(&self) -> Conversion<Vec<TypedRelatedResource>> {
        let authored = self.meta.related_resources.as_ref().into_iter().flat_map(|values| values.iter().cloned());
        let arxiv = self.meta.doi.as_ref().into_iter().flat_map(|values| {
            values
                .iter()
                .filter(|value| DOI::format(value).is_empty())
                .map(|value| TypedRelatedResource::related(value, "arXiv", "References", Some("Text")))
        });
        let raid = self.meta.raid.as_ref().into_iter().flat_map(|values| {
            values
                .iter()
                .map(|value| TypedRelatedResource::related(value, "RAiD", "IsIdenticalTo", Some("Project")))
        });
        let swhid = self.meta.swhid.as_ref().into_iter().flat_map(|values| {
            values
                .iter()
                .map(|value| TypedRelatedResource::related(value, "SWHID", "HasPart", Some("Software")))
        });
        let books = self.meta.books.as_ref().into_iter().flat_map(|values| {
            values
                .iter()
                .map(|value| TypedRelatedResource::related(value, "ISBN", "References", Some("Book")))
        });
        let patents = self.meta.patents.as_ref().into_iter().flat_map(|values| {
            values
                .iter()
                .map(|value| TypedRelatedResource::related(value, "URN", "References", Some("Other")))
        });
        let publications = self
            .meta
            .publications
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter().map(|value| TypedRelatedResource::related(value, "URL", "References", None)));
        let outputs = self.meta.outputs.as_ref().into_iter().flat_map(|values| {
            values.iter().filter_map(|output| {
                output.doi.as_ref().map(|doi| {
                    TypedRelatedResource::related(
                        doi,
                        "DOI",
                        "HasPart",
                        output.resource_type_general.as_ref().map(ResourceTypeGeneral::to_string).as_deref(),
                    )
                })
            })
        });
        let (values, warnings) = authored
            .chain(arxiv)
            .chain(raid)
            .chain(swhid)
            .chain(books)
            .chain(patents)
            .chain(publications)
            .chain(outputs)
            .enumerate()
            .fold((Vec::new(), Vec::new()), |(mut values, mut warnings), (index, value)| {
                match value.identifier.scheme.as_deref() {
                    | Some(_) => values.push(value),
                    | None => warnings.push(ConversionWarning::omitted(
                        RAD,
                        DATACITE,
                        format!("meta.relatedResources[{index}]"),
                        "DataCite related identifiers require an authored identifier scheme",
                    )),
                }
                (values, warnings)
            });
        Conversion::new(values, warnings)
    }
    fn project_related_items(&self) -> Conversion<Option<Vec<RelatedItem>>> {
        let (items, warnings) = self.meta.outputs.as_ref().into_iter().flat_map(|values| values.iter()).enumerate().fold(
            (Vec::new(), Vec::new()),
            |(mut items, mut warnings), (index, output)| {
                match &output.resource_type_general {
                    | Some(resource_type) => {
                        let creators = output.creators.as_ref().map(|creators| {
                            creators
                                .iter()
                                .map(|creator| RelatedItemCreator {
                                    name: creator.name.clone(),
                                    name_type: creator.name_type.clone(),
                                    given_name: None,
                                    family_name: None,
                                })
                                .collect()
                        });
                        let contributors = output.contributors.as_ref().map(|contributors| {
                            contributors
                                .iter()
                                .filter_map(|contributor| {
                                    contributor.contributor_type.clone().map(|contributor_type| RelatedItemContributor {
                                        name: contributor.name.clone(),
                                        contributor_type,
                                        name_type: contributor.name_type.clone(),
                                        given_name: None,
                                        family_name: None,
                                    })
                                })
                                .collect::<Vec<_>>()
                        });
                        items.push(RelatedItem {
                            related_item_type: resource_type.clone(),
                            relation_type: RelationType::HasPart,
                            relation_type_information: None,
                            related_item_identifier: output.doi.as_ref().map(|doi| RelatedItemIdentifier {
                                related_item_identifier: DOI::format(doi),
                                related_item_identifier_type: Some(datacite::RelatedIdentifierType::Doi),
                                related_metadata_scheme: None,
                                scheme_uri: Some("https://doi.org".to_string()),
                                scheme_type: None,
                            }),
                            creators,
                            titles: Some(vec![RelatedItemTitle {
                                title: output.title.clone(),
                                title_type: None,
                            }]),
                            publication_year: output.publication_year.map(|year| year.to_string()),
                            volume: None,
                            issue: None,
                            number: None,
                            first_page: None,
                            last_page: None,
                            publisher: output.publisher.as_ref().map(|publisher| publisher.name.clone()),
                            edition: output.version.clone(),
                            contributors,
                        });
                        warnings.push(ConversionWarning::simplified(
                            RAD,
                            DATACITE,
                            format!("meta.outputs[{index}]"),
                            "the canonical RAD output is represented as a DataCite RelatedItem",
                        ));
                    }
                    | None => warnings.push(ConversionWarning::omitted(
                        RAD,
                        DATACITE,
                        format!("meta.outputs[{index}]"),
                        "a DataCite RelatedItem requires resourceTypeGeneral",
                    )),
                }
                (items, warnings)
            },
        );
        Conversion::new((!items.is_empty()).then_some(items), warnings)
    }
    fn project_rights(&self) -> Vec<StructuredRights> {
        self.meta
            .rights
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter().cloned())
            .chain(self.meta.access.iter().cloned().filter_map(|mut rights| {
                rights.access = None;
                (rights.rights.is_some()
                    || rights.rights_uri.is_some()
                    || rights.rights_identifier.is_some()
                    || rights.rights_identifier_scheme.is_some()
                    || rights.scheme_uri.is_some())
                .then_some(rights)
            }))
            .collect()
    }
}
impl TypedRelatedResource {
    fn related(value: &str, scheme: &str, relation_type: &str, resource_type_general: Option<&str>) -> Self {
        Self {
            identifier: TypedIdentifier {
                value: value.to_string(),
                scheme: Some(scheme.to_string()),
                scheme_uri: None,
            },
            relation_type: relation_type.to_string(),
            relation_type_information: None,
            resource_type_general: resource_type_general.map(ToString::to_string),
            category: None,
        }
    }
}
