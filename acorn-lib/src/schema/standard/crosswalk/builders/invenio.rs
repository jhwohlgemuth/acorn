//! Invenio Record schema builder
//!
//! Reconstructs invenio::Record from normalized FieldMap representation.
use crate::schema::standard::crosswalk::{CrosswalkError, FieldMap, SchemaBuilder};
use crate::schema::standard::invenio;

impl SchemaBuilder for invenio::Record {
    fn build_from_fields(fields: &FieldMap) -> Result<Self, CrosswalkError> {
        let identifier_str = fields.get_string("identifier")?;
        let mut metadata = invenio::Metadata {
            resource_type: None,
            title: fields.get_string_opt("title"),
            publication_date: fields.get_number_opt("publication-year").map(|n| format!("{:04}-01-01", n as i32)),
            creators: None,
            additional_titles: None,
            description: fields.get_string_opt("description"),
            additional_descriptions: None,
            rights: None,
            copyright: None,
            contributors: None,
            subjects: None,
            languages: None,
            dates: None,
            version: fields.get_string_opt("version"),
            publisher: None,
            identifiers: None,
            related_identifiers: None,
            sizes: None,
            formats: None,
            locations: None,
            funding: None,
            references: None,
        };
        if let Some(creator_names) = fields.get_string_vec_opt("creators") {
            metadata.creators = Some(
                creator_names
                    .into_iter()
                    .map(|name| invenio::Creator {
                        person_or_org: invenio::PersonOrOrganization {
                            kind: "personal".to_string(),
                            name: Some(name),
                            given_name: None,
                            family_name: None,
                            identifiers: None,
                        },
                        affiliations: None,
                        role: None,
                    })
                    .collect(),
            );
        }
        if let Some(rt_str) = fields.get_string_opt("resource-type") {
            metadata.resource_type = Some(invenio::ResourceType {
                id: Some(rt_str),
                title: None,
            });
        }
        if let Some(lang_str) = fields.get_string_opt("language") {
            metadata.languages = Some(vec![invenio::Language { id: lang_str }]);
        }
        if let Some(ids) = fields.get_string_vec_opt("alternate-identifiers") {
            metadata.identifiers = Some(
                ids.into_iter()
                    .map(|identifier| invenio::Identifier {
                        identifier,
                        scheme: "unknown".to_string(),
                    })
                    .collect(),
            );
        }
        if let Some(subjects) = fields.get_string_vec_opt("subjects") {
            metadata.subjects = Some(
                subjects
                    .into_iter()
                    .map(|subject| invenio::Subject {
                        id: None,
                        subject: Some(subject),
                    })
                    .collect(),
            );
        }
        if let Some(license_iri) = fields.get_iri_opt("license") {
            metadata.rights = Some(vec![invenio::Right {
                id: None,
                title: None,
                description: None,
                link: Some(license_iri),
            }]);
        }
        if let Some(contributor_names) = fields.get_string_vec_opt("contributors") {
            metadata.contributors = Some(
                contributor_names
                    .into_iter()
                    .map(|name| invenio::Contributor {
                        person_or_org: invenio::PersonOrOrganization {
                            kind: "personal".to_string(),
                            name: Some(name),
                            given_name: None,
                            family_name: None,
                            identifiers: None,
                        },
                        role: invenio::ControlledVocabularyValue {
                            id: "contributor".to_string(),
                        },
                        affiliations: None,
                    })
                    .collect(),
            );
        }
        Ok(invenio::Record {
            schema: Some("https://inveniordm.docs.cern.ch/reference/metadata/".to_string()),
            id: Some(identifier_str.clone()),
            pid: Some(invenio::PersistentIdentifier {
                pk: None,
                status: Some("U".to_string()),
            }),
            pids: None,
            parent: None,
            access: None,
            metadata: Some(metadata),
            custom_fields: None,
            files: None,
            tombstone: None,
            created: fields.get_date_opt("created"),
            updated: fields.get_date_opt("updated"),
        })
    }
}
