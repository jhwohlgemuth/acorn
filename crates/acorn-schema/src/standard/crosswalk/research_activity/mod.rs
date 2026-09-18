//! Bidirectional crosswalks between ACORN research activities and RAiD metadata.
//!
//! Conversions return any lossy-field warnings alongside the converted value.
//! Callers can opt into documented inference when required RAiD assertions are unavailable.
//!
//! ### Examples
//!
//! Convert a research activity into RAiD metadata:
//!
//! ```rust
//! use acorn_schema::pid::raid;
//! use acorn_schema::research_activity::ResearchActivity;
//! use acorn_schema::standard::crosswalk::{CrosswalkError, RaidExportOptions};
//!
//! fn export(activity: &ResearchActivity) -> Result<raid::Metadata, CrosswalkError> {
//!     activity
//!         .to_raid(&RaidExportOptions::default())
//!         .map(|(metadata, _warnings)| metadata)
//! }
//! ```
//!
//! Convert RAiD metadata into a research activity:
//!
//! ```rust
//! use acorn_schema::pid::raid;
//! use acorn_schema::research_activity::ResearchActivity;
//! use acorn_schema::standard::crosswalk::{CrosswalkError, RaidImportOptions};
//!
//! fn import(metadata: &raid::Metadata, options: &RaidImportOptions) -> Result<ResearchActivity, CrosswalkError> {
//!     metadata
//!         .to_research_activity(options)
//!         .map(|(activity, _warnings)| activity)
//! }
//! ```
mod datacite;
mod output;

use crate::namespaces::DEFAULT_ROR_SCHEMA_URI;
use crate::pid::raid::{
    self, AccessType, CreditRole, DescriptionType, ObjectCategoryType, ObjectType, OrganizationRoleType, PositionType, RelatedRaidType, TitleType,
};
use crate::pid::{Identifier, PersistentIdentifierParse, PID, RAID};
use crate::research_activity::output::Funding;
use crate::research_activity::{
    ActivityContributor, ActivityDateType, ActivityOrganization, AgentIdentifier, ControlledSubject, DatedAgentRole, DatedOrganizationRole,
    ResearchActivity, ResearchActivityMetadata, RightsAccess, StructuredRights, TypedActivityDate, TypedIdentifier, TypedRelatedResource,
};
use crate::standard::crosswalk::CrosswalkSchema::{Rad as RAD, Raid as RAID_STANDARD};
use crate::standard::crosswalk::{Conversion, ConversionWarning, Crosswalk, CrosswalkError};
use crate::util::constants::app::{CREDIT_SCHEMA_URI, ISNI_SCHEMA_URI, ORCID_SCHEMA_URI, RAID_SCHEMA_URI};
use crate::validation::{rules, Validate};
use crate::{ContactPoint, Date, OneOrMany, Website};
use acorn_core::prelude::alloc::{format, vec, String, ToString, Vec};
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::{to_rfc3339, StringConversion};
use alloc::collections::BTreeMap;
use jiff::{fmt::temporal::DateTimeParser, Timestamp};

static DATE_TIME_PARSER: DateTimeParser = DateTimeParser::new();
type ExportedRelated = Conversion<(Option<Vec<raid::RelatedRaid>>, Option<Vec<raid::RelatedObject>>)>;
trait Export {
    fn export_access(&self, options: &RaidExportOptions) -> Result<Conversion<raid::Access>, CrosswalkError>;
    fn export_contributors(&self, options: &RaidExportOptions, activity_date: &Date) -> Result<Conversion<Vec<raid::Contributor>>, CrosswalkError>;
    fn export_date(&self, options: &RaidExportOptions) -> Result<Conversion<Date>, CrosswalkError>;
    fn export_identifier(
        &self,
        options: &RaidExportOptions,
        organizations: Option<&[raid::Organization]>,
    ) -> Result<Conversion<Option<raid::MetadataIdentifier>>, CrosswalkError>;
    fn export_local_warnings(&self, options: &RaidExportOptions) -> Vec<ConversionWarning>;
    fn export_organizations(&self, options: &RaidExportOptions) -> Result<Conversion<Option<Vec<raid::Organization>>>, CrosswalkError>;
    fn export_related(&self, options: &RaidExportOptions) -> ExportedRelated;
    fn export_subjects(&self) -> Option<Vec<raid::Subject>>;
    fn export_titles(&self, date: &Date) -> Vec<raid::Title>;
}
trait Import {
    fn import_contact(&self, options: &RaidImportOptions) -> Result<Conversion<(ContactPoint, Option<String>)>, CrosswalkError>;
    fn import_contributors(
        &self,
        options: &RaidImportOptions,
        contact_identifier: Option<&str>,
        contact: &ContactPoint,
    ) -> Conversion<Vec<ActivityContributor>>;
    fn import_dates(&self) -> Conversion<Vec<TypedActivityDate>>;
    fn import_handles(&self) -> Option<Vec<String>>;
    fn import_local_identifier(&self, options: &RaidImportOptions) -> Result<String, CrosswalkError>;
    fn import_local_warnings(&self) -> Vec<ConversionWarning>;
    fn import_organizations(&self, options: &RaidImportOptions) -> Conversion<(Vec<ActivityOrganization>, Vec<Funding>, Vec<String>)>;
    fn import_related(&self) -> Conversion<Vec<TypedRelatedResource>>;
    fn import_subjects(&self) -> (Vec<ControlledSubject>, Vec<String>);
    fn import_title(&self) -> Result<(String, Option<String>), CrosswalkError>;
}
/// Options for converting research activity data to RAiD metadata.
#[derive(Clone, Debug, Default)]
pub struct RaidExportOptions {
    /// Permit legacy defaults for missing RAiD assertions.
    pub allow_infer: bool,
    /// Explicit ISO 8601 date used by opt-in inference instead of reading a system clock.
    pub inference_date: Option<String>,
    /// Complete registry-owned identifier metadata for an existing RAiD.
    pub identifier: Option<raid::MetadataIdentifier>,
}
/// Options for converting RAiD metadata to research activity data.
#[derive(Clone, Debug, Default)]
pub struct RaidImportOptions {
    /// Required local contact data that RAiD does not model.
    pub contact: Option<ContactPoint>,
    /// Local RAD identifier used when no ACORN alternate identifier is present.
    pub identifier: Option<String>,
    /// Contributor display names keyed by normalized ORCID or ISNI.
    pub contributor_names: BTreeMap<String, String>,
    /// Organization display names keyed by normalized ROR.
    pub organization_names: BTreeMap<String, String>,
}
impl ActivityContributor {
    fn export(&self) -> Conversion<Option<raid::Contributor>> {
        let identifier = self.identifiers.iter().find_map(|identifier| {
            agent_identifier(&identifier.value, identifier.scheme.as_deref()).map(|(id, schema_uri)| (id, schema_uri, identifier))
        });
        match identifier {
            | Some((id, schema_uri, _)) => {
                let positions = self
                    .roles
                    .iter()
                    .filter_map(|role| {
                        role.role
                            .as_deref()
                            .and_then(|role| PositionType::try_from(role).ok())
                            .and_then(|position| {
                                role.start_date.as_ref().map(|start| {
                                    raid::ContributorPosition::init()
                                        .id(position)
                                        .date(Date {
                                            start_date: to_date(start),
                                            end_date: role.end_date.as_deref().and_then(to_date),
                                        })
                                        .build()
                                })
                            })
                    })
                    .collect::<Vec<_>>();
                let roles = self
                    .roles
                    .iter()
                    .filter_map(|role| role.role.as_deref().and_then(|role| CreditRole::try_from(role).ok()))
                    .map(|id| raid::Role::init().id(id).schema_uri(CREDIT_SCHEMA_URI).build())
                    .collect::<Vec<_>>();
                let mut warnings = vec![ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "contributors[].name")];
                warnings.extend(
                    self.affiliations
                        .iter()
                        .map(|_| ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "contributors[].affiliations[]")),
                );
                warnings.extend(
                    self.roles
                        .iter()
                        .filter(|role| role.role.as_deref().is_some_and(|role| PositionType::try_from(role).is_ok()) && role.start_date.is_none())
                        .map(|_| {
                            ConversionWarning::omitted(
                                RAD,
                                RAID_STANDARD,
                                "contributors[].roles[].startDate",
                                "RAiD administrative positions require a start date",
                            )
                        }),
                );
                warnings.extend(
                    self.roles
                        .iter()
                        .filter(|role| {
                            role.role
                                .as_deref()
                                .is_some_and(|role| PositionType::try_from(role).is_err() && CreditRole::try_from(role).is_err())
                        })
                        .map(|_| {
                            ConversionWarning::omitted(
                                RAD,
                                RAID_STANDARD,
                                "contributors[].roles[].role",
                                "the role is not in the RAiD position or CRediT vocabulary",
                            )
                        }),
                );
                let leader = self.roles.iter().any(|role| role.leader == Some(true));
                let contact = self.roles.iter().any(|role| role.contact == Some(true));
                Conversion::new(
                    Some(raid::Contributor {
                        id: Some(id),
                        schema_uri: Some(schema_uri),
                        status: None,
                        status_message: None,
                        position: positions,
                        leader,
                        contact,
                        email: None,
                        role: (!roles.is_empty()).then_some(roles),
                        uuid: None,
                    }),
                    warnings,
                )
            }
            | None => Conversion::new(
                None,
                vec![ConversionWarning::omitted(
                    RAD,
                    RAID_STANDARD,
                    "contributors[].identifiers",
                    "RAiD contributors require an ORCID or ISNI",
                )],
            ),
        }
    }
}
impl ActivityOrganization {
    fn export(&self) -> Conversion<Option<raid::Organization>> {
        let ror = self
            .identifiers
            .iter()
            .find(|identifier| identifier.scheme.as_deref().is_some_and(|scheme| PID::from(scheme) == PID::ROR));
        match ror {
            | Some(ror) => {
                let roles = self
                    .roles
                    .iter()
                    .filter_map(|role| {
                        role.role.as_deref().map(OrganizationRoleType::from).and_then(|id| {
                            role.start_date.as_ref().map(|start| raid::OrganizationRole {
                                id,
                                schema_uri: role.role_scheme_uri.clone(),
                                date: Date {
                                    start_date: to_date(start),
                                    end_date: role.end_date.as_deref().and_then(to_date),
                                },
                            })
                        })
                    })
                    .collect::<Vec<_>>();
                let warnings = core::iter::once(ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "organizations[].name"))
                    .chain(self.roles.iter().filter(|role| role.start_date.is_none()).map(|_| {
                        ConversionWarning::omitted(
                            RAD,
                            RAID_STANDARD,
                            "organizations[].roles[].startDate",
                            "RAiD organization roles require a start date",
                        )
                    }))
                    .collect();
                Conversion::new(
                    Some(raid::Organization {
                        id: ror.value.clone(),
                        schema_uri: Some(ror.scheme_uri.clone().unwrap_or_else(|| DEFAULT_ROR_SCHEMA_URI.to_string())),
                        role: roles,
                    }),
                    warnings,
                )
            }
            | None => Conversion::new(
                None,
                vec![ConversionWarning::omitted(
                    RAD,
                    RAID_STANDARD,
                    "organizations[].identifiers",
                    "RAiD organizations require a ROR identifier",
                )],
            ),
        }
    }
}
impl TryFrom<&ResearchActivity> for Conversion<Option<raid::MetadataMetadata>> {
    type Error = CrosswalkError;
    fn try_from(value: &ResearchActivity) -> Result<Self, Self::Error> {
        let find = |kind| {
            value
                .meta
                .typed_dates
                .as_ref()
                .into_iter()
                .flat_map(|values| values.iter())
                .find(|date| core::mem::discriminant(&date.date_type) == core::mem::discriminant(&kind))
                .map(|date| {
                    date.date
                        .parse::<Timestamp>()
                        .map_err(|error| CrosswalkError::TransformationFailed {
                            field: "typedDates".to_string(),
                            reason: error.to_string(),
                        })
                        .and_then(|timestamp| {
                            usize::try_from(timestamp.as_second()).map_err(|error| CrosswalkError::TransformationFailed {
                                field: "typedDates".to_string(),
                                reason: error.to_string(),
                            })
                        })
                })
                .transpose()
        };
        find(ActivityDateType::Created).and_then(|created| {
            find(ActivityDateType::Updated).and_then(|updated| match (created, updated) {
                | (Some(created), Some(updated)) => Ok(Self::new(Some(raid::MetadataMetadata { created, updated }), Vec::new())),
                | (None, None) => Ok(Self::new(None, Vec::new())),
                | _ => Err(CrosswalkError::MissingRequiredField(
                    "typedDates[Created, Updated] must be supplied together".to_string(),
                )),
            })
        })
    }
}
impl From<&ResearchActivity> for Conversion<Option<Vec<raid::AlternateIdentifier>>> {
    fn from(value: &ResearchActivity) -> Self {
        let local = core::iter::once(raid::AlternateIdentifier {
            id: value.meta.identifier.clone(),
            alternate_identifier_type: APPLICATION.to_ascii_uppercase(),
        });
        let handles = value.meta.handle.as_ref().into_iter().flat_map(|values| {
            values.iter().map(|value| raid::AlternateIdentifier {
                id: value.clone(),
                alternate_identifier_type: "Handle".to_string(),
            })
        });
        let identifiers = local.chain(handles).collect::<Vec<_>>();
        Self::new((!identifiers.is_empty()).then_some(identifiers), Vec::new())
    }
}
impl raid::Contributor {
    fn import_roles(&self, identifier: &AgentIdentifier) -> Vec<DatedAgentRole> {
        let positions = self.position.iter().map(|position| DatedAgentRole {
            agent: identifier.clone(),
            role: Some(position.id.to_string()),
            role_scheme_uri: Some(position.schema_uri.clone()),
            start_date: position.date.start_date.as_deref().map(append_midnight_utc_suffix),
            end_date: position
                .date
                .end_date
                .as_deref()
                .and_then(StringConversion::to_complete_date)
                .map(append_midnight_utc_suffix),
            contact: Some(self.contact),
            leader: Some(self.leader),
        });
        let roles = self.role.as_deref().unwrap_or_default().iter().map(|role| DatedAgentRole {
            agent: identifier.clone(),
            role: role.id.as_ref().map(ToString::to_string),
            role_scheme_uri: Some(role.schema_uri.clone()),
            start_date: None,
            end_date: None,
            contact: Some(self.contact),
            leader: Some(self.leader),
        });
        let values = positions.chain(roles).collect::<Vec<_>>();
        match values.is_empty() {
            | true => vec![DatedAgentRole {
                agent: identifier.clone(),
                role: None,
                role_scheme_uri: None,
                start_date: None,
                end_date: None,
                contact: Some(self.contact),
                leader: Some(self.leader),
            }],
            | false => values,
        }
    }
}
impl Crosswalk<ResearchActivity> for raid::Metadata {
    fn crosswalk(&self) -> Result<(ResearchActivity, Vec<ConversionWarning>), CrosswalkError> {
        self.to_research_activity(&RaidImportOptions::default())
    }
}
impl Import for raid::Metadata {
    fn import_contact(&self, options: &RaidImportOptions) -> Result<Conversion<(ContactPoint, Option<String>)>, CrosswalkError> {
        let contacts = self
            .contributor
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|contributor| contributor.contact)
            .collect::<Vec<_>>();
        let contacts_valid = match contacts.len() > 1 {
            | true => Err(CrosswalkError::TransformationFailed {
                field: "contributor[].contact".to_string(),
                reason: "multiple designated contacts cannot populate one RAD contact".to_string(),
            }),
            | false => Ok(()),
        };
        contacts_valid
            .and_then(|()| {
                options
                    .contact
                    .clone()
                    .ok_or_else(|| CrosswalkError::MissingRequiredField("options.contact".to_string()))
            })
            .and_then(|mut contact| {
                let raid_contact = contacts.first().copied();
                let raw_identifier = raid_contact.and_then(|value| value.id.clone());
                let identifier = raid_contact.and_then(|value| {
                    value
                        .id
                        .as_deref()
                        .zip(value.schema_uri.as_deref())
                        .and_then(|(identifier, scheme)| agent_identifier(identifier, Some(scheme)))
                        .map(|(identifier, _)| identifier)
                });
                let email = raid_contact.and_then(|value| value.email.clone());
                let identifier_matches = contact
                    .identifier
                    .as_ref()
                    .zip(identifier.as_ref())
                    .is_none_or(|(left, right)| Identifier::new(left).normalized() == Identifier::new(right).normalized());
                let email_matches = !email
                    .as_ref()
                    .is_some_and(|value| !contact.email.trim().is_empty() && !contact.email.trim().eq_ignore_ascii_case(value.trim()));
                match (identifier_matches, email_matches) {
                    | (false, _) => Err(CrosswalkError::TransformationFailed {
                        field: "options.contact.identifier".to_string(),
                        reason: "contact seed conflicts with the designated RAiD contact".to_string(),
                    }),
                    | (_, false) => Err(CrosswalkError::TransformationFailed {
                        field: "options.contact.email".to_string(),
                        reason: "contact seed conflicts with the designated RAiD contact".to_string(),
                    }),
                    | (true, true) => {
                        if contact.identifier.is_none() {
                            contact.identifier.clone_from(&identifier);
                        }
                        if let Some(email) = email {
                            contact.email = email;
                        }
                        let warnings = (raw_identifier.is_some() && identifier.is_none())
                            .then(|| {
                                ConversionWarning::omitted(
                                    RAID_STANDARD,
                                    RAD,
                                    "contributor[contact].id",
                                    "the designated contact identifier is not a valid canonical ORCID or ISNI",
                                )
                            })
                            .into_iter()
                            .collect();
                        Ok(Conversion::new((contact, identifier), warnings))
                    }
                }
            })
    }
    fn import_contributors(
        &self,
        options: &RaidImportOptions,
        contact_identifier: Option<&str>,
        contact: &ContactPoint,
    ) -> Conversion<Vec<ActivityContributor>> {
        let converted = self
            .contributor
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|contributor| {
                let identifier = contributor
                    .id
                    .as_ref()
                    .zip(contributor.schema_uri.as_ref())
                    .and_then(|(value, scheme_uri)| agent_identifier(value, Some(scheme_uri)))
                    .map(|(value, scheme_uri)| AgentIdentifier {
                        scheme: Some(if scheme_uri.contains("orcid") { "ORCID" } else { "ISNI" }.to_string()),
                        value,
                        scheme_uri: Some(scheme_uri),
                    });
                let name = contributor.id.as_ref().and_then(|id| {
                    if contact_identifier.is_some_and(|contact_id| Identifier::new(contact_id).normalized() == Identifier::new(id).normalized()) {
                        Some(format!("{} {}", contact.given_name, contact.family_name))
                    } else {
                        Identifier::new(id).normalized().and_then(|identifier| {
                            options
                                .contributor_names
                                .iter()
                                .find(|(key, _)| Identifier::new(key.as_str()).normalized().as_ref() == Some(&identifier))
                                .map(|(_, name)| name.clone())
                        })
                    }
                });
                match (identifier, name) {
                    | (Some(identifier), Some(name)) => {
                        let roles = contributor.import_roles(&identifier);
                        let warnings = contributor
                            .position
                            .iter()
                            .flat_map(|position| {
                                [
                                    position.date.start_date.as_ref().map(|_| {
                                        ConversionWarning::approximated(
                                            RAID_STANDARD,
                                            RAD,
                                            "contributor[].position[].startDate",
                                            "day precision was expanded to midnight UTC",
                                        )
                                    }),
                                    position.date.end_date.as_ref().map(|_| {
                                        ConversionWarning::approximated(
                                            RAID_STANDARD,
                                            RAD,
                                            "contributor[].position[].endDate",
                                            "day precision was expanded to midnight UTC",
                                        )
                                    }),
                                ]
                                .into_iter()
                                .flatten()
                            })
                            .collect();
                        (
                            Some(ActivityContributor::init().name(name).identifiers(vec![identifier]).roles(roles).build()),
                            warnings,
                        )
                    }
                    | _ => (
                        None,
                        vec![ConversionWarning::omitted(
                            RAID_STANDARD,
                            RAD,
                            "contributor[]",
                            "RAD structured contributors require a caller-supplied display name",
                        )],
                    ),
                }
            })
            .collect::<Vec<_>>();
        Conversion::new(
            converted.iter().filter_map(|(value, _)| value.clone()).collect(),
            converted.into_iter().flat_map(|(_, warnings)| warnings).collect(),
        )
    }
    fn import_dates(&self) -> Conversion<Vec<TypedActivityDate>> {
        let activity_dates = self.date.as_ref().into_iter().flat_map(|date| {
            [
                date.start_date.as_deref().map(append_midnight_utc_suffix).map(|date| TypedActivityDate {
                    date_type: ActivityDateType::StartDate,
                    date,
                    information: None,
                }),
                date.end_date
                    .as_deref()
                    .and_then(StringConversion::to_complete_date)
                    .map(append_midnight_utc_suffix)
                    .map(|date| TypedActivityDate {
                        date_type: ActivityDateType::EndDate,
                        date,
                        information: None,
                    }),
            ]
            .into_iter()
            .flatten()
        });
        let record_dates = self.metadata.as_ref().into_iter().flat_map(|value| {
            [
                i64::try_from(value.created)
                    .ok()
                    .and_then(|value| Timestamp::from_second(value).ok())
                    .map(to_rfc3339)
                    .map(|date| TypedActivityDate {
                        date_type: ActivityDateType::Created,
                        date,
                        information: None,
                    }),
                i64::try_from(value.updated)
                    .ok()
                    .and_then(|value| Timestamp::from_second(value).ok())
                    .map(to_rfc3339)
                    .map(|date| TypedActivityDate {
                        date_type: ActivityDateType::Updated,
                        date,
                        information: None,
                    }),
            ]
            .into_iter()
            .flatten()
        });
        let warnings = self.date.as_ref().into_iter().flat_map(|date| {
            [
                date.start_date
                    .as_ref()
                    .map(|_| ConversionWarning::approximated(RAID_STANDARD, RAD, "date.startDate", "day precision was expanded to midnight UTC")),
                date.end_date.as_ref().map(|value| match value.to_complete_date() {
                    | Some(_) => ConversionWarning::approximated(RAID_STANDARD, RAD, "date.endDate", "day precision was expanded to midnight UTC"),
                    | None => ConversionWarning::omitted(
                        RAID_STANDARD,
                        RAD,
                        "date.endDate",
                        "year-only dates cannot be expanded without inventing month and day",
                    ),
                }),
            ]
            .into_iter()
            .flatten()
        });
        Conversion::new(activity_dates.chain(record_dates).collect(), warnings.collect())
    }
    fn import_handles(&self) -> Option<Vec<String>> {
        let handles = self
            .alternate_identifier
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|value| value.alternate_identifier_type.eq_ignore_ascii_case("handle"))
            .map(|value| value.id.clone())
            .collect::<Vec<_>>();
        (!handles.is_empty()).then_some(handles)
    }
    fn import_local_identifier(&self, options: &RaidImportOptions) -> Result<String, CrosswalkError> {
        let sources = self
            .alternate_identifier
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|value| value.alternate_identifier_type.eq_ignore_ascii_case(APPLICATION))
            .map(|value| value.id.clone())
            .collect::<Vec<_>>();
        match sources.len() > 1 {
            | true => Err(CrosswalkError::TransformationFailed {
                field: "alternateIdentifier[ACORN]".to_string(),
                reason: "multiple local identifiers cannot populate one RAD identifier".to_string(),
            }),
            | false => match (sources.first().cloned(), options.identifier.clone()) {
                | (Some(source), Some(supplied)) if source != supplied => Err(CrosswalkError::TransformationFailed {
                    field: "options.identifier".to_string(),
                    reason: "local identifier conflicts with the RAiD ACORN alternate identifier".to_string(),
                }),
                | (Some(source), _) => Ok(source),
                | (None, Some(supplied)) => Ok(supplied),
                | (None, None) => Err(CrosswalkError::MissingRequiredField("options.identifier".to_string())),
            },
        }
    }
    fn import_local_warnings(&self) -> Vec<ConversionWarning> {
        let identifier_warnings = self.identifier.as_ref().into_iter().flat_map(|identifier| {
            [
                identifier.owner.as_ref().map(|_| "identifier.owner"),
                identifier.raid_agency_url.as_ref().map(|_| "identifier.raidAgencyUrl"),
                identifier.registration_agency.as_ref().map(|_| "identifier.registrationAgency"),
                identifier.license.as_ref().map(|_| "identifier.license"),
                identifier.version.map(|_| "identifier.version"),
            ]
            .into_iter()
            .flatten()
            .map(|field| ConversionWarning::no_equivalent(RAID_STANDARD, RAD, field))
        });
        let alternate_identifier_warnings = self
            .alternate_identifier
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|identifier| {
                !identifier.alternate_identifier_type.eq_ignore_ascii_case(APPLICATION)
                    && !identifier.alternate_identifier_type.eq_ignore_ascii_case("handle")
            })
            .map(|_| ConversionWarning::no_equivalent(RAID_STANDARD, RAD, "alternateIdentifier[]"));
        let access_warnings = self.access.as_ref().into_iter().flat_map(|access| {
            [
                access.embargo_expiry.as_ref().map(|_| "access.embargoExpiry"),
                access
                    .statement
                    .as_ref()
                    .and_then(|statement| statement.language.as_ref())
                    .map(|_| "access.statement.language"),
            ]
            .into_iter()
            .flatten()
            .map(|field| ConversionWarning::no_equivalent(RAID_STANDARD, RAD, field))
        });
        let contributor_warnings = self.contributor.as_deref().unwrap_or_default().iter().flat_map(|contributor| {
            [
                contributor.status.as_ref().map(|_| "contributor[].status"),
                contributor.status_message.as_ref().map(|_| "contributor[].statusMessage"),
                contributor.uuid.as_ref().map(|_| "contributor[].uuid"),
            ]
            .into_iter()
            .flatten()
            .map(|field| ConversionWarning::no_equivalent(RAID_STANDARD, RAD, field))
        });
        let title_warnings = self.title.as_deref().unwrap_or_default().iter().flat_map(|title| {
            [
                title.language.as_ref().map(|_| "title[].language"),
                title.start_date.as_ref().map(|_| "title[].startDate"),
                title.end_date.as_ref().map(|_| "title[].endDate"),
            ]
            .into_iter()
            .flatten()
            .map(|field| ConversionWarning::no_equivalent(RAID_STANDARD, RAD, field))
        });
        let additional_title_warnings = self
            .title
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|title| title.title_type.as_ref().is_some_and(|kind| matches!(kind.id, TitleType::Alternative)))
            .skip(1)
            .map(|_| ConversionWarning::omitted(RAID_STANDARD, RAD, "title[Alternative]", "RAD stores one subtitle"));
        let subject_warnings = self.subject.as_deref().unwrap_or_default().iter().flat_map(|subject| {
            subject
                .keyword
                .as_deref()
                .unwrap_or_default()
                .iter()
                .filter(|keyword| keyword.language.is_some())
                .map(|_| ConversionWarning::no_equivalent(RAID_STANDARD, RAD, "subject[].keyword[].language"))
        });
        identifier_warnings
            .chain(access_warnings)
            .chain(additional_title_warnings)
            .chain(alternate_identifier_warnings)
            .chain(contributor_warnings)
            .chain(subject_warnings)
            .chain(title_warnings)
            .chain(self.traditional_knowledge_label.as_ref().into_iter().flat_map(|values| {
                values
                    .iter()
                    .enumerate()
                    .map(|(index, _)| ConversionWarning::no_equivalent(RAID_STANDARD, RAD, format!("traditionalKnowledgeLabel[{index}]")))
            }))
            .collect()
    }
    fn import_organizations(&self, options: &RaidImportOptions) -> Conversion<(Vec<ActivityOrganization>, Vec<Funding>, Vec<String>)> {
        let converted = self.organization.as_deref().unwrap_or_default().iter().map(|organization| {
            let name = Identifier::new(&organization.id).normalized().and_then(|identifier| {
                options
                    .organization_names
                    .iter()
                    .find(|(key, _)| Identifier::new(key.as_str()).normalized().as_ref() == Some(&identifier))
                    .map(|(_, name)| name.clone())
            });
            let identifier = TypedIdentifier {
                value: organization.id.clone(),
                scheme: Some("ROR".to_string()),
                scheme_uri: organization.schema_uri.clone().or_else(|| Some(DEFAULT_ROR_SCHEMA_URI.to_string())),
            };
            let roles = organization
                .role
                .iter()
                .map(|role| DatedOrganizationRole {
                    organization: identifier.clone(),
                    role: Some(role.id.to_string()),
                    role_scheme_uri: role.schema_uri.clone(),
                    start_date: role.date.start_date.as_deref().map(append_midnight_utc_suffix),
                    end_date: role
                        .date
                        .end_date
                        .as_deref()
                        .and_then(StringConversion::to_complete_date)
                        .map(append_midnight_utc_suffix),
                })
                .collect::<Vec<_>>();
            let is_funder = organization.role.iter().any(|role| matches!(role.id, OrganizationRoleType::Funder));
            match name {
                | Some(name) => {
                    let warnings = organization
                        .role
                        .iter()
                        .flat_map(|role| {
                            [
                                role.date.start_date.as_ref().map(|_| {
                                    ConversionWarning::approximated(
                                        RAID_STANDARD,
                                        RAD,
                                        "organization[].role[].startDate",
                                        "day precision was expanded to midnight UTC",
                                    )
                                }),
                                role.date.end_date.as_ref().map(|_| {
                                    ConversionWarning::approximated(
                                        RAID_STANDARD,
                                        RAD,
                                        "organization[].role[].endDate",
                                        "day precision was expanded to midnight UTC",
                                    )
                                }),
                            ]
                            .into_iter()
                            .flatten()
                        })
                        .collect();
                    (
                        Some(
                            ActivityOrganization::init()
                                .name(name.clone())
                                .identifiers(vec![identifier])
                                .roles(roles)
                                .build(),
                        ),
                        is_funder.then(|| Funding::init().name(name).ror(organization.id.clone()).build()),
                        None,
                        warnings,
                    )
                }
                | None => (
                    None,
                    None,
                    Some(organization.id.clone()),
                    vec![ConversionWarning::omitted(
                        RAID_STANDARD,
                        RAD,
                        "organization[]",
                        "RAD structured organizations require a caller-supplied display name; the ROR was retained in meta.ror",
                    )],
                ),
            }
        });
        let (organizations, funders, rors, warnings) = converted.fold(
            (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            |(mut organizations, mut funders, mut rors, mut warnings), (organization, funder, ror, item_warnings)| {
                organizations.extend(organization);
                funders.extend(funder);
                rors.extend(ror);
                warnings.extend(item_warnings);
                (organizations, funders, rors, warnings)
            },
        );
        Conversion::new((organizations, funders, rors), warnings)
    }
    fn import_related(&self) -> Conversion<Vec<TypedRelatedResource>> {
        let raids = self
            .related_raid
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|related| TypedRelatedResource {
                identifier: TypedIdentifier {
                    value: RAID::format(&related.id),
                    scheme: Some("RAiD".to_string()),
                    scheme_uri: Some(RAID_SCHEMA_URI.to_string()),
                },
                relation_type: related.related_raid_type.id.to_string(),
                relation_type_information: None,
                resource_type_general: Some("Project".to_string()),
                category: None,
            });
        let objects = self
            .related_object
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|related| TypedRelatedResource {
                identifier: TypedIdentifier {
                    value: related.id.clone(),
                    scheme: None,
                    scheme_uri: related.schema_uri.clone(),
                },
                relation_type: "Other".to_string(),
                relation_type_information: Some("RAiD related object".to_string()),
                resource_type_general: Some(related.related_object_type.id.to_string()),
                category: None,
            });
        let warnings = self.related_object.as_deref().unwrap_or_default().iter().map(|_| {
            ConversionWarning::approximated(
                RAID_STANDARD,
                RAD,
                "relatedObject[].category",
                "RAiD object categories have no equivalent RAD relation category",
            )
        });
        Conversion::new(raids.chain(objects).collect(), warnings.collect())
    }
    fn import_subjects(&self) -> (Vec<ControlledSubject>, Vec<String>) {
        self.subject
            .as_deref()
            .unwrap_or_default()
            .iter()
            .flat_map(|subject| {
                subject.keyword.as_deref().unwrap_or_default().iter().map(|keyword| {
                    if subject.schema_uri.is_some() || subject.id != keyword.text {
                        (
                            Some(ControlledSubject {
                                subject: keyword.text.clone(),
                                subject_scheme: None,
                                scheme_uri: subject.schema_uri.clone(),
                                value_uri: subject.id.starts_with("http").then(|| subject.id.clone()),
                            }),
                            None,
                        )
                    } else {
                        (None, Some(keyword.text.clone()))
                    }
                })
            })
            .fold((Vec::new(), Vec::new()), |(mut subjects, mut keywords), (subject, keyword)| {
                subjects.extend(subject);
                keywords.extend(keyword);
                (subjects, keywords)
            })
    }
    fn import_title(&self) -> Result<(String, Option<String>), CrosswalkError> {
        let titles = self.title.as_deref().unwrap_or_default();
        let primary = titles
            .iter()
            .filter(|title| title.title_type.as_ref().is_some_and(|kind| matches!(kind.id, TitleType::Primary)))
            .collect::<Vec<_>>();
        match primary.as_slice() {
            | [title] => title
                .text
                .clone()
                .ok_or_else(|| CrosswalkError::MissingRequiredField("title[Primary].text".to_string()))
                .map(|title| {
                    let subtitle = titles
                        .iter()
                        .find(|candidate| {
                            candidate
                                .title_type
                                .as_ref()
                                .is_some_and(|kind| matches!(kind.id, TitleType::Alternative))
                        })
                        .and_then(|candidate| candidate.text.clone());
                    (title, subtitle)
                }),
            | [] => Err(CrosswalkError::MissingRequiredField("title[Primary]".to_string())),
            | _ => Err(CrosswalkError::TransformationFailed {
                field: "title".to_string(),
                reason: "multiple primary titles cannot populate one RAD title".to_string(),
            }),
        }
    }
}
impl raid::Metadata {
    /// Convert this RAiD record to research activity data using caller-supplied local identity context.
    pub fn to_research_activity(&self, options: &RaidImportOptions) -> Result<(ResearchActivity, Vec<ConversionWarning>), CrosswalkError> {
        self.import_title().and_then(|(title, subtitle)| {
            self.import_local_identifier(options).and_then(|identifier| {
                self.import_contact(options).and_then(|contact_conversion| {
                    let ((contact, contact_identifier), contact_warnings) = contact_conversion.into_parts();
                    let (contributors, contributor_warnings) =
                        self.import_contributors(options, contact_identifier.as_deref(), &contact).into_parts();
                    let ((organizations, funders, legacy_rors), organization_warnings) = self.import_organizations(options).into_parts();
                    let (related_resources, related_warnings) = self.import_related().into_parts();
                    let (typed_dates, date_warnings) = self.import_dates().into_parts();
                    let (subjects, keywords) = self.import_subjects();
                    let websites = self.alternate_url.as_ref().map(|values| {
                        values
                            .iter()
                            .map(|value| Website {
                                url: value.url().to_string(),
                                description: String::new(),
                            })
                            .collect::<Vec<_>>()
                    });
                    let access = self.access.as_ref().map(StructuredRights::from);
                    let raid = self
                        .identifier
                        .as_ref()
                        .and_then(|value| value.id.as_ref())
                        .map(|value| OneOrMany::One(RAID::format(value)));
                    let mut warnings = contact_warnings
                        .into_iter()
                        .chain(contributor_warnings)
                        .chain(organization_warnings)
                        .chain(related_warnings)
                        .chain(date_warnings)
                        .chain(self.import_local_warnings())
                        .collect::<Vec<_>>();
                    warnings.extend(self.description.as_ref().into_iter().flat_map(|descriptions| {
                        descriptions
                            .iter()
                            .enumerate()
                            .map(|(index, _)| ConversionWarning::no_equivalent(RAID_STANDARD, RAD, format!("description[{index}]")))
                    }));
                    let meta = ResearchActivityMetadata {
                        identifier,
                        handle: self.import_handles().map(OneOrMany::Many),
                        raid,
                        ror: (!legacy_rors.is_empty()).then_some(legacy_rors).map(OneOrMany::Many),
                        contributors: (!contributors.is_empty()).then_some(contributors).map(OneOrMany::Many),
                        organizations: (!organizations.is_empty()).then_some(organizations).map(OneOrMany::Many),
                        funders: (!funders.is_empty()).then_some(funders).map(OneOrMany::Many),
                        typed_dates: (!typed_dates.is_empty()).then_some(typed_dates).map(OneOrMany::Many),
                        related_resources: (!related_resources.is_empty()).then_some(related_resources).map(OneOrMany::Many),
                        websites: websites.map(OneOrMany::Many),
                        keywords,
                        subjects: (!subjects.is_empty()).then_some(subjects).map(OneOrMany::Many),
                        spatial_coverage: self.spatial_coverage.as_ref().map(|values| OneOrMany::Many(values.clone())),
                        access,
                        ..ResearchActivityMetadata::default()
                    };
                    let activity = ResearchActivity {
                        meta,
                        title,
                        subtitle,
                        contact,
                        ..ResearchActivity::default()
                    };
                    activity
                        .validate()
                        .map(|_| (activity, warnings))
                        .map_err(|error| CrosswalkError::BuildFailed(error.to_string()))
                })
            })
        })
    }
}
impl RaidExportOptions {
    fn inference_date(&self) -> Result<String, CrosswalkError> {
        self.inference_date
            .as_deref()
            .filter(|date| rules::date(date).is_ok())
            .map(ToString::to_string)
            .ok_or_else(|| CrosswalkError::MissingRequiredField("options.inferenceDate".to_string()))
    }
}
impl Crosswalk<raid::Metadata> for ResearchActivity {
    fn crosswalk(&self) -> Result<(raid::Metadata, Vec<ConversionWarning>), CrosswalkError> {
        self.to_raid(&RaidExportOptions::default())
    }
}
impl Export for ResearchActivity {
    fn export_access(&self, options: &RaidExportOptions) -> Result<Conversion<raid::Access>, CrosswalkError> {
        let authored = self.meta.access.as_ref().or_else(|| {
            self.meta
                .rights
                .as_ref()
                .and_then(|rights| rights.iter().find(|right| right.access.is_some()))
        });
        match authored {
            | Some(rights) => match &rights.access {
                | Some(access) => Ok(Conversion::new(
                    raid::Access {
                        access_type: Some(
                            raid::AccessIdentifier::init()
                                .id(match access {
                                    | RightsAccess::Open => AccessType::OpenAccess,
                                    | RightsAccess::Embargoed => AccessType::EmbargoedAccess,
                                })
                                .build(),
                        ),
                        embargo_expiry: None,
                        statement: rights.rights.as_ref().map(|text| raid::AccessStatement {
                            text: Some(text.clone()),
                            language: None,
                        }),
                    },
                    Vec::new(),
                )),
                | None => Err(CrosswalkError::MissingRequiredField("meta.access.access".to_string())),
            },
            | None if options.allow_infer => Ok(Conversion::new(
                raid::Access {
                    access_type: Some(raid::AccessIdentifier::init().id(AccessType::OpenAccess).build()),
                    embargo_expiry: None,
                    statement: None,
                },
                vec![ConversionWarning::approximated(
                    RAD,
                    RAID_STANDARD,
                    "access",
                    "open access was inferred by explicit request",
                )],
            )),
            | None => Err(CrosswalkError::MissingRequiredField("meta.access".to_string())),
        }
    }
    fn export_contributors(&self, options: &RaidExportOptions, activity_date: &Date) -> Result<Conversion<Vec<raid::Contributor>>, CrosswalkError> {
        let structured = self
            .meta
            .contributors
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter())
            .map(ActivityContributor::export)
            .collect::<Vec<_>>();
        let mut warnings = structured
            .iter()
            .flat_map(|conversion| conversion.warnings.iter().cloned())
            .collect::<Vec<_>>();
        let mut contributors = structured.into_iter().filter_map(|conversion| conversion.value).collect::<Vec<_>>();
        let contact_identifier = self.contact.identifier.as_ref().and_then(|value| agent_identifier(value, None));
        match contact_identifier {
            | Some((id, schema_uri)) => {
                let key = Identifier::new(&id).normalized();
                match contributors
                    .iter_mut()
                    .find(|contributor| contributor.id.as_deref().and_then(|id| Identifier::new(id).normalized()) == key)
                {
                    | Some(contributor) => {
                        contributor.contact = true;
                        contributor.email = Some(self.contact.email.clone());
                    }
                    | None => contributors.push(raid::Contributor {
                        id: Some(id),
                        schema_uri: Some(schema_uri),
                        status: None,
                        status_message: None,
                        position: Vec::new(),
                        leader: false,
                        contact: true,
                        email: Some(self.contact.email.clone()),
                        role: None,
                        uuid: None,
                    }),
                }
            }
            | None => warnings.push(ConversionWarning::omitted(
                RAD,
                RAID_STANDARD,
                "contact.identifier",
                "RAiD contributors require an ORCID or ISNI",
            )),
        }
        let contact_count = contributors.iter().filter(|contributor| contributor.contact).count();
        let leader_count = contributors.iter().filter(|contributor| contributor.leader).count();
        let validation = match () {
            | () if contributors.is_empty() => Err(CrosswalkError::MissingRequiredField("contributors[].identifier".to_string())),
            | () if contact_count == 0 => Err(CrosswalkError::MissingRequiredField("contributors[].contact".to_string())),
            | () if contact_count > 1 => Err(CrosswalkError::TransformationFailed {
                field: "contributors[].contact".to_string(),
                reason: "RAiD permits one designated contact".to_string(),
            }),
            | () if leader_count > 1 => Err(CrosswalkError::TransformationFailed {
                field: "contributors[].leader".to_string(),
                reason: "RAiD permits one designated leader".to_string(),
            }),
            | () if leader_count == 0 => match (options.allow_infer, options.inference_date()) {
                | (true, Ok(date)) => {
                    contributors.first_mut().into_iter().for_each(|contributor| {
                        contributor.leader = true;
                        contributor.position.push(
                            raid::ContributorPosition::init()
                                .id(PositionType::PrincipalInvestigator)
                                .date(Date {
                                    start_date: Some(date.clone()),
                                    end_date: activity_date.end_date.clone(),
                                })
                                .build(),
                        );
                    });
                    warnings.extend([
                        ConversionWarning::approximated(RAD, RAID_STANDARD, "contributor[0].leader", "leader was inferred by explicit request"),
                        ConversionWarning::approximated(
                            RAD,
                            RAID_STANDARD,
                            "contributor[0].position",
                            "principal investigator was inferred by explicit request",
                        ),
                    ]);
                    Ok(())
                }
                | (true, Err(error)) => Err(error),
                | (false, _) => Err(CrosswalkError::MissingRequiredField("contributors[].leader".to_string())),
            },
            | () => Ok(()),
        };
        validation.map(|()| Conversion::new(contributors, warnings))
    }
    fn export_date(&self, options: &RaidExportOptions) -> Result<Conversion<Date>, CrosswalkError> {
        let dates = self.meta.typed_dates.as_ref();
        let start_dates = dates
            .into_iter()
            .flat_map(|values| values.iter())
            .filter(|date| matches!(date.date_type, ActivityDateType::StartDate))
            .collect::<Vec<_>>();
        let end_dates = dates
            .into_iter()
            .flat_map(|values| values.iter())
            .filter(|date| matches!(date.date_type, ActivityDateType::EndDate))
            .collect::<Vec<_>>();
        let start = start_dates.first().and_then(|date| to_date(&date.date));
        let start = match start {
            | Some(start) => Ok((
                start,
                vec![ConversionWarning::simplified(
                    RAD,
                    RAID_STANDARD,
                    "typedDates[StartDate]",
                    "RAiD stores activity dates without time-of-day or offset precision",
                )],
            )),
            | None if options.allow_infer => options.inference_date().map(|date| {
                (
                    date,
                    vec![ConversionWarning::approximated(
                        RAD,
                        RAID_STANDARD,
                        "date.startDate",
                        "start date was inferred from the caller-supplied inference date",
                    )],
                )
            }),
            | None => Err(CrosswalkError::MissingRequiredField("typedDates[StartDate]".to_string())),
        };
        start.map(|(start_date, mut warnings)| {
            warnings.extend(start_dates.iter().skip(1).map(|_| {
                ConversionWarning::omitted(
                    RAD,
                    RAID_STANDARD,
                    "typedDates[StartDate]",
                    "only the first start date can be represented",
                )
            }));
            warnings.extend(
                end_dates
                    .iter()
                    .skip(1)
                    .map(|_| ConversionWarning::omitted(RAD, RAID_STANDARD, "typedDates[EndDate]", "only the first end date can be represented")),
            );
            let end_date = end_dates.first().and_then(|date| to_date(&date.date));
            if end_date.is_some() {
                warnings.push(ConversionWarning::simplified(
                    RAD,
                    RAID_STANDARD,
                    "typedDates[EndDate]",
                    "RAiD stores activity dates without time-of-day or offset precision",
                ));
            }
            Conversion::new(
                Date {
                    start_date: Some(start_date),
                    end_date,
                },
                warnings,
            )
        })
    }
    fn export_identifier(
        &self,
        options: &RaidExportOptions,
        organizations: Option<&[raid::Organization]>,
    ) -> Result<Conversion<Option<raid::MetadataIdentifier>>, CrosswalkError> {
        let raid_ids = self.meta.raid.as_ref().map(OneOrMany::as_slice).unwrap_or_default();
        match raid_ids.first() {
            | None => Ok(Conversion::new(None, Vec::new())),
            | Some(source_id) => {
                let canonical = RAID::format(source_id);
                let extra_warnings = || {
                    raid_ids
                        .iter()
                        .skip(1)
                        .map(|_| ConversionWarning::omitted(RAD, RAID_STANDARD, "meta.raid[1..]", "RAiD metadata has one primary identifier"))
                        .collect::<Vec<_>>()
                };
                match &options.identifier {
                    | Some(identifier) => {
                        let supplied = identifier.id.as_deref().map(RAID::format).unwrap_or_default();
                        match supplied == canonical {
                            | true => identifier
                                .validate()
                                .map_err(|error| CrosswalkError::BuildFailed(error.to_string()))
                                .map(|_| Conversion::new(Some(identifier.clone()), extra_warnings())),
                            | false => Err(CrosswalkError::TransformationFailed {
                                field: "options.identifier.id".to_string(),
                                reason: "identifier does not match the authored RAD RAiD".to_string(),
                            }),
                        }
                    }
                    | None if options.allow_infer => organizations
                        .unwrap_or_default()
                        .iter()
                        .find(|organization| {
                            organization
                                .role
                                .iter()
                                .any(|role| matches!(role.id, OrganizationRoleType::LeadResearchOrganization))
                        })
                        .map(|organization| organization.id.clone())
                        .ok_or_else(|| CrosswalkError::MissingRequiredField("options.identifier.owner".to_string()))
                        .map(|ror| {
                            let identifier = raid::MetadataIdentifier {
                                id: Some(canonical),
                                schema_uri: Some(RAID_SCHEMA_URI.to_string()),
                                owner: Some(raid::Owner {
                                    id: ror.clone(),
                                    schema_uri: Some(DEFAULT_ROR_SCHEMA_URI.to_string()),
                                    service_point: 0,
                                }),
                                raid_agency_url: Some(RAID_SCHEMA_URI.to_string()),
                                registration_agency: Some(raid::RegistrationAgency {
                                    id: ror,
                                    schema_uri: Some(DEFAULT_ROR_SCHEMA_URI.to_string()),
                                }),
                                license: Some(OneOrMany::One("CC0-1.0".to_string())),
                                version: Some(1),
                            };
                            let warnings = extra_warnings()
                                .into_iter()
                                .chain(
                                    [
                                        "identifier.owner",
                                        "identifier.owner.servicePoint",
                                        "identifier.raidAgencyUrl",
                                        "identifier.registrationAgency",
                                        "identifier.license",
                                        "identifier.version",
                                    ]
                                    .into_iter()
                                    .map(|field| {
                                        ConversionWarning::approximated(
                                            RAD,
                                            RAID_STANDARD,
                                            field,
                                            "legacy identifier metadata was inferred by explicit request",
                                        )
                                    }),
                                )
                                .collect();
                            Conversion::new(Some(identifier), warnings)
                        }),
                    | None => Err(CrosswalkError::MissingRequiredField("options.identifier".to_string())),
                }
            }
        }
    }
    fn export_local_warnings(&self, options: &RaidExportOptions) -> Vec<ConversionWarning> {
        let fields = [
            self.context.as_ref().map(|_| "@context"),
            self.research_activity_type.as_ref().map(|_| "@type"),
            self.aspect.as_ref().map(|_| "aspect"),
            self.logbook.as_ref().map(|_| "logbook"),
            self.notes.as_ref().map(|_| "notes"),
            Some("contact.givenName"),
            Some("contact.familyName"),
            Some("contact.jobTitle"),
            Some("contact.organization"),
            Some("contact.telephone"),
            Some("contact.url"),
            self.contact.affiliation.as_ref().map(|_| "contact.affiliation"),
            self.meta.enrichment.as_ref().map(|_| "meta.enrichment"),
            self.meta.classification.as_ref().map(|_| "meta.classification"),
            self.meta.archive.then_some("meta.archive"),
            Some("meta.draft"),
            Some("meta.status"),
            self.meta.doi.as_ref().map(|_| "meta.doi"),
            self.meta.books.as_ref().map(|_| "meta.books"),
            self.meta.patents.as_ref().map(|_| "meta.patents"),
            self.meta.publications.as_ref().map(|_| "meta.publications"),
            self.meta.swhid.as_ref().map(|_| "meta.swhid"),
            (self.meta.ror.is_some() && !options.allow_infer).then_some("meta.ror"),
            self.meta.additional_type.as_ref().map(|_| "meta.additionalType"),
            self.meta.media.as_ref().map(|_| "meta.media"),
            (!self.meta.technology.is_empty()).then_some("meta.technology"),
            self.meta.resources.as_ref().map(|_| "meta.resources"),
            self.meta.infrastructure.as_ref().map(|_| "meta.infrastructure"),
            self.meta.rights.as_ref().map(|_| "meta.rights"),
        ];
        fields
            .into_iter()
            .flatten()
            .map(|field| ConversionWarning::no_equivalent(RAD, RAID_STANDARD, field))
            .collect()
    }
    fn export_organizations(&self, options: &RaidExportOptions) -> Result<Conversion<Option<Vec<raid::Organization>>>, CrosswalkError> {
        let structured = self
            .meta
            .organizations
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter())
            .map(ActivityOrganization::export)
            .collect::<Vec<_>>();
        let mut organizations = structured.iter().filter_map(|conversion| conversion.value.clone()).collect::<Vec<_>>();
        let mut warnings = structured.into_iter().flat_map(|conversion| conversion.warnings).collect::<Vec<_>>();
        let funders = self.meta.funders.as_ref().map(OneOrMany::as_slice).unwrap_or_default();
        let inferred_funder_date = match options.allow_infer && funders.iter().any(|funder| funder.ror.is_some()) {
            | true => options.inference_date().map(Some),
            | false => Ok(None),
        };
        inferred_funder_date.and_then(|inferred_funder_date| {
            funders.iter().for_each(|funder| match &funder.ror {
                | Some(ror) if options.allow_infer => {
                    inferred_funder_date.iter().for_each(|date| {
                        organizations.push(raid::Organization {
                            id: ror.clone(),
                            schema_uri: Some(DEFAULT_ROR_SCHEMA_URI.to_string()),
                            role: vec![raid::OrganizationRole {
                                id: OrganizationRoleType::Funder,
                                schema_uri: None,
                                date: Date {
                                    start_date: Some(date.clone()),
                                    end_date: None,
                                },
                            }],
                        });
                        warnings.push(ConversionWarning::approximated(
                            RAD,
                            RAID_STANDARD,
                            "funders[].roles[].startDate",
                            "the caller-supplied inference date was used for the funder role",
                        ));
                    });
                }
                | Some(_) => warnings.push(ConversionWarning::omitted(
                    RAD,
                    RAID_STANDARD,
                    "funders[]",
                    "RAiD organization roles require an authored start date",
                )),
                | None => warnings.push(ConversionWarning::omitted(
                    RAD,
                    RAID_STANDARD,
                    "funders[].ror",
                    "RAiD organizations require a ROR identifier",
                )),
            });
            warnings.extend(self.meta.partners.as_ref().into_iter().flat_map(|values| {
                values
                    .iter()
                    .map(|_| ConversionWarning::omitted(RAD, RAID_STANDARD, "partners[]", "legacy partner names have no portable ROR identifier"))
            }));
            warnings.extend(funders.iter().flat_map(|funder| {
                [
                    Some(ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "funders[].name")),
                    funder
                        .identifier
                        .as_ref()
                        .map(|_| ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "funders[].identifier")),
                    (!funder.awards.is_empty()).then(|| ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "funders[].awards")),
                ]
                .into_iter()
                .flatten()
            }));
            let lead_count = organizations
                .iter()
                .flat_map(|organization| organization.role.iter())
                .filter(|role| matches!(role.id, OrganizationRoleType::LeadResearchOrganization))
                .count();
            let lead_inference = match (
                lead_count == 0 && options.allow_infer,
                self.meta.ror.as_ref().and_then(OneOrMany::first).cloned(),
            ) {
                | (true, Some(ror)) => options.inference_date().map(|date| {
                    let role = raid::OrganizationRole {
                        id: OrganizationRoleType::LeadResearchOrganization,
                        schema_uri: None,
                        date: Date {
                            start_date: Some(date),
                            end_date: None,
                        },
                    };
                    match organizations
                        .iter_mut()
                        .find(|organization| Identifier::new(&organization.id).normalized() == Identifier::new(&ror).normalized())
                    {
                        | Some(organization) => organization.role.push(role),
                        | None => organizations.push(raid::Organization {
                            id: ror,
                            schema_uri: Some(DEFAULT_ROR_SCHEMA_URI.to_string()),
                            role: vec![role],
                        }),
                    }
                    warnings.push(ConversionWarning::approximated(
                        RAD,
                        RAID_STANDARD,
                        "organizations[].roles[LeadResearchOrganization]",
                        "lead research organization was inferred from the first legacy ROR",
                    ));
                }),
                | _ => Ok(()),
            };
            lead_inference.and_then(|()| {
                let lead_count = organizations
                    .iter()
                    .flat_map(|organization| organization.role.iter())
                    .filter(|role| matches!(role.id, OrganizationRoleType::LeadResearchOrganization))
                    .count();
                match () {
                    | () if !organizations.is_empty() && lead_count == 0 => Err(CrosswalkError::MissingRequiredField(
                        "organizations[].roles[LeadResearchOrganization]".to_string(),
                    )),
                    | () if lead_count > 1 => Err(CrosswalkError::TransformationFailed {
                        field: "organizations[].roles[LeadResearchOrganization]".to_string(),
                        reason: "RAiD permits one lead research organization".to_string(),
                    }),
                    | () => Ok(Conversion::new((!organizations.is_empty()).then_some(organizations), warnings)),
                }
            })
        })
    }
    fn export_related(&self, options: &RaidExportOptions) -> ExportedRelated {
        let mut related_raids = Vec::new();
        let mut related_objects = Vec::new();
        let mut warnings = Vec::new();
        self.meta
            .related_resources
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter())
            .for_each(|resource| {
                if resource.identifier.scheme.as_deref().is_some_and(|scheme| PID::from(scheme) == PID::RAID) {
                    match RelatedRaidType::try_from(resource.relation_type.as_str()) {
                        | Ok(id) => related_raids.push(raid::RelatedRaid {
                            id: RAID::format(&resource.identifier.value),
                            related_raid_type: raid::RelatedRaidIdentifier { id, schema_uri: None },
                        }),
                        | Err(_) => warnings.push(ConversionWarning::omitted(
                            RAD,
                            RAID_STANDARD,
                            "relatedResources[].relationType",
                            "the relation is not in the RAiD related-activity vocabulary",
                        )),
                    }
                    if resource.relation_type_information.is_some() {
                        warnings.push(ConversionWarning::no_equivalent(
                            RAD,
                            RAID_STANDARD,
                            "relatedResources[].relationTypeInformation",
                        ));
                    }
                    if resource.category.is_some() {
                        warnings.push(ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "relatedResources[].category"));
                    }
                } else if options.allow_infer {
                    match resource.resource_type_general.as_deref().map(ObjectType::from) {
                        | Some(kind) => {
                            related_objects.push(raid::RelatedObject {
                                id: resource.identifier.value.clone(),
                                schema_uri: resource.identifier.scheme_uri.clone(),
                                related_object_type: raid::RelatedObjectIdentifier { id: kind, schema_uri: None },
                                category: vec![raid::RelatedObjectCategory {
                                    id: ObjectCategoryType::Output,
                                    schema_uri: None,
                                }],
                            });
                            warnings.push(ConversionWarning::approximated(
                                RAD,
                                RAID_STANDARD,
                                "relatedResources[].category",
                                "RAiD output category was inferred by explicit request",
                            ));
                        }
                        | None => warnings.push(ConversionWarning::omitted(
                            RAD,
                            RAID_STANDARD,
                            "relatedResources[].resourceTypeGeneral",
                            "the related resource type is not representable by RAiD",
                        )),
                    }
                } else {
                    warnings.push(ConversionWarning::omitted(
                        RAD,
                        RAID_STANDARD,
                        "relatedResources[]",
                        "RAiD related objects require an explicit input/output/process category",
                    ));
                }
            });
        self.meta
            .outputs
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter())
            .for_each(|output| {
                let identifier = output.doi.as_ref().or(output.identifier.as_ref());
                let kind = output
                    .resource_type_general
                    .as_ref()
                    .map(|kind| ObjectType::from(kind.to_string().as_str()));
                match (identifier, kind) {
                    | (Some(identifier), Some(kind)) => {
                        related_objects.push(raid::RelatedObject {
                            id: identifier.clone(),
                            schema_uri: output.doi.as_ref().map(|_| "https://doi.org/".to_string()),
                            related_object_type: raid::RelatedObjectIdentifier { id: kind, schema_uri: None },
                            category: vec![raid::RelatedObjectCategory {
                                id: ObjectCategoryType::Output,
                                schema_uri: None,
                            }],
                        });
                        warnings.push(ConversionWarning::simplified(
                            RAD,
                            RAID_STANDARD,
                            "outputs[]",
                            "only the output identifier, general type, and output category are represented",
                        ));
                    }
                    | _ => warnings.push(ConversionWarning::omitted(
                        RAD,
                        RAID_STANDARD,
                        "outputs[]",
                        "RAiD related objects require an identifier and representable resource type",
                    )),
                }
            });
        Conversion::new(
            (
                (!related_raids.is_empty()).then_some(related_raids),
                (!related_objects.is_empty()).then_some(related_objects),
            ),
            warnings,
        )
    }
    fn export_subjects(&self) -> Option<Vec<raid::Subject>> {
        let structured = self
            .meta
            .subjects
            .as_ref()
            .into_iter()
            .flat_map(|values| values.iter())
            .map(|subject| raid::Subject {
                id: subject.value_uri.clone().unwrap_or_else(|| subject.subject.clone()),
                schema_uri: subject.scheme_uri.clone(),
                keyword: Some(vec![raid::Keyword {
                    text: subject.subject.clone(),
                    language: None,
                }]),
            });
        let keywords = self.meta.keywords.iter().map(|keyword| raid::Subject {
            id: keyword.clone(),
            schema_uri: None,
            keyword: Some(vec![raid::Keyword {
                text: keyword.clone(),
                language: None,
            }]),
        });
        let subjects = structured.chain(keywords).collect::<Vec<_>>();
        (!subjects.is_empty()).then_some(subjects)
    }
    fn export_titles(&self, date: &Date) -> Vec<raid::Title> {
        let primary = raid::Title {
            text: Some(self.title.clone()),
            title_type: Some(raid::TitleIdentifier::init().id(TitleType::Primary).build()),
            language: self.meta.languages.as_ref().and_then(OneOrMany::first).cloned(),
            start_date: date.start_date.clone(),
            end_date: date.end_date.as_ref().and_then(|end| end.get(0..4).map(ToString::to_string)),
        };
        core::iter::once(primary)
            .chain(self.subtitle.as_ref().map(|subtitle| raid::Title {
                text: Some(subtitle.clone()),
                title_type: Some(raid::TitleIdentifier::init().id(TitleType::Alternative).build()),
                language: self.meta.languages.as_ref().and_then(OneOrMany::first).cloned(),
                start_date: date.start_date.clone(),
                end_date: date.end_date.as_ref().and_then(|end| end.get(0..4).map(ToString::to_string)),
            }))
            .collect()
    }
}
impl ResearchActivity {
    /// Convert this research activity to RAiD metadata with an explicit inference policy.
    pub fn to_raid(&self, options: &RaidExportOptions) -> Result<(raid::Metadata, Vec<ConversionWarning>), CrosswalkError> {
        let inference = match options.allow_infer {
            | true => options.inference_date().map(Some),
            | false => Ok(None),
        };
        inference.and_then(|_| self.export_date(options)).and_then(|date_conversion| {
            let (date, date_warnings) = date_conversion.into_parts();
            self.export_access(options).and_then(|access_conversion| {
                let (access, access_warnings) = access_conversion.into_parts();
                self.export_contributors(options, &date).and_then(|contributor_conversion| {
                    let (contributors, contributor_warnings) = contributor_conversion.into_parts();
                    self.export_organizations(options).and_then(|organization_conversion| {
                        let (organizations, organization_warnings) = organization_conversion.into_parts();
                        self.export_identifier(options, organizations.as_deref())
                            .and_then(|identifier_conversion| {
                                let (identifier, identifier_warnings) = identifier_conversion.into_parts();
                                Conversion::<Option<raid::MetadataMetadata>>::try_from(self).and_then(|metadata_conversion| {
                                    let (metadata, metadata_warnings) = metadata_conversion.into_parts();
                                    let alternate_identifier = Conversion::<Option<Vec<raid::AlternateIdentifier>>>::from(self).value;
                                    let alternate_url = self.meta.websites.as_ref().map(|websites| {
                                        websites
                                            .iter()
                                            .map(|website| raid::AlternateUrl::builder().url(website.url.clone()).build())
                                            .collect::<Vec<_>>()
                                    });
                                    let descriptions = self.sections.as_ref().map(|sections| {
                                        vec![raid::Description {
                                            text: Some(sections.mission.clone()),
                                            description_type: Some(raid::DescriptionIdentifier::init().id(DescriptionType::Primary).build()),
                                            language: None,
                                        }]
                                    });
                                    let ((related_raid, related_object), related_warnings) = self.export_related(options).into_parts();
                                    let mut warnings = date_warnings
                                        .into_iter()
                                        .chain(access_warnings)
                                        .chain(contributor_warnings)
                                        .chain(organization_warnings)
                                        .chain(identifier_warnings)
                                        .chain(metadata_warnings)
                                        .chain(related_warnings)
                                        .chain(self.export_local_warnings(options))
                                        .collect::<Vec<_>>();
                                    warnings.extend(self.sections.as_ref().into_iter().flat_map(|sections| {
                                        [
                                            Some(ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "sections.challenge")),
                                            (!sections.approach.is_empty())
                                                .then(|| ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "sections.approach")),
                                            (!sections.impact.is_empty())
                                                .then(|| ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "sections.impact")),
                                            sections
                                                .achievement
                                                .as_ref()
                                                .map(|_| ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "sections.achievement")),
                                            sections
                                                .capabilities
                                                .as_ref()
                                                .map(|_| ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "sections.capabilities")),
                                            Some(ConversionWarning::no_equivalent(RAD, RAID_STANDARD, "sections.research")),
                                        ]
                                        .into_iter()
                                        .flatten()
                                    }));
                                    let target = raid::Metadata {
                                        access: Some(access),
                                        contributor: Some(contributors),
                                        date: Some(date.clone()),
                                        identifier,
                                        title: Some(self.export_titles(&date)),
                                        alternate_identifier,
                                        alternate_url,
                                        description: descriptions,
                                        metadata,
                                        organization: organizations,
                                        related_object,
                                        related_raid,
                                        spatial_coverage: self.meta.spatial_coverage.as_ref().map(|values| values.iter().cloned().collect()),
                                        subject: self.export_subjects(),
                                        traditional_knowledge_label: None,
                                    };
                                    target
                                        .validate()
                                        .map(|_| (target, warnings))
                                        .map_err(|error| CrosswalkError::BuildFailed(error.to_string()))
                                })
                            })
                    })
                })
            })
        })
    }
}
impl From<&raid::Access> for StructuredRights {
    fn from(value: &raid::Access) -> Self {
        let access = value.access_type.as_ref().map(|access| match access.id {
            | AccessType::OpenAccess => RightsAccess::Open,
            | AccessType::EmbargoedAccess => RightsAccess::Embargoed,
        });
        Self {
            rights: value.statement.as_ref().and_then(|statement| statement.text.clone()),
            rights_uri: None,
            rights_identifier: None,
            rights_identifier_scheme: None,
            scheme_uri: value.access_type.as_ref().map(|access| access.schema_uri.clone()),
            access,
        }
    }
}
fn agent_identifier(value: &str, scheme: Option<&str>) -> Option<(String, String)> {
    let scheme = PID::infer(scheme.unwrap_or(value));
    match scheme {
        | PID::ORCID if rules::orcid(value).is_ok() => Some((value.to_string(), ORCID_SCHEMA_URI.to_string())),
        | PID::ISNI if rules::isni(value).is_ok() => Some((value.to_string(), ISNI_SCHEMA_URI.to_string())),
        | _ => None,
    }
}
fn append_midnight_utc_suffix(value: &str) -> String {
    format!("{value}T00:00:00Z")
}
fn to_date(value: &str) -> Option<String> {
    DATE_TIME_PARSER.parse_pieces(value).ok().map(|pieces| pieces.date().to_string())
}

#[cfg(all(test, feature = "std"))]
mod tests;
