use crate::io::api::orcid;
use crate::io::enrichment::{Enrich, OrcidBatch, OrcidBatchResult, OrcidExt, OrcidResolution};
use acorn_schema::research_activity::output::{Contributor as OutputContributor, ResearchOutput};
use acorn_schema::research_activity::{ActivityContributor, ResearchActivity, ResearchActivityMetadata};
use acorn_schema::standard::cff::{Agent as CffAgent, Cff, Person as CffPerson, Reference as CffReference};
use acorn_schema::standard::{datacite, invenio};
use acorn_schema::ContactPoint;

fn contact(given_name: &str, family_name: &str, identifier: Option<&str>) -> ContactPoint {
    ContactPoint {
        given_name: given_name.to_string(),
        family_name: family_name.to_string(),
        identifier: identifier.map(ToString::to_string),
        ..ContactPoint::default()
    }
}
fn profile(identifier: &str, given_name: Option<&str>, family_name: Option<&str>) -> orcid::SearchResult {
    orcid::SearchResult {
        orcid_id: Some(identifier.to_string()),
        given_names: given_name.map(ToString::to_string),
        family_names: family_name.map(ToString::to_string),
        credit_name: None,
        emails: None,
        institution_names: None,
        other_name: None,
    }
}
fn resolved(path: &str, profile: orcid::SearchResult) -> OrcidBatchResult {
    OrcidBatchResult {
        profiles: vec![(path.to_string(), profile)],
        ..OrcidBatchResult::default()
    }
}
fn response(results: impl IntoIterator<Item = orcid::SearchResult>) -> orcid::SearchResponse {
    let results = results.into_iter().collect::<Vec<_>>();
    orcid::SearchResponse {
        num_found: results.len(),
        namespace: String::new(),
        results,
    }
}
#[test]
fn test_orcid_batch_deduplicates_display_name_requests() {
    let activity = ResearchActivity {
        contact: contact("", "", None),
        meta: ResearchActivityMetadata {
            contributors: Some(vec![ActivityContributor::init().name("Jane Doe").build()].into()),
            outputs: Some(
                vec![ResearchOutput::init()
                    .title("Output".to_string())
                    .contributors(vec![OutputContributor::init().name("Jane Doe").build()])
                    .build()]
                .into(),
            ),
            ..ResearchActivityMetadata::default()
        },
        ..ResearchActivity::default()
    };
    let batch = OrcidBatch::new(activity.orcid_people(None));
    assert_eq!(batch.requests.len(), 1);
    assert_eq!(batch.requests.first().expect("deduplicated request").paths.len(), 2);
}
#[test]
fn test_orcid_contact_affiliation_queries_root_parent_ror() {
    let nssd = ContactPoint {
        affiliation: Some("National Security Sciences Directorate".to_string()),
        ..contact("Marie", "Urban", None)
    };
    let identity = nssd.orcid_identity().expect("NSSD contact identity");
    assert_eq!(
        identity
            .ror_query()
            .expect("name-and-ROR query")
            .to_string::<orcid::SearchField, orcid::OutputColumn>(),
        "q=given-names:Marie+AND+family-name:Urban+AND+ror-org-id:%22https%3A%2F%2Fror.org%2F01qz5mb56%22"
    );
    let alias = ContactPoint {
        affiliation: Some("NSSD".to_string()),
        ..contact("Marie", "Urban", None)
    };
    assert!(alias.orcid_identity().expect("NSSD alias identity").ror_query().is_some());
    let selected = identity
        .select(&response([
            orcid::SearchResult {
                institution_names: Some(vec!["Elsewhere".to_string()]),
                ..profile("0000-0001-6229-6300", Some("Marie"), Some("Urban"))
            },
            orcid::SearchResult {
                institution_names: Some(vec!["Oak Ridge National Laboratory".to_string()]),
                ..profile("0000-0001-9571-832X", Some("Marie"), Some("Urban"))
            },
        ]))
        .expect("root organization resolves ambiguity")
        .expect("ORNL profile");
    assert_eq!(selected.orcid_id.as_deref(), Some("0000-0001-9571-832X"));
}
#[test]
fn test_orcid_contact_email_queries_refine_resolution_and_identity_key() {
    let ornl = ContactPoint {
        email: " Jane@ORNL.GOV ".to_string(),
        ..contact("Jane", "Doe", None)
    };
    let other = ContactPoint {
        email: "other@ornl.gov".to_string(),
        ..contact("Jane", "Doe", None)
    };
    let identity = ornl.orcid_identity().expect("ORNL contact identity");
    assert_eq!(
        identity
            .email_query()
            .expect("email-and-name query")
            .to_string::<orcid::SearchField, orcid::OutputColumn>(),
        "q=given-names:Jane+AND+family-name:Doe+AND+email:jane%40ornl.gov"
    );
    let query = identity
        .email_domain_query()
        .expect("email-domain refinement query")
        .to_string::<orcid::SearchField, orcid::OutputColumn>();
    assert_eq!(query, "q=given-names:Jane+AND+family-name:Doe+AND+email:%2A%40ornl.gov");
    assert_eq!(
        identity.search_query().to_string::<orcid::SearchField, orcid::OutputColumn>(),
        "q=given-names:Jane+AND+family-name:Doe"
    );
    assert_ne!(identity.key(), other.orcid_identity().expect("other contact identity").key());
}
#[tokio::test]
async fn test_orcid_enrichment_skips_complete_or_incomplete_contacts() {
    let incomplete = ResearchActivity {
        contact: contact("Jane", "", None),
        ..ResearchActivity::default()
    };
    let (actual, conflicts, failures) = incomplete.enrich_orcid().await.expect("incomplete contact should be skipped");
    assert!(actual.contact.identifier.is_none());
    assert_eq!(actual.contact.given_name, "Jane");
    assert!(actual.contact.family_name.is_empty());
    assert!(conflicts.is_empty());
    assert!(failures.is_empty());
    let complete = ResearchActivity {
        contact: contact("Jason", "Wohlgemuth", Some("0000-0002-2057-9115")),
        ..ResearchActivity::default()
    };
    let (actual, conflicts, failures) = complete.enrich_orcid().await.expect("complete contact should be skipped");
    assert_eq!(actual.contact.given_name, "Jason");
    assert_eq!(actual.contact.family_name, "Wohlgemuth");
    assert!(conflicts.is_empty());
    assert!(failures.is_empty());
}
#[test]
fn test_orcid_identifier_accepts_exact_alternate_display_name() {
    let identity = contact("Lexie", "Yang", None).orcid_identity().expect("Lexie Yang identity");
    let candidate = orcid::SearchResult {
        other_name: Some(vec!["Lexie Yang".to_string()]),
        ..profile("0000-0003-2252-6778", Some("Hsiuhan Lexie"), Some("Yang"))
    };
    assert_eq!(
        identity
            .select(&response([candidate]))
            .expect("alternate display-name match")
            .and_then(|profile| profile.orcid_id)
            .as_deref(),
        Some("0000-0003-2252-6778")
    );
}
#[test]
fn test_orcid_identifier_accepts_given_name_component() {
    let identity = contact("Lexie", "Yang", None).orcid_identity().expect("Lexie Yang identity");
    assert_eq!(
        identity
            .select(&response([profile("0000-0003-2252-6778", Some("Hsiuhan Lexie"), Some("Yang"),)]))
            .expect("given-name component match")
            .and_then(|profile| profile.orcid_id)
            .as_deref(),
        Some("0000-0003-2252-6778")
    );
}
#[test]
fn test_orcid_identifier_uses_exact_matches_and_reports_ambiguity() {
    let exact = contact(" Jason ", "Wohlgemuth", None);
    let exact_identity = exact.orcid_identity().expect("exact contact lookup");
    let selected = exact_identity
        .select(&response([
            profile("0000-0002-1825-0097", Some("Other"), Some("Person")),
            profile("0000-0002-2057-9115", Some("jason"), Some("Wohlgemuth")),
        ]))
        .expect("unique exact match");
    let actual = exact.with_orcid_profile(selected.as_ref());
    assert_eq!(actual.identifier.as_deref(), Some("0000-0002-2057-9115"));
    let email = ContactPoint {
        email: "jane@example.com".to_string(),
        ..contact("Jane", "Doe", None)
    };
    let email_match = orcid::SearchResult {
        emails: Some(vec![email.email.clone()]),
        ..profile("0000-0002-2057-9115", Some("Janet"), Some("Doe"))
    };
    assert_eq!(
        email
            .orcid_identity()
            .expect("email contact lookup")
            .select(&response([email_match]))
            .expect("unique email match")
            .and_then(|profile| profile.orcid_id)
            .as_deref(),
        Some("0000-0002-2057-9115")
    );
    let ambiguous = ContactPoint {
        email: "jane@example.com".to_string(),
        ..contact("Jane", "Doe", None)
    };
    let matches = ["0000-0002-1825-0097", "0000-0002-2057-9115"].map(|identifier| orcid::SearchResult {
        emails: Some(vec![ambiguous.email.clone()]),
        ..profile(identifier, Some("Jane"), Some("Doe"))
    });
    assert!(ambiguous
        .orcid_identity()
        .expect("ambiguous contact lookup")
        .select(&response(matches))
        .is_err());
}
#[test]
fn test_orcid_names_fill_only_missing_values() {
    let blank = contact("", "", Some("0000-0002-2057-9115"));
    let actual = blank.with_orcid_profile(Some(&profile("0000-0002-2057-9115", Some("Jane"), Some("Doe"))));
    assert_eq!(actual.given_name, "Jane");
    assert_eq!(actual.family_name, "Doe");
    let partial = contact("Jane", "", Some("0000-0002-2057-9115"));
    let actual = partial.with_orcid_profile(Some(&profile("0000-0002-2057-9115", Some("Janet"), Some("Doe"))));
    assert_eq!(actual.given_name, "Jane");
    assert_eq!(actual.family_name, "Doe");
    let unavailable = contact("", " ", Some("0000-0002-2057-9115"));
    let actual = unavailable.with_orcid_profile(Some(&profile("0000-0002-2057-9115", None, Some(" "))));
    assert!(actual.given_name.is_empty());
    assert_eq!(actual.family_name, " ");
}
#[test]
fn test_orcid_refinement_preserves_ambiguous_response_when_empty() {
    let identity = contact("Marie", "Urban", None).orcid_identity().expect("Marie Urban identity");
    let ambiguous = response([
        profile("0000-0001-6229-6300", Some("Marie"), Some("Urban")),
        profile("0000-0001-9571-832X", Some("Marie"), Some("Urban")),
    ]);
    let actual = identity.prefer_refined_response(ambiguous, response([]));
    assert!(identity.select(&actual).is_err());
}
#[test]
fn test_orcid_resolution_distinguishes_missing_and_ambiguous() {
    let identity = contact("Marie", "Urban", None).orcid_identity().expect("Marie Urban identity");
    assert!(matches!(identity.resolve(&response([])), OrcidResolution::Missing));
    assert!(matches!(
        identity.resolve(&response([
            profile("0000-0001-6229-6300", Some("Marie"), Some("Urban")),
            profile("0000-0001-9571-832X", Some("Marie"), Some("Urban")),
        ])),
        OrcidResolution::Ambiguous
    ));
}
#[test]
fn test_orcid_source_prefers_identifier_then_names() {
    let identified = ResearchActivity {
        contact: contact("", "", Some("0000-0002-2057-9115")),
        ..ResearchActivity::default()
    };
    assert_eq!(identified.orcid_source().as_deref(), Some("0000-0002-2057-9115"));
    let named = ResearchActivity {
        contact: contact("Jane", "Doe", None),
        ..ResearchActivity::default()
    };
    assert_eq!(named.orcid_source().as_deref(), Some("Jane Doe"));
}
#[test]
fn test_orcid_updates_activity_and_output_people() {
    let activity = ResearchActivity {
        contact: contact("", "", None),
        meta: ResearchActivityMetadata {
            contributors: Some(vec![ActivityContributor::init().name("Jane Doe").build()].into()),
            outputs: Some(
                vec![ResearchOutput::init()
                    .title("Output".to_string())
                    .creators(vec![OutputContributor::init().name("Jane Doe").build()])
                    .build()]
                .into(),
            ),
            ..ResearchActivityMetadata::default()
        },
        ..ResearchActivity::default()
    };
    let profile = profile("0000-0002-2057-9115", Some("Jane"), Some("Doe"));
    let result = OrcidBatchResult {
        profiles: vec![
            ("meta.contributors[0]".to_string(), profile.clone()),
            ("meta.outputs[0].creators[0]".to_string(), profile),
        ],
        ..OrcidBatchResult::default()
    };
    let actual = activity.with_orcid_profiles(&result, None);
    let contributor = actual
        .meta
        .contributors
        .as_ref()
        .expect("activity contributors")
        .first()
        .expect("activity contributor");
    assert_eq!(contributor.identifiers.len(), 1);
    assert_eq!(
        contributor.identifiers.first().expect("ORCID identifier").scheme.as_deref(),
        Some("ORCID")
    );
    let creator = actual
        .meta
        .outputs
        .as_ref()
        .expect("outputs")
        .first()
        .expect("output")
        .creators
        .as_ref()
        .expect("creators")
        .first()
        .expect("creator");
    assert_eq!(creator.orcid.as_deref(), Some("0000-0002-2057-9115"));
}
#[test]
fn test_orcid_updates_cff_person_and_skips_entity() {
    let person = CffPerson {
        given_names: Some("Jane".to_string()),
        family_names: Some("Doe".to_string()),
        ..serde_json::from_value(serde_json::json!({})).expect("empty CFF person")
    };
    let entity = serde_json::from_value(serde_json::json!({ "name": "Example Team" })).expect("CFF entity");
    let cff = Cff {
        authors: vec![CffAgent::Person(person), CffAgent::Entity(entity)],
        ..Cff::default()
    };
    let actual = cff.with_orcid_profiles(&resolved("authors[0]", profile("0000-0002-2057-9115", Some("Jane"), Some("Doe"))), None);
    assert!(
        matches!(actual.authors.first(), Some(CffAgent::Person(person)) if person.orcid.as_deref() == Some("https://orcid.org/0000-0002-2057-9115"))
    );
    assert!(matches!(actual.authors.get(1), Some(CffAgent::Entity(entity)) if entity.orcid.is_none()));
}
#[test]
fn test_orcid_updates_cff_reference_people() {
    let reference = |label: &str| {
        serde_json::from_value::<CffReference>(serde_json::json!({
            "authors": [{"given-names": label, "family-names": "Author"}],
            "contact": [{"given-names": label, "family-names": "Contact"}],
            "editors": [{"given-names": label, "family-names": "Editor"}],
            "editors-series": [{"given-names": label, "family-names": "Series"}],
            "recipients": [{"given-names": label, "family-names": "Recipient"}],
            "senders": [{"given-names": label, "family-names": "Sender"}],
            "title": format!("{label} reference"),
            "translators": [{"given-names": label, "family-names": "Translator"}],
            "type": "article"
        }))
        .expect("CFF reference")
    };
    let cff = Cff {
        preferred_citation: Some(reference("Preferred")),
        references: Some(vec![reference("Related")]),
        ..Cff::default()
    };
    let profile = profile("0000-0002-2057-9115", Some("Resolved"), Some("Person"));
    let paths = [
        "preferredCitation.authors[0]",
        "preferredCitation.contact[0]",
        "preferredCitation.editors[0]",
        "preferredCitation.editorsSeries[0]",
        "preferredCitation.recipients[0]",
        "preferredCitation.senders[0]",
        "preferredCitation.translators[0]",
        "references[0].authors[0]",
        "references[0].contact[0]",
        "references[0].editors[0]",
        "references[0].editorsSeries[0]",
        "references[0].recipients[0]",
        "references[0].senders[0]",
        "references[0].translators[0]",
    ];
    let result = OrcidBatchResult {
        profiles: paths.into_iter().map(|path| (path.to_string(), profile.clone())).collect(),
        ..OrcidBatchResult::default()
    };
    let actual = cff.with_orcid_profiles(&result, None);
    let preferred = actual.preferred_citation.expect("preferred citation");
    let references = actual.references.expect("references");
    let related = references.first().expect("related reference");
    let people = preferred
        .authors
        .iter()
        .chain(preferred.contact.as_ref().into_iter().flatten())
        .chain(preferred.editors.as_ref().into_iter().flatten())
        .chain(preferred.editors_series.as_ref().into_iter().flatten())
        .chain(preferred.recipients.as_ref().into_iter().flatten())
        .chain(preferred.senders.as_ref().into_iter().flatten())
        .chain(preferred.translators.as_ref().into_iter().flatten())
        .chain(related.authors.iter())
        .chain(related.contact.as_ref().into_iter().flatten())
        .chain(related.editors.as_ref().into_iter().flatten())
        .chain(related.editors_series.as_ref().into_iter().flatten())
        .chain(related.recipients.as_ref().into_iter().flatten())
        .chain(related.senders.as_ref().into_iter().flatten())
        .chain(related.translators.as_ref().into_iter().flatten())
        .collect::<Vec<_>>();
    assert!(people
        .iter()
        .all(|person| matches!(person, CffAgent::Person(person) if person.orcid.as_deref() == Some("https://orcid.org/0000-0002-2057-9115"))));
}
#[test]
fn test_orcid_updates_datacite_and_invenio_people() {
    let datacite: datacite::Record = serde_json::from_value(serde_json::json!({
        "id": "10.1234/example",
        "type": "dois",
        "attributes": {
            "doi": "10.1234/example",
            "creators": [{"name": "Jane Doe"}, {"name": "Example Team", "nameType": "Organizational"}]
        }
    }))
    .expect("DataCite record");
    let actual = datacite.with_orcid_profiles(
        &resolved("attributes.creators[0]", profile("0000-0002-2057-9115", Some("Jane"), Some("Doe"))),
        None,
    );
    let creators = actual.attributes.creators.expect("DataCite creators");
    assert_eq!(
        creators
            .first()
            .expect("personal DataCite creator")
            .name_identifiers
            .as_ref()
            .map(Vec::len),
        Some(1)
    );
    assert!(creators.get(1).expect("organizational DataCite creator").name_identifiers.is_none());
    let invenio: invenio::Record = serde_json::from_value(serde_json::json!({
        "metadata": {
            "creators": [
                {"person_or_org": {"type": "personal", "name": "Jane Doe"}},
                {"person_or_org": {"type": "organizational", "name": "Example Team"}}
            ]
        }
    }))
    .expect("Invenio record");
    let actual = invenio.with_orcid_profiles(
        &resolved("metadata.creators[0]", profile("0000-0002-2057-9115", Some("Jane"), Some("Doe"))),
        None,
    );
    let creators = actual.metadata.expect("Invenio metadata").creators.expect("Invenio creators");
    assert_eq!(
        creators
            .first()
            .expect("personal Invenio creator")
            .person_or_org
            .identifiers
            .as_ref()
            .map(Vec::len),
        Some(1)
    );
    assert!(creators
        .get(1)
        .expect("organizational Invenio creator")
        .person_or_org
        .identifiers
        .is_none());
}
