use crate::pid::{PersistentIdentifierParse, DOI, PID};
use crate::research_activity::output::{Affiliation, Award, Contributor, Funding, ResearchOutput};
use crate::research_activity::{
    ActivityDateType, ControlledSubject, RightsAccess, StructuredRights, TypedActivityDate, TypedIdentifier, TypedRelatedResource,
};
use crate::standard::crosswalk::CrosswalkSchema::{DataCite as DATACITE, RadOutput as RAD_OUTPUT};
use crate::standard::crosswalk::{Conversion, ConversionWarning, Crosswalk, CrosswalkError};
use crate::standard::datacite::{
    self, AlternateIdentifier, Attributes, Creator, Date, DateType, FunderIdentifierType, FundingReference, NameIdentifier, RelatedIdentifier,
    RelatedIdentifierType, ResourceTypes, Rights, Subject, Title,
};
use crate::{OneOrMany, Website};
use acorn_core::prelude::alloc::{format, vec, String, ToString, Vec};
use serde::de::DeserializeOwned;
use serde::Serialize;

const PROVIDER_IDENTIFIER_TYPE: &str = "ACORN Provider";
type FundingIdentity = (Option<String>, Option<FunderIdentifierType>, Option<String>);
struct OutputIdentifiers {
    identifier: Option<String>,
    alternate_identifiers: Option<Vec<TypedIdentifier>>,
}
struct OutputSubjects {
    subjects: Option<Vec<ControlledSubject>>,
    keywords: Vec<String>,
}
impl Contributor {
    fn warnings(&self, creator: bool) -> Vec<ConversionWarning> {
        let locations = [
            self.corresponding.map(|_| "corresponding"),
            self.position.as_ref().map(|_| "position"),
            (!self.roles.is_empty()).then_some("roles"),
            (creator && self.contributor_type.is_some()).then_some("contributorType"),
        ];
        locations
            .into_iter()
            .flatten()
            .map(|field| {
                ConversionWarning::no_equivalent(
                    RAD_OUTPUT,
                    DATACITE,
                    format!("{}[].{field}", if creator { "creators" } else { "contributors" }),
                )
            })
            .collect()
    }
}
impl From<&ResearchOutput> for Conversion<Option<Vec<Subject>>> {
    fn from(value: &ResearchOutput) -> Self {
        let structured = value.subjects.as_deref().unwrap_or_default().iter().map(|subject| {
            let warning = (subject.subject_scheme.is_none() && subject.scheme_uri.is_none() && subject.value_uri.is_none()).then(|| {
                ConversionWarning::approximated(
                    RAD_OUTPUT,
                    DATACITE,
                    "subjects[]",
                    "an unqualified structured subject is indistinguishable from an output keyword",
                )
            });
            (
                Subject {
                    subject: subject.subject.clone(),
                    language: None,
                    subject_scheme: subject.subject_scheme.clone(),
                    scheme_uri: subject.scheme_uri.clone(),
                    value_uri: subject.value_uri.clone(),
                    classification_code: None,
                },
                warning,
            )
        });
        let keywords = value.keywords.iter().map(|keyword| {
            (
                Subject {
                    subject: keyword.clone(),
                    language: None,
                    subject_scheme: None,
                    scheme_uri: None,
                    value_uri: None,
                    classification_code: None,
                },
                None,
            )
        });
        let converted = structured.chain(keywords).collect::<Vec<_>>();
        let subjects = (!converted.is_empty()).then(|| converted.iter().map(|(subject, _)| subject.clone()).collect());
        let warnings = converted.into_iter().filter_map(|(_, warning)| warning).collect();
        Self::new(subjects, warnings)
    }
}
impl From<Award> for Conversion<Option<String>> {
    fn from(value: Award) -> Self {
        let doi_uri = value.doi.as_ref().map(|doi| format!("https://doi.org/{}", DOI::format(doi)));
        let conflict = doi_uri
            .as_ref()
            .zip(value.url.as_ref())
            .filter(|(doi, url)| DOI::normalize(doi) != DOI::normalize(url))
            .map(|_| {
                ConversionWarning::omitted(
                    RAD_OUTPUT,
                    DATACITE,
                    "funding[].awards[].url",
                    "DataCite has one awardURI; the grant DOI was retained",
                )
            });
        Self::new(doi_uri.or(value.url), conflict.into_iter().collect())
    }
}
impl From<Contributor> for Conversion<Creator> {
    fn from(value: Contributor) -> Self {
        let warnings = value.warnings(true);
        let name_identifiers = Conversion::<Option<Vec<NameIdentifier>>>::from(value.clone()).value;
        Self::new(
            Creator {
                name: value.name,
                name_type: value.name_type,
                given_name: None,
                family_name: None,
                name_identifiers,
                affiliation: (!value.affiliations.is_empty()).then(|| value.affiliations.into_iter().map(datacite::Affiliation::from).collect()),
            },
            warnings,
        )
    }
}
impl From<Contributor> for Conversion<Option<Vec<NameIdentifier>>> {
    fn from(value: Contributor) -> Self {
        let identifiers = [
            value.orcid.as_ref().map(|orcid| Contributor {
                orcid: Some(orcid.clone()),
                ror: None,
                ..value.clone()
            }),
            value.ror.as_ref().map(|ror| Contributor {
                orcid: None,
                ror: Some(ror.clone()),
                ..value
            }),
        ]
        .into_iter()
        .flatten()
        .map(NameIdentifier::try_from)
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
        Self::new((!identifiers.is_empty()).then_some(identifiers), Vec::new())
    }
}
impl From<Creator> for Conversion<Contributor> {
    fn from(value: Creator) -> Self {
        let Conversion {
            value: (orcid, ror),
            warnings: identifier_warnings,
        } = Conversion::from(value.name_identifiers);
        let affiliation = collect_conversions(value.affiliation.unwrap_or_default().into_iter().map(Conversion::<Affiliation>::from));
        let warnings = identifier_warnings
            .into_iter()
            .chain(affiliation.warnings)
            .chain(person_fields_warning(&value.given_name, &value.family_name, "creators[]"))
            .collect();
        Self::new(
            Contributor {
                name: value.name,
                name_type: value.name_type,
                orcid,
                ror,
                contributor_type: None,
                roles: Vec::new(),
                corresponding: None,
                position: None,
                affiliations: affiliation.value,
            },
            warnings,
        )
    }
}
impl From<datacite::Affiliation> for Conversion<Affiliation> {
    fn from(value: datacite::Affiliation) -> Self {
        let is_ror = value
            .affiliation_identifier_scheme
            .as_deref()
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("ROR"));
        let ror = is_ror.then(|| value.affiliation_identifier.clone()).flatten();
        let identifier_warning = value
            .affiliation_identifier
            .as_ref()
            .filter(|_| !is_ror)
            .map(|_| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "affiliation[].affiliationIdentifier"));
        let scheme_warning = value
            .scheme_uri
            .as_deref()
            .filter(|uri| is_ror && uri.trim_end_matches('/') != "https://ror.org")
            .map(|_| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "affiliation[].schemeURI"));
        let orphaned_scheme_warning = (value.affiliation_identifier.is_none()
            && (value.affiliation_identifier_scheme.is_some() || value.scheme_uri.is_some()))
        .then(|| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "affiliation[].affiliationIdentifierScheme"));
        Self::new(
            Affiliation { name: value.name, ror },
            [identifier_warning, scheme_warning, orphaned_scheme_warning]
                .into_iter()
                .flatten()
                .collect(),
        )
    }
}
impl From<datacite::Contributor> for Conversion<Contributor> {
    fn from(value: datacite::Contributor) -> Self {
        let Conversion {
            value: (orcid, ror),
            warnings: identifier_warnings,
        } = Conversion::from(value.name_identifiers);
        let affiliation = collect_conversions(value.affiliation.unwrap_or_default().into_iter().map(Conversion::<Affiliation>::from));
        let warnings = identifier_warnings
            .into_iter()
            .chain(affiliation.warnings)
            .chain(person_fields_warning(&value.given_name, &value.family_name, "contributors[]"))
            .collect();
        Self::new(
            Contributor {
                name: value.name,
                name_type: value.name_type,
                orcid,
                ror,
                contributor_type: Some(value.contributor_type),
                roles: Vec::new(),
                corresponding: None,
                position: None,
                affiliations: affiliation.value,
            },
            warnings,
        )
    }
}
impl From<FundingReference> for Conversion<Funding> {
    fn from(value: FundingReference) -> Self {
        let (identifier, identifier_type, ror) = match value.funder_identifier_type {
            | Some(FunderIdentifierType::Ror) => (None, None, value.funder_identifier.clone()),
            | identifier_type => (value.funder_identifier.clone(), identifier_type, None),
        };
        let award = value.award_number.as_ref().map(|award_number| {
            let doi = value.award_uri.as_deref().map(DOI::format).filter(|value| !value.is_empty());
            Award {
                identifier: award_number.clone(),
                title: value.award_title.clone(),
                doi,
                url: value
                    .award_uri
                    .clone()
                    .filter(|_| value.award_uri.as_deref().map(DOI::format).is_some_and(|doi| doi.is_empty())),
            }
        });
        let orphaned_award_warnings = [
            (value.award_number.is_none() && value.award_uri.is_some()).then(|| {
                ConversionWarning::omitted(
                    DATACITE,
                    RAD_OUTPUT,
                    "fundingReferences[].awardURI",
                    "award metadata without awardNumber cannot be represented as a RAD award",
                )
            }),
            (value.award_number.is_none() && value.award_title.is_some()).then(|| {
                ConversionWarning::omitted(
                    DATACITE,
                    RAD_OUTPUT,
                    "fundingReferences[].awardTitle",
                    "award metadata without awardNumber cannot be represented as a RAD award",
                )
            }),
        ];
        Self::new(
            Funding {
                name: value.funder_name,
                identifier,
                identifier_type,
                scheme_uri: value.scheme_uri,
                ror,
                awards: award.into_iter().collect(),
            },
            orphaned_award_warnings.into_iter().flatten().collect(),
        )
    }
}
impl From<Option<Vec<NameIdentifier>>> for Conversion<(Option<String>, Option<String>)> {
    fn from(values: Option<Vec<NameIdentifier>>) -> Self {
        values.map_or_else(
            || Self::new((None, None), Vec::new()),
            |values| {
                let (orcids, rors, unsupported) = values.into_iter().fold(
                    (Vec::new(), Vec::new(), Vec::new()),
                    |(mut orcids, mut rors, mut unsupported), identifier| {
                        let scheme = identifier.name_identifier_scheme.as_deref().map(str::to_ascii_uppercase);
                        match scheme.as_deref() {
                            | Some("ORCID") => {
                                orcids.push(identifier.name_identifier);
                                if identifier
                                    .scheme_uri
                                    .as_deref()
                                    .is_some_and(|uri| uri.trim_end_matches('/') != "https://orcid.org")
                                {
                                    unsupported.push(ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "nameIdentifiers[].schemeUri"));
                                }
                            }
                            | Some("ROR") => {
                                rors.push(identifier.name_identifier);
                                if identifier
                                    .scheme_uri
                                    .as_deref()
                                    .is_some_and(|uri| uri.trim_end_matches('/') != "https://ror.org")
                                {
                                    unsupported.push(ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "nameIdentifiers[].schemeUri"));
                                }
                            }
                            | _ => unsupported.push(ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "nameIdentifiers[].nameIdentifier")),
                        }
                        (orcids, rors, unsupported)
                    },
                );
                let duplicate_warnings = [
                    (orcids.len() > 1).then(|| {
                        ConversionWarning::simplified(
                            DATACITE,
                            RAD_OUTPUT,
                            "nameIdentifiers[]",
                            "only one ORCID can be stored per output contributor",
                        )
                    }),
                    (rors.len() > 1).then(|| {
                        ConversionWarning::simplified(
                            DATACITE,
                            RAD_OUTPUT,
                            "nameIdentifiers[]",
                            "only one ROR can be stored per output contributor",
                        )
                    }),
                ];
                Self::new(
                    (orcids.into_iter().next(), rors.into_iter().next()),
                    unsupported.into_iter().chain(duplicate_warnings.into_iter().flatten()).collect(),
                )
            },
        )
    }
}
impl From<Option<Vec<Subject>>> for Conversion<OutputSubjects> {
    fn from(values: Option<Vec<Subject>>) -> Self {
        let (structured, keywords, warnings) = values.unwrap_or_default().into_iter().fold(
            (Vec::new(), Vec::new(), Vec::new()),
            |(mut structured, mut keywords, mut warnings), subject| {
                let is_structured = subject.subject_scheme.is_some() || subject.scheme_uri.is_some() || subject.value_uri.is_some();
                match is_structured {
                    | true => structured.push(ControlledSubject {
                        subject: subject.subject,
                        subject_scheme: subject.subject_scheme,
                        scheme_uri: subject.scheme_uri,
                        value_uri: subject.value_uri,
                    }),
                    | false => keywords.push(subject.subject),
                }
                if subject.language.is_some() {
                    warnings.push(ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "subjects[].language"));
                }
                if subject.classification_code.is_some() {
                    warnings.push(ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "subjects[].classificationCode"));
                }
                (structured, keywords, warnings)
            },
        );
        Self::new(
            OutputSubjects {
                subjects: (!structured.is_empty()).then_some(structured),
                keywords,
            },
            warnings,
        )
    }
}
impl From<Rights> for Conversion<StructuredRights> {
    fn from(value: Rights) -> Self {
        let identifiers = value.rights_identifier.map(OneOrMany::into_vec).unwrap_or_default();
        let warnings = if identifiers.len() > 1 {
            vec![ConversionWarning::simplified(
                DATACITE,
                RAD_OUTPUT,
                "rightsList[].rightsIdentifier",
                "only one rights identifier can be stored per RAD rights statement",
            )]
        } else {
            Vec::new()
        };
        Self::new(
            StructuredRights {
                rights: value.rights,
                rights_uri: value.rights_uri,
                rights_identifier: identifiers.into_iter().next(),
                rights_identifier_scheme: value.rights_identifier_scheme,
                scheme_uri: value.scheme_uri,
                access: None::<RightsAccess>,
            },
            warnings,
        )
    }
}
impl From<StructuredRights> for Conversion<Rights> {
    fn from(value: StructuredRights) -> Self {
        let warnings = value
            .access
            .as_ref()
            .map(|_| vec![ConversionWarning::no_equivalent(RAD_OUTPUT, DATACITE, "rightsList[].access")])
            .unwrap_or_default();
        Self::new(
            Rights {
                rights: value.rights,
                rights_uri: value.rights_uri,
                rights_identifier: value.rights_identifier.map(OneOrMany::One),
                rights_identifier_scheme: value.rights_identifier_scheme,
                scheme_uri: value.scheme_uri,
            },
            warnings,
        )
    }
}
impl From<TypedActivityDate> for Conversion<Date> {
    fn from(value: TypedActivityDate) -> Self {
        let (date_type, warnings) = match value.date_type {
            | ActivityDateType::Accepted => (DateType::Accepted, Vec::new()),
            | ActivityDateType::Available => (DateType::Available, Vec::new()),
            | ActivityDateType::Collected => (DateType::Collected, Vec::new()),
            | ActivityDateType::Copyrighted => (DateType::Copyrighted, Vec::new()),
            | ActivityDateType::Coverage => (DateType::Coverage, Vec::new()),
            | ActivityDateType::Created => (DateType::Created, Vec::new()),
            | ActivityDateType::Issued => (DateType::Issued, Vec::new()),
            | ActivityDateType::Other => (DateType::Other, Vec::new()),
            | ActivityDateType::Submitted => (DateType::Submitted, Vec::new()),
            | ActivityDateType::Updated => (DateType::Updated, Vec::new()),
            | ActivityDateType::Valid => (DateType::Valid, Vec::new()),
            | ActivityDateType::Withdrawn => (DateType::Withdrawn, Vec::new()),
            | ActivityDateType::StartDate | ActivityDateType::EndDate => (
                DateType::Other,
                vec![ConversionWarning::approximated(
                    RAD_OUTPUT,
                    DATACITE,
                    "dates[].dateType",
                    "activity interval boundary is represented as DataCite Other",
                )],
            ),
        };
        Self::new(
            Date {
                date: value.date,
                date_type,
                date_information: value.information,
            },
            warnings,
        )
    }
}
impl From<Vec<FundingReference>> for Conversion<Vec<Funding>> {
    fn from(values: Vec<FundingReference>) -> Self {
        let converted = collect_conversions(values.into_iter().map(Conversion::<Funding>::from));
        let fundings = converted.value.into_iter().fold(Vec::<Funding>::new(), |mut grouped, funding| {
            let existing = grouped.iter_mut().find(|candidate| {
                candidate.name == funding.name
                    && candidate.identifier == funding.identifier
                    && candidate.identifier_type == funding.identifier_type
                    && candidate.scheme_uri == funding.scheme_uri
                    && candidate.ror == funding.ror
            });
            match existing {
                | Some(existing) => existing.awards.extend(funding.awards),
                | None => grouped.push(funding),
            }
            grouped
        });
        Self::new(fundings, converted.warnings)
    }
}
impl TryFrom<&ResearchOutput> for Conversion<Option<Vec<AlternateIdentifier>>> {
    type Error = CrosswalkError;

    fn try_from(value: &ResearchOutput) -> Result<Self, Self::Error> {
        let provider = value.identifier.iter().map(|identifier| AlternateIdentifier {
            alternate_identifier: identifier.clone(),
            alternate_identifier_type: PROVIDER_IDENTIFIER_TYPE.to_string(),
        });
        value
            .alternate_identifiers
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|identifier| {
                identifier
                    .scheme
                    .clone()
                    .ok_or_else(|| CrosswalkError::MissingRequiredField("alternateIdentifiers[].scheme".to_string()))
                    .map(|scheme| {
                        (
                            AlternateIdentifier {
                                alternate_identifier: identifier.value.clone(),
                                alternate_identifier_type: scheme,
                            },
                            identifier
                                .scheme_uri
                                .as_ref()
                                .map(|_| ConversionWarning::no_equivalent(RAD_OUTPUT, DATACITE, "alternateIdentifiers[].schemeURI")),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|converted| {
                let values = provider
                    .chain(converted.iter().map(|(identifier, _)| identifier.clone()))
                    .collect::<Vec<_>>();
                let warnings = converted.into_iter().filter_map(|(_, warning)| warning).collect();
                Self::new((!values.is_empty()).then_some(values), warnings)
            })
    }
}
impl TryFrom<Contributor> for Conversion<datacite::Contributor> {
    type Error = CrosswalkError;

    fn try_from(value: Contributor) -> Result<Self, Self::Error> {
        let contributor_type = value
            .contributor_type
            .clone()
            .ok_or_else(|| CrosswalkError::MissingRequiredField("contributors[].contributorType".to_string()))?;
        let warnings = value.warnings(false);
        let name_identifiers = Conversion::<Option<Vec<NameIdentifier>>>::from(value.clone()).value;
        Ok(Self::new(
            datacite::Contributor {
                name: value.name,
                contributor_type,
                name_type: value.name_type,
                given_name: None,
                family_name: None,
                name_identifiers,
                affiliation: (!value.affiliations.is_empty()).then(|| value.affiliations.into_iter().map(datacite::Affiliation::from).collect()),
            },
            warnings,
        ))
    }
}
impl TryFrom<Funding> for Conversion<Vec<FundingReference>> {
    type Error = CrosswalkError;

    fn try_from(value: Funding) -> Result<Self, Self::Error> {
        value
            .identity()
            .map(|(identifier, identifier_type, scheme_uri)| match value.awards.is_empty() {
                | true => Self::new(
                    vec![FundingReference {
                        funder_name: value.name,
                        funder_identifier: identifier,
                        funder_identifier_type: identifier_type,
                        scheme_uri,
                        award_number: None,
                        award_uri: None,
                        award_title: None,
                    }],
                    Vec::new(),
                ),
                | false => {
                    let funder_name = value.name;
                    let converted = value.awards.into_iter().map(|award| {
                        let award_title = award.title.clone();
                        let award_number = award.identifier.clone();
                        let award_uri = Conversion::<Option<String>>::from(award);
                        Conversion::new(
                            FundingReference {
                                funder_name: funder_name.clone(),
                                funder_identifier: identifier.clone(),
                                funder_identifier_type: identifier_type.clone(),
                                scheme_uri: scheme_uri.clone(),
                                award_number: Some(award_number),
                                award_uri: award_uri.value,
                                award_title,
                            },
                            award_uri.warnings,
                        )
                    });
                    collect_conversions(converted)
                }
            })
    }
}
impl TryFrom<RelatedIdentifier> for Conversion<TypedRelatedResource> {
    type Error = CrosswalkError;

    fn try_from(value: RelatedIdentifier) -> Result<Self, Self::Error> {
        match value.related_identifier_type.as_ref() {
            | Some(RelatedIdentifierType::Rrid) => value.crosswalk().map(Self::from),
            | _ => value
                .related_identifier_type
                .as_ref()
                .ok_or_else(|| CrosswalkError::MissingRequiredField("relatedIdentifiers[].relatedIdentifierType".to_string()))
                .and_then(|identifier_type| serialize_enum(identifier_type, "relatedIdentifiers[].relatedIdentifierType"))
                .and_then(|scheme| {
                    value
                        .relation_type
                        .as_ref()
                        .ok_or_else(|| CrosswalkError::MissingRequiredField("relatedIdentifiers[].relationType".to_string()))
                        .map(|relation_type| (scheme, relation_type.to_string()))
                })
                .map(|(scheme, relation_type)| {
                    let warnings = [
                        value
                            .related_metadata_scheme
                            .as_ref()
                            .map(|_| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "relatedIdentifiers[].relatedMetadataScheme")),
                        value
                            .scheme_type
                            .as_ref()
                            .map(|_| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "relatedIdentifiers[].schemeType")),
                    ]
                    .into_iter()
                    .flatten()
                    .collect();
                    Self::new(
                        TypedRelatedResource {
                            identifier: TypedIdentifier {
                                value: value.related_identifier,
                                scheme: Some(scheme),
                                scheme_uri: value.scheme_uri,
                            },
                            relation_type,
                            relation_type_information: value.relation_type_information,
                            resource_type_general: value.resource_type_general.as_ref().map(ToString::to_string),
                            category: None,
                        },
                        warnings,
                    )
                }),
        }
    }
}
impl TryFrom<TypedRelatedResource> for Conversion<RelatedIdentifier> {
    type Error = CrosswalkError;

    fn try_from(value: TypedRelatedResource) -> Result<Self, Self::Error> {
        let is_rrid = value.identifier.scheme.as_deref().is_some_and(|scheme| PID::from(scheme) == PID::RRID);
        match is_rrid {
            | true => value.crosswalk().map(Self::from),
            | false => value
                .identifier
                .scheme
                .as_deref()
                .ok_or_else(|| CrosswalkError::MissingRequiredField("relatedIdentifiers[].identifier.scheme".to_string()))
                .and_then(|scheme| deserialize_enum::<RelatedIdentifierType>(scheme, "relatedIdentifiers[].identifier.scheme"))
                .and_then(|related_identifier_type| {
                    value
                        .relation_type
                        .parse::<datacite::RelationType>()
                        .map_err(|error| CrosswalkError::TransformationFailed {
                            field: "relatedIdentifiers[].relationType".to_string(),
                            reason: error.to_string(),
                        })
                        .map(|relation_type| (related_identifier_type, relation_type))
                })
                .and_then(|(related_identifier_type, relation_type)| {
                    value
                        .resource_type_general
                        .as_deref()
                        .map(|resource_type| {
                            resource_type
                                .parse::<datacite::ResourceTypeGeneral>()
                                .map_err(|error| CrosswalkError::TransformationFailed {
                                    field: "relatedIdentifiers[].resourceTypeGeneral".to_string(),
                                    reason: error.to_string(),
                                })
                        })
                        .transpose()
                        .map(|resource_type_general| (related_identifier_type, relation_type, resource_type_general))
                })
                .map(|(related_identifier_type, relation_type, resource_type_general)| {
                    let warnings = value
                        .category
                        .as_ref()
                        .map(|_| vec![ConversionWarning::no_equivalent(RAD_OUTPUT, DATACITE, "relatedIdentifiers[].category")])
                        .unwrap_or_default();
                    Self::new(
                        RelatedIdentifier {
                            related_identifier: value.identifier.value,
                            related_identifier_type: Some(related_identifier_type),
                            relation_type: Some(relation_type),
                            related_metadata_scheme: None,
                            scheme_uri: value.identifier.scheme_uri,
                            scheme_type: None,
                            resource_type_general,
                            relation_type_information: value.relation_type_information,
                        },
                        warnings,
                    )
                }),
        }
    }
}
impl From<Affiliation> for datacite::Affiliation {
    fn from(value: Affiliation) -> Self {
        let identifier_scheme = value.ror.as_ref().map(|_| "ROR".to_string());
        let scheme_uri = value.ror.as_ref().map(|_| "https://ror.org".to_string());
        Self {
            name: value.name,
            affiliation_identifier: value.ror,
            affiliation_identifier_scheme: identifier_scheme,
            scheme_uri,
        }
    }
}
impl Crosswalk<ResearchOutput> for datacite::Record {
    fn crosswalk(&self) -> Result<(ResearchOutput, Vec<ConversionWarning>), CrosswalkError> {
        self.attributes
            .titles
            .as_ref()
            .and_then(|titles| titles.first())
            .filter(|title| !title.title.trim().is_empty())
            .ok_or_else(|| CrosswalkError::MissingRequiredField("titles".to_string()))
            .and_then(|title| {
                let creators = self
                    .attributes
                    .creators
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .cloned()
                    .map(Conversion::<Contributor>::from);
                let creators = collect_conversions(creators);
                let contributors = self
                    .attributes
                    .contributors
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .cloned()
                    .map(Conversion::<Contributor>::from);
                let contributors = collect_conversions(contributors);
                let related = self
                    .attributes
                    .related_identifiers
                    .as_deref()
                    .map(try_convert_all::<RelatedIdentifier, TypedRelatedResource>)
                    .transpose();
                related.map(|related| (title, creators, contributors, related))
            })
            .map(|(title, creators, contributors, related)| {
                let related = related.unwrap_or_else(|| Conversion::new(Vec::new(), Vec::new()));
                let OutputIdentifiers {
                    identifier,
                    alternate_identifiers,
                } = OutputIdentifiers::from(self.attributes.alternate_identifiers.clone());
                let subjects = Conversion::<OutputSubjects>::from(self.attributes.subjects.clone());
                let funding = Conversion::<Vec<Funding>>::from(self.attributes.funding_references.as_deref().unwrap_or_default().to_vec());
                let rights = collect_conversions(
                    self.attributes
                        .rights_list
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .cloned()
                        .map(Conversion::<StructuredRights>::from),
                );
                let dates = self
                    .attributes
                    .dates
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .cloned()
                    .map(TypedActivityDate::from)
                    .collect::<Vec<_>>();
                let additional_titles = self
                    .attributes
                    .titles
                    .as_ref()
                    .is_some_and(|titles| titles.len() > 1)
                    .then(|| ConversionWarning::omitted(DATACITE, RAD_OUTPUT, "titles[1..]", "ResearchOutput stores one title"));
                let title_type_warning = title
                    .title_type
                    .as_ref()
                    .map(|_| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "titles[0].titleType"));
                let record_id_warning = (self.id != self.attributes.doi).then(|| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "id"));
                let event_warning = self
                    .attributes
                    .event
                    .as_ref()
                    .map(|_| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "event"));
                let schema_warning = self
                    .attributes
                    .schema_version
                    .as_ref()
                    .map(|_| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "schemaVersion"));
                let kind_warning = (self.kind != "dois").then(|| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, "type"));
                let url_warning = self.attributes.url.as_ref().map(|_| {
                    ConversionWarning::approximated(DATACITE, RAD_OUTPUT, "url", "the URL is retained with an empty local website description")
                });
                let output = ResearchOutput {
                    identifier,
                    doi: (!self.attributes.doi.trim().is_empty()).then(|| self.attributes.doi.clone()),
                    title: title.title.clone(),
                    kind: None,
                    publication_year: self.attributes.publication_year,
                    dates: (!dates.is_empty()).then_some(dates),
                    publisher: self.attributes.publisher.clone(),
                    resource_type_general: self
                        .attributes
                        .resource_types
                        .as_ref()
                        .and_then(|resource_types| resource_types.resource_type_general.clone()),
                    resource_type: self
                        .attributes
                        .resource_types
                        .as_ref()
                        .and_then(|resource_types| resource_types.resource_type.clone()),
                    language: self.attributes.language.clone(),
                    alternate_identifiers,
                    related_identifiers: (!related.value.is_empty()).then_some(related.value),
                    related_items: self.attributes.related_items.clone(),
                    version: self.attributes.version.clone(),
                    rights_list: (!rights.value.is_empty()).then_some(rights.value),
                    descriptions: self.attributes.descriptions.clone(),
                    subjects: subjects.value.subjects,
                    formats: self.attributes.formats.clone(),
                    sizes: self.attributes.sizes.clone(),
                    geo_locations: self.attributes.geo_locations.clone(),
                    creators: (!creators.value.is_empty()).then_some(creators.value),
                    contributors: (!contributors.value.is_empty()).then_some(contributors.value),
                    funding: (!funding.value.is_empty()).then_some(funding.value),
                    websites: self.attributes.url.as_ref().map(|url| {
                        vec![Website {
                            url: url.clone(),
                            description: String::new(),
                        }]
                    }),
                    keywords: subjects.value.keywords,
                    access: None,
                };
                let warnings = creators
                    .warnings
                    .into_iter()
                    .chain(contributors.warnings)
                    .chain(related.warnings)
                    .chain(subjects.warnings)
                    .chain(funding.warnings)
                    .chain(rights.warnings)
                    .chain(
                        [
                            additional_titles,
                            title_type_warning,
                            record_id_warning,
                            event_warning,
                            schema_warning,
                            kind_warning,
                            url_warning,
                        ]
                        .into_iter()
                        .flatten(),
                    )
                    .collect();
                (output, warnings)
            })
    }
}
impl Funding {
    fn identity(&self) -> Result<FundingIdentity, CrosswalkError> {
        match (&self.identifier, &self.ror) {
            | (Some(identifier), Some(ror)) if identifier != ror => Err(CrosswalkError::TransformationFailed {
                field: "funding[].identifier".to_string(),
                reason: "identifier conflicts with the compatibility ROR field".to_string(),
            }),
            | (Some(_), Some(_)) if self.identifier_type.as_ref().is_some_and(|kind| kind != &FunderIdentifierType::Ror) => {
                Err(CrosswalkError::TransformationFailed {
                    field: "funding[].identifierType".to_string(),
                    reason: "a compatibility ROR identifier cannot use a different registry type".to_string(),
                })
            }
            | (Some(identifier), Some(_)) => Ok((Some(identifier.clone()), Some(FunderIdentifierType::Ror), self.scheme_uri.clone())),
            | (Some(identifier), None) => self
                .identifier_type
                .clone()
                .ok_or_else(|| CrosswalkError::MissingRequiredField("funding[].identifierType".to_string()))
                .map(|identifier_type| (Some(identifier.clone()), Some(identifier_type), self.scheme_uri.clone())),
            | (None, Some(ror)) => Ok((Some(ror.clone()), Some(FunderIdentifierType::Ror), self.scheme_uri.clone())),
            | (None, None) if self.identifier_type.is_some() || self.scheme_uri.is_some() => {
                Err(CrosswalkError::MissingRequiredField("funding[].identifier".to_string()))
            }
            | (None, None) => Ok((None, None, None)),
        }
    }
}
impl TryFrom<Contributor> for NameIdentifier {
    type Error = CrosswalkError;

    fn try_from(value: Contributor) -> Result<Self, Self::Error> {
        match (value.orcid, value.ror) {
            | (Some(identifier), None) => Ok(Self {
                name_identifier: identifier,
                name_identifier_scheme: Some("ORCID".to_string()),
                scheme_uri: Some("https://orcid.org".to_string()),
            }),
            | (None, Some(identifier)) => Ok(Self {
                name_identifier: identifier,
                name_identifier_scheme: Some("ROR".to_string()),
                scheme_uri: Some("https://ror.org".to_string()),
            }),
            | (Some(_), Some(_)) => Err(CrosswalkError::TransformationFailed {
                field: "contributors[].nameIdentifiers".to_string(),
                reason: "one NameIdentifier represents either an ORCID or a ROR, not both".to_string(),
            }),
            | (None, None) => Err(CrosswalkError::MissingRequiredField(
                "contributors[].orcid or contributors[].ror".to_string(),
            )),
        }
    }
}
impl From<Option<Vec<AlternateIdentifier>>> for OutputIdentifiers {
    fn from(values: Option<Vec<AlternateIdentifier>>) -> Self {
        let values = values.unwrap_or_default();
        let identifier = values
            .iter()
            .find(|identifier| identifier.alternate_identifier_type == PROVIDER_IDENTIFIER_TYPE)
            .map(|identifier| identifier.alternate_identifier.clone());
        let alternate_identifiers = values
            .into_iter()
            .scan(false, |removed_provider, value| {
                let is_provider = value.alternate_identifier_type == PROVIDER_IDENTIFIER_TYPE
                    && identifier.as_ref().is_some_and(|identifier| identifier == &value.alternate_identifier);
                match is_provider && !*removed_provider {
                    | true => {
                        *removed_provider = true;
                        Some(None)
                    }
                    | false => Some(Some(TypedIdentifier {
                        value: value.alternate_identifier,
                        scheme: Some(value.alternate_identifier_type),
                        scheme_uri: None,
                    })),
                }
            })
            .flatten()
            .collect::<Vec<_>>();
        Self {
            identifier,
            alternate_identifiers: (!alternate_identifiers.is_empty()).then_some(alternate_identifiers),
        }
    }
}

impl Crosswalk<datacite::Record> for ResearchOutput {
    fn crosswalk(&self) -> Result<(datacite::Record, Vec<ConversionWarning>), CrosswalkError> {
        self.validate_deposit_ready().map_err(missing_output_fields).and_then(|_| {
            let creators = self
                .creators
                .as_deref()
                .unwrap_or_default()
                .iter()
                .cloned()
                .map(Conversion::<Creator>::from);
            let creators = collect_conversions(creators);
            let contributors = self
                .contributors
                .as_deref()
                .map(try_convert_all::<Contributor, datacite::Contributor>)
                .transpose();
            let alternate_identifiers = Conversion::<Option<Vec<AlternateIdentifier>>>::try_from(self);
            let related_identifiers = self
                .related_identifiers
                .as_deref()
                .map(try_convert_all::<TypedRelatedResource, RelatedIdentifier>)
                .transpose();
            let funding = self
                .funding
                .as_deref()
                .unwrap_or_default()
                .iter()
                .cloned()
                .map(Conversion::<Vec<FundingReference>>::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map(collect_conversions);
            contributors
                .and_then(|contributors| alternate_identifiers.map(|alternate| (contributors, alternate)))
                .and_then(|(contributors, alternate)| related_identifiers.map(|related| (contributors, alternate, related)))
                .and_then(|(contributors, alternate, related)| funding.map(|funding| (contributors, alternate, related, funding)))
                .map(|(contributors, alternate_identifiers, related, funding)| {
                    let contributors = contributors.unwrap_or_else(|| Conversion::new(Vec::new(), Vec::new()));
                    let related = related.unwrap_or_else(|| Conversion::new(Vec::new(), Vec::new()));
                    let subjects = Conversion::<Option<Vec<Subject>>>::from(self);
                    let rights = collect_conversions(
                        self.rights_list
                            .as_deref()
                            .unwrap_or_default()
                            .iter()
                            .cloned()
                            .map(Conversion::<Rights>::from),
                    );
                    let dates = collect_conversions(self.dates.as_deref().unwrap_or_default().iter().cloned().map(Conversion::<Date>::from));
                    let url = self
                        .websites
                        .as_ref()
                        .and_then(|values| values.first())
                        .map(|website| website.url.clone());
                    let mut warnings = creators
                        .warnings
                        .into_iter()
                        .chain(contributors.warnings)
                        .chain(alternate_identifiers.warnings)
                        .chain(related.warnings)
                        .chain(funding.warnings)
                        .chain(subjects.warnings)
                        .chain(rights.warnings)
                        .chain(dates.warnings)
                        .collect::<Vec<_>>();
                    warnings.extend(self.kind.as_ref().map(|_| ConversionWarning::no_equivalent(RAD_OUTPUT, DATACITE, "type")));
                    warnings.extend(
                        self.access
                            .as_ref()
                            .map(|_| ConversionWarning::no_equivalent(RAD_OUTPUT, DATACITE, "access")),
                    );
                    warnings.extend(self.websites.as_ref().into_iter().flat_map(|websites| {
                        websites.iter().enumerate().flat_map(|(index, website)| {
                            [
                                (!website.description.is_empty())
                                    .then(|| ConversionWarning::no_equivalent(RAD_OUTPUT, DATACITE, format!("websites[{index}].description"))),
                                (index > 0).then(|| {
                                    ConversionWarning::omitted(
                                        RAD_OUTPUT,
                                        DATACITE,
                                        format!("websites[{index}].url"),
                                        "DataCite JSON records expose one resource URL",
                                    )
                                }),
                            ]
                            .into_iter()
                            .flatten()
                        })
                    }));
                    let attributes = Attributes {
                        doi: self.doi.clone().unwrap_or_default(),
                        event: None,
                        titles: Some(vec![Title {
                            title: self.title.clone(),
                            title_type: None,
                        }]),
                        creators: Some(creators.value),
                        publisher: self.publisher.clone(),
                        publication_year: self.publication_year,
                        resource_types: Some(ResourceTypes {
                            resource_type_general: self.resource_type_general.clone(),
                            resource_type: self.resource_type.clone(),
                        }),
                        url,
                        subjects: subjects.value,
                        contributors: (!contributors.value.is_empty()).then_some(contributors.value),
                        dates: (!dates.value.is_empty()).then_some(dates.value),
                        language: self.language.clone(),
                        alternate_identifiers: alternate_identifiers.value,
                        related_identifiers: (!related.value.is_empty()).then_some(related.value),
                        related_items: self.related_items.clone(),
                        sizes: self.sizes.clone(),
                        formats: self.formats.clone(),
                        version: self.version.clone(),
                        rights_list: (!rights.value.is_empty()).then_some(rights.value),
                        descriptions: self.descriptions.clone(),
                        geo_locations: self.geo_locations.clone(),
                        funding_references: (!funding.value.is_empty()).then(|| funding.value.into_iter().flatten().collect()),
                        schema_version: None,
                    };
                    (
                        datacite::Record {
                            id: attributes.doi.clone(),
                            kind: "dois".to_string(),
                            attributes,
                        },
                        warnings,
                    )
                })
        })
    }
}
impl From<Date> for TypedActivityDate {
    fn from(value: Date) -> Self {
        let date_type = match value.date_type {
            | DateType::Accepted => ActivityDateType::Accepted,
            | DateType::Available => ActivityDateType::Available,
            | DateType::Collected => ActivityDateType::Collected,
            | DateType::Copyrighted => ActivityDateType::Copyrighted,
            | DateType::Coverage => ActivityDateType::Coverage,
            | DateType::Created => ActivityDateType::Created,
            | DateType::Issued => ActivityDateType::Issued,
            | DateType::Other => ActivityDateType::Other,
            | DateType::Submitted => ActivityDateType::Submitted,
            | DateType::Updated => ActivityDateType::Updated,
            | DateType::Valid => ActivityDateType::Valid,
            | DateType::Withdrawn => ActivityDateType::Withdrawn,
        };
        Self {
            date_type,
            date: value.date,
            information: value.date_information,
        }
    }
}
fn collect_conversions<T>(converted: impl IntoIterator<Item = Conversion<T>>) -> Conversion<Vec<T>> {
    let (values, warnings): (Vec<_>, Vec<_>) = converted.into_iter().map(Conversion::into_parts).unzip();
    Conversion::new(values, warnings.into_iter().flatten().collect())
}
fn deserialize_enum<T>(value: &str, field: &str) -> Result<T, CrosswalkError>
where
    T: DeserializeOwned,
{
    serde_json::from_value(serde_json::Value::String(value.to_string())).map_err(|error| CrosswalkError::TransformationFailed {
        field: field.to_string(),
        reason: error.to_string(),
    })
}
fn missing_output_fields(fields: Vec<String>) -> CrosswalkError {
    CrosswalkError::MissingRequiredField(fields.join(", "))
}
fn person_fields_warning(given_name: &Option<String>, family_name: &Option<String>, field: &str) -> Vec<ConversionWarning> {
    [
        given_name.as_ref().map(|_| format!("{field}.givenName")),
        family_name.as_ref().map(|_| format!("{field}.familyName")),
    ]
    .into_iter()
    .flatten()
    .map(|field| ConversionWarning::no_equivalent(DATACITE, RAD_OUTPUT, field))
    .collect()
}
fn serialize_enum<T>(value: &T, field: &str) -> Result<String, CrosswalkError>
where
    T: Serialize,
{
    serde_json::to_value(value)
        .map_err(|error| CrosswalkError::TransformationFailed {
            field: field.to_string(),
            reason: error.to_string(),
        })
        .and_then(|value| {
            value
                .as_str()
                .map(ToString::to_string)
                .ok_or_else(|| CrosswalkError::TransformationFailed {
                    field: field.to_string(),
                    reason: "controlled value did not serialize as a string".to_string(),
                })
        })
}
fn try_convert_all<S, T>(values: &[S]) -> Result<Conversion<Vec<T>>, CrosswalkError>
where
    S: Clone,
    Conversion<T>: TryFrom<S, Error = CrosswalkError>,
{
    values
        .iter()
        .cloned()
        .map(Conversion::<T>::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map(collect_conversions)
}
