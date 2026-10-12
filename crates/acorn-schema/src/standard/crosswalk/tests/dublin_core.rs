use super::super::{Crosswalk, DublinCoreImportOptions};
use crate::research_activity::common::ControlledSubject;
use crate::research_activity::output::{Contributor, ResearchOutput};
use crate::research_activity::ResearchActivity;
use crate::standard::datacite::{self, Description, DescriptionType, Publisher, ResourceTypeGeneral};
use crate::standard::dublin_core::{
    DublinCoreDescription, DublinCoreDocument, DublinCoreLiteral, DublinCoreResource, DublinCoreValue, DCTERMS_NAMESPACE, ELEMENTS_NAMESPACE,
    TYPE_NAMESPACE,
};
use crate::{ContactPoint, OneOrMany};
use acorn_core::prelude::{format, vec, ToString, Vec};
use alloc::collections::BTreeMap;

use super::*;

#[test]
fn test_dublin_core_import_selects_titled_root_and_warns_only_for_extensions() {
    let document = DublinCoreDocument::new(vec![
        DublinCoreDescription {
            properties: BTreeMap::new(),
            subject: DublinCoreResource::iri("https://example.org/untitled"),
            types: Vec::new(),
        },
        DublinCoreDescription {
            properties: BTreeMap::from([
                (
                    format!("{ELEMENTS_NAMESPACE}title"),
                    vec![DublinCoreValue::Literal(DublinCoreLiteral {
                        datatype: None,
                        language: None,
                        value: "Legacy title".to_string(),
                    })],
                ),
                (
                    "https://schema.org/version".to_string(),
                    vec![DublinCoreValue::Literal(DublinCoreLiteral {
                        datatype: None,
                        language: None,
                        value: "1.0".to_string(),
                    })],
                ),
            ]),
            subject: DublinCoreResource::iri("https://example.org/titled"),
            types: Vec::new(),
        },
    ]);
    let (output, warnings) = <DublinCoreDocument as Crosswalk<ResearchOutput>>::crosswalk(&document).expect("import Dublin Core");
    assert_eq!(output.title, "Legacy title");
    assert_eq!(output.identifier.as_deref(), Some("https://example.org/titled"));
    assert_eq!(warnings.len(), 2);
    assert!(warnings.iter().any(|warning| warning.field == "descriptions[1..]"));
    assert!(warnings.iter().any(|warning| warning.field == "https://schema.org/version"));
}
#[test]
fn test_dublin_core_research_output_round_trip_maps_common_fields() {
    let output = ResearchOutput {
        access: None,
        alternate_identifiers: None,
        contributors: None,
        creators: Some(vec![Contributor {
            affiliations: Vec::new(),
            contributor_type: None,
            corresponding: None,
            name: "Ada Lovelace".to_string(),
            name_type: None,
            orcid: None,
            position: None,
            roles: Vec::new(),
            ror: None,
        }]),
        dates: None,
        descriptions: Some(vec![Description {
            description: "Summary".to_string(),
            description_type: Some(DescriptionType::Abstract),
            language: None,
        }]),
        doi: Some("10.1234/example".to_string()),
        formats: Some(vec!["text/csv".to_string()]),
        funding: None,
        geo_locations: None,
        identifier: Some("https://example.org/work".to_string()),
        keywords: Vec::new(),
        kind: None,
        language: Some("en".to_string()),
        publication_year: Some(2024),
        publisher: Some(Publisher {
            name: "Example Publisher".to_string(),
            publisher_identifier: None,
            publisher_identifier_scheme: None,
            scheme_uri: None,
        }),
        related_identifiers: None,
        related_items: None,
        resource_type: Some("Dataset".to_string()),
        resource_type_general: Some(ResourceTypeGeneral::Dataset),
        rights_list: None,
        sizes: None,
        subjects: Some(vec![ControlledSubject {
            scheme_uri: None,
            subject: "metadata".to_string(),
            subject_scheme: None,
            value_uri: None,
        }]),
        title: "A dataset".to_string(),
        version: None,
        websites: None,
    };
    let (document, warnings) = <ResearchOutput as Crosswalk<DublinCoreDocument>>::crosswalk(&output).expect("export Dublin Core");
    let (decoded, _) = <DublinCoreDocument as Crosswalk<ResearchOutput>>::crosswalk(&document).expect("import Dublin Core");
    assert!(warnings.is_empty());
    assert_eq!(decoded.title, output.title);
    assert_eq!(decoded.doi, output.doi);
    assert_eq!(decoded.resource_type_general, output.resource_type_general);
    assert_eq!(
        decoded.creators.and_then(|values| values.into_iter().next()).map(|value| value.name),
        Some("Ada Lovelace".to_string())
    );
}
#[test]
fn test_dublin_core_to_datacite_requires_deposit_fields() {
    let document = DublinCoreDocument::new(vec![DublinCoreDescription {
        properties: BTreeMap::from([(
            format!("{DCTERMS_NAMESPACE}title"),
            vec![DublinCoreValue::Literal(DublinCoreLiteral {
                datatype: None,
                language: None,
                value: "Incomplete".to_string(),
            })],
        )]),
        subject: DublinCoreResource::iri("https://example.org/work"),
        types: vec![format!("{TYPE_NAMESPACE}Dataset")],
    }]);
    assert!(<DublinCoreDocument as Crosswalk<datacite::Record>>::crosswalk(&document).is_err());
}
#[test]
fn test_dublin_core_to_research_activity_requires_local_context() {
    let document = DublinCoreDocument::new(vec![DublinCoreDescription {
        properties: BTreeMap::from([(
            format!("{ELEMENTS_NAMESPACE}title"),
            vec![DublinCoreValue::Literal(DublinCoreLiteral {
                datatype: None,
                language: None,
                value: "Imported dataset".to_string(),
            })],
        )]),
        subject: DublinCoreResource::iri("https://example.org/work"),
        types: vec![format!("{TYPE_NAMESPACE}Dataset")],
    }]);
    assert!(document.to_research_activity(&DublinCoreImportOptions::default()).is_err());
    assert!(document
        .to_research_activity(&DublinCoreImportOptions {
            contact: Some(ContactPoint::default()),
            identifier: None,
        })
        .is_err());
    let (activity, _) = document
        .to_research_activity(&DublinCoreImportOptions {
            contact: Some(ContactPoint::default()),
            identifier: Some("dublin-core-import".to_string()),
        })
        .expect("import Dublin Core with local context");
    assert_eq!(activity.meta.identifier, "dublin-core-import");
    assert_eq!(activity.title, "Imported dataset");
    assert_eq!(activity.meta.outputs.map(OneOrMany::into_vec).map(|values| values.len()), Some(1));
}
