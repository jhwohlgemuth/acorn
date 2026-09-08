#![allow(clippy::indexing_slicing, clippy::unwrap_used)]
mod finalization;

use super::*;
use crate::io::api::osti::SearchResults;
use crate::io::api::{orcid, osti};
use crate::io::read_file;
use acorn_schema::discovery::{Person, ProjectOrganization};
use std::fs::remove_file;

#[tokio::test]
async fn test_analysis_materializes_source_content_without_discoveries() {
    let sources = vec![SourceDocument::init()
        .content("plain text without identifiers")
        .format("text")
        .source("<text:1>")
        .build()];
    let discoveries = Records::from(sources);
    let database_path = None;
    let filter = None;
    let ignore = None;
    let input = Vec::new();
    let output = None;
    let standard = Some(Standard::Text);
    let text = Vec::new();
    let retained_sources: &[SourceDocument] = (&discoveries).into();
    assert_eq!(retained_sources.len(), 1);
    let options = Options {
        citation_format: CitationFormat::default(),
        database_path: &database_path,
        enrich: false,
        filter: &filter,
        format: Some(OutputFormat::Json),
        ignore: &ignore,
        input: &input,
        max_depth: None,
        merge_request: false,
        no_local_database: true,
        offline: true,
        watching: false,
        output: &output,
        resolve: false,
        standard: &standard,
        text: &text,
        quiet: true,
        terse: false,
        verbosity: None,
        remote: None,
    };
    assert_eq!(options.roots(), vec![PathBuf::from(".")]);
    let (_, paths) = discoveries.analyze(options).await;
    assert!(discoveries.is_empty());
    assert_eq!(paths.len(), 1);
    let path = paths.first().expect("one source should produce one materialized path");
    assert_eq!(read_file(path.clone()).unwrap_or_default(), "plain text without identifiers");
    let _ = remove_file(path);
}
#[test]
fn test_arxiv_discovery_preserves_versions_and_uses_work_identity() {
    let identifiers = Identifier::find_all("arXiv:2106.09685v1 https://arxiv.org/abs/2106.09685v2");
    assert_eq!(identifiers.len(), 2);
    assert!(identifiers.iter().all(|identifier| identifier.kind == PID::Arxiv));
    let candidate = ArtifactCandidate {
        identifiers,
        ..ArtifactCandidate::default()
    };
    assert_eq!(candidate.identity_keys(), vec!["arxiv:2106.09685"]);
}
#[test]
fn test_candidate_enrichment_normalizes_controlled_vocabularies() {
    assert_eq!(
        ControlledVocabulary::normalize("keywords", ["AI".to_string(), "unknown-value".to_string()]).into_values(),
        ["artificial-intelligence"]
    );
    assert_eq!(
        ControlledVocabulary::normalize("technology", ["Rust".to_string(), "Python".to_string()]).into_values(),
        ["python", "rust"]
    );
}
#[test]
fn test_candidate_urls_include_repository_websites() {
    let websites = vec![Candidate::Website {
        description: "Source repository".to_string(),
        url: "https://github.com/example/project".to_string(),
    }];
    let urls = candidate_urls(Some("https://example.org/project"), &websites);
    assert_eq!(urls, ["https://example.org/project", "https://github.com/example/project"]);
    assert_eq!(
        candidate_repositories(Some("https://example.org/project"), &websites, "gitlab.com")
            .first()
            .and_then(Repository::project_path),
        Some("example/project".to_string())
    );
}
#[test]
fn test_citation_format_parses_supported_values() {
    assert_eq!(CitationFormat::from("APA"), CitationFormat::Apa);
    assert_eq!(CitationFormat::from("unknown"), CitationFormat::Ieee);
    assert_eq!(CitationFormat::Ieee.to_string(), "ieee");
    assert_eq!(CitationFormat::Apa.to_string(), "apa");
    assert_eq!("APA".parse(), Ok(CitationFormat::Apa));
    assert_eq!("chicago".parse(), Ok(CitationFormat::Chicago));
    assert_eq!("harvard".parse(), Ok(CitationFormat::Harvard));
    assert_eq!("ieee".parse(), Ok(CitationFormat::Ieee));
    assert_eq!("mla".parse(), Ok(CitationFormat::Mla));
    assert_eq!("vancouver".parse(), Ok(CitationFormat::Vancouver));
    assert!("unknown".parse::<CitationFormat>().is_err());
}
#[test]
fn test_citation_format_resolves_cli_precedence() {
    assert_eq!(CitationFormat::resolve(true, true, Some(CitationFormat::Apa)), CitationFormat::Apa);
    assert_eq!(CitationFormat::resolve(true, false, None), CitationFormat::Ieee);
    assert_eq!(CitationFormat::resolve(false, true, None), CitationFormat::Ieee);
    assert_eq!(CitationFormat::resolve(false, false, Some(CitationFormat::Chicago)), CitationFormat::Ieee);
}
#[test]
fn test_citation_format_selects_and_flattens_requested_style() {
    let citations = api::citeas::Citations {
        citations: vec![
            api::citeas::Citation {
                text: "APA citation".to_string(),
                style_fullname: "American Psychological Association".to_string(),
                style_shortname: "APA".to_string(),
            },
            api::citeas::Citation {
                text: "IEEE\n citation\ttext".to_string(),
                style_fullname: "Institute of Electrical and Electronics Engineers".to_string(),
                style_shortname: "IEEE".to_string(),
            },
        ],
        ..api::citeas::Citations::default()
    };
    assert_eq!(CitationFormat::Ieee.citation(&citations).as_deref(), Some("IEEE citation text"));
    assert_eq!(CitationFormat::Apa.citation(&citations).as_deref(), Some("APA citation"));
    assert!(CitationFormat::Mla.citation(&citations).is_none());
}
#[test]
fn test_discovers_all_persistent_identifier_types() {
    let content = "ark:/12345/abc doi:10.1234/example hdl:10013/epic.10033 ISBN 978-0-306-40627-0 https://isni.org/isni/0000000492299539 https://orcid.org/0000-0002-2057-9115 US1234567B2 RAID:https://raid.org/10.83962/fb5be317 https://ror.org/01qz5mb56 RRID:SCR_003512";
    let kinds = Identifier::find_all(content)
        .into_iter()
        .map(|identifier| identifier.kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&PID::ARK));
    assert!(kinds.contains(&PID::DOI));
    assert!(kinds.contains(&PID::Handle));
    assert!(kinds.contains(&PID::ISBN));
    assert!(kinds.contains(&PID::ISNI));
    assert!(kinds.contains(&PID::ORCID));
    assert!(kinds.contains(&PID::Patent));
    assert!(kinds.contains(&PID::RAID));
    assert!(kinds.contains(&PID::ROR));
    assert!(kinds.contains(&PID::RRID));
}
#[test]
fn test_discovers_supported_identifiers_without_duplicates() {
    let identifiers = discover_identifiers("Results: doi:10.1234/example and https://doi.org/10.1234/example plus https://example.org/artifact");
    assert_eq!(identifiers.len(), 2);
    assert_eq!(identifiers[0].kind, PID::DOI);
    assert_eq!(identifiers[1].kind, PID::URL);
}
#[test]
fn test_discovery_analysis_checks_failed_resolution() {
    let discoveries = Records(
        vec![Record::Discovery {
            identifier: "10.1234/example".to_string(),
            identifier_type: PID::DOI,
            metadata: Some("resolution error".to_string()),
            resolution_status: ResolutionStatus::Failed,
            source: "example.md".to_string(),
            source_format: "markdown".to_string(),
        }],
        Vec::new(),
    );
    let checks = discoveries.link_checks();
    assert_eq!(checks.len(), 2);
    assert_eq!(checks.as_slice()[0].message, "found");
    assert_eq!(checks.as_slice()[1].category, CheckCategory::Link);
    assert_eq!(checks.as_slice()[1].message, "failed");
    assert_eq!(checks.as_slice()[1].context.as_deref(), Some("resolution error"));
}
#[test]
fn test_exact_grouping_does_not_use_matching_metadata() {
    let candidates = vec![
        ArtifactCandidate {
            identifiers: vec![Identifier::new("doi:10.1234/one").normalized().unwrap()],
            title: Some("Same title".to_string()),
            authors: vec!["Same author".to_string()],
            ..ArtifactCandidate::default()
        },
        ArtifactCandidate {
            identifiers: vec![Identifier::new("doi:10.1234/two").normalized().unwrap()],
            title: Some("Same title".to_string()),
            authors: vec!["Same author".to_string()],
            ..ArtifactCandidate::default()
        },
    ];
    assert_eq!(group_artifacts(candidates.clone()).len(), 1);
    let records = Records(
        candidates
            .into_iter()
            .flat_map(|candidate| candidate.identifiers)
            .map(Record::from)
            .collect(),
        Vec::new(),
    );
    assert_eq!(records.candidates().len(), 2);
}
#[test]
fn test_explicit_raid_suppresses_duplicate_doi() {
    let identifiers = Identifier::find_all("RAID:https://raid.org/10.83962/fb5be317");
    assert_eq!(identifiers.iter().filter(|identifier| identifier.kind == PID::RAID).count(), 1);
    assert_eq!(identifiers.iter().filter(|identifier| identifier.kind == PID::DOI).count(), 0);
}
#[test]
fn test_gather_pdf_discovers_only_valid_identifiers() {
    let document = SourceDocument::from_path("../../tests/fixtures/gather/gather.pdf").unwrap();
    let identifiers = Identifier::find_all(&document.content)
        .into_iter()
        .filter(|identifier| identifier.kind != PID::URL)
        .map(|identifier| (identifier.kind, identifier.value))
        .collect::<Vec<_>>();
    assert_eq!(
        identifiers,
        vec![
            (PID::Arxiv, "arXiv:2605.10913v3".to_string()),
            (PID::Arxiv, "arXiv:2507.19457".to_string()),
            (PID::Arxiv, "arXiv:2510.02453".to_string()),
            (PID::Arxiv, "arXiv:2503.13657".to_string()),
            (PID::Arxiv, "arXiv:2406.16218".to_string()),
            (PID::Arxiv, "arXiv:2601.16443".to_string()),
            (PID::Arxiv, "arXiv:2308.00352".to_string()),
            (PID::Arxiv, "arXiv:2011.03088".to_string()),
            (PID::Arxiv, "arXiv:2310.03714".to_string()),
            (PID::Arxiv, "arXiv:2601.13295".to_string()),
            (PID::Arxiv, "arXiv:2603.28052".to_string()),
            (PID::Arxiv, "arXiv:2604.04247".to_string()),
            (PID::Arxiv, "arXiv:2511.00628".to_string()),
            (PID::Arxiv, "arXiv:2510.26585".to_string()),
            (PID::Arxiv, "arXiv:2303.11366".to_string()),
            (PID::Arxiv, "arXiv:2602.08199".to_string()),
            (PID::Arxiv, "arXiv:2511.03690".to_string()),
            (PID::Arxiv, "arXiv:2308.08155".to_string()),
            (PID::Arxiv, "arXiv:2512.24601".to_string()),
            (PID::Arxiv, "arXiv:2603.19461".to_string()),
            (PID::Arxiv, "arXiv:2510.01171".to_string()),
            (PID::Arxiv, "arXiv:2310.04406".to_string()),
            (PID::DOI, "10.1145/73560.73576".to_string()),
            (PID::DOI, "10.1023/A:1023064908962".to_string()),
            (PID::DOI, "10.1145/3371119".to_string()),
            (PID::DOI, "10.1007/s10462-022-10228-y".to_string()),
            (PID::ISBN, "978-0-521-66350-2".to_string()),
            (PID::ISBN, "978-3-642-00590-9".to_string()),
        ]
    );
}
#[test]
fn test_groups_only_proven_equivalent_candidates() {
    let doi = Identifier::new("doi:10.1234/abc").normalized().unwrap();
    let url = Identifier::new("https://example.org/artifact").normalized().unwrap();
    let candidates = vec![
        ArtifactCandidate {
            identifiers: vec![doi.clone()],
            title: Some("A Result".to_string()),
            authors: vec!["Alice Example".to_string()],
            ..ArtifactCandidate::default()
        },
        ArtifactCandidate {
            identifiers: vec![doi, url],
            ..ArtifactCandidate::default()
        },
        ArtifactCandidate {
            identifiers: vec![Identifier::new("doi:10.9999/other").normalized().unwrap()],
            title: Some("A Different Result".to_string()),
            authors: vec!["Alice Example".to_string()],
            ..ArtifactCandidate::default()
        },
    ];
    let grouped = group_artifacts(candidates);
    assert_eq!(grouped.len(), 2);
    assert_eq!(grouped[0].identifiers.len(), 2);
}
#[test]
fn test_handle_discovery_is_conservative_and_supports_same_source_candidate() {
    assert!(Identifier::find_all("Read docs/index.md and output/report.json").is_empty());
    let metadata = serde_json::json!({
        "responseCode": 1,
        "handle": "10013/epic.10033",
        "values": [{"index": 1, "type": "URL", "data": {"format": "string", "value": "https://example.org/project"}}]
    });
    let records = Records(
        vec![
            Record::Discovery {
                identifier: "10.1234/example".to_string(),
                identifier_type: PID::DOI,
                metadata: None,
                resolution_status: ResolutionStatus::NotRequested,
                source: "same.md".to_string(),
                source_format: "markdown".to_string(),
            },
            Record::Discovery {
                identifier: "10013/epic.10033".to_string(),
                identifier_type: PID::Handle,
                metadata: Some(metadata.to_string()),
                resolution_status: ResolutionStatus::Resolved,
                source: "same.md".to_string(),
                source_format: "markdown".to_string(),
            },
            Record::Discovery {
                identifier: "20.5000/unrelated".to_string(),
                identifier_type: PID::Handle,
                metadata: None,
                resolution_status: ResolutionStatus::NotRequested,
                source: "other.md".to_string(),
                source_format: "markdown".to_string(),
            },
        ],
        Vec::new(),
    );
    let candidates = records.candidates();
    assert_eq!(candidates.len(), 1, "Handles must not independently create project candidates");
    let candidate = candidates.first().expect("DOI should create a project candidate");
    assert!(candidate.identifiers.iter().any(|identifier| identifier.kind == PID::Handle));
    assert!(!candidate.identifiers.iter().any(|identifier| identifier.value == "20.5000/unrelated"));
    assert!(candidate
        .websites
        .iter()
        .any(|website| website.url() == Some("https://example.org/project")));
    assert_eq!(candidate.to_partial_rad_json()["meta"]["handle"], serde_json::json!(["10013/epic.10033"]));
}
#[test]
fn test_handle_is_not_associated_with_ambiguous_same_source_candidates() {
    let records = Records(
        vec![("10.1234/one", PID::DOI), ("10.1234/two", PID::DOI), ("10013/epic.10033", PID::Handle)]
            .into_iter()
            .map(|(identifier, identifier_type)| Record::Discovery {
                identifier: identifier.to_string(),
                identifier_type,
                metadata: None,
                resolution_status: ResolutionStatus::NotRequested,
                source: "same.md".to_string(),
                source_format: "markdown".to_string(),
            })
            .collect(),
        Vec::new(),
    );
    let candidates = records.candidates();
    assert_eq!(candidates.len(), 2);
    assert!(candidates
        .iter()
        .all(|candidate| candidate.identifiers.iter().all(|identifier| identifier.kind != PID::Handle)));
}
#[test]
fn test_identifier_find_all_extracts_document_urls_and_deduplicates() {
    let content = "[Docs](https://example.org/path), https://example.org/path and https://example.net/end.";
    let urls = Identifier::find_all(content)
        .into_iter()
        .filter_map(|identifier| (identifier.kind == PID::URL).then_some(identifier.value))
        .collect::<Vec<_>>();
    assert_eq!(urls, vec!["https://example.org/path", "https://example.net/end"]);
}
#[test]
fn test_identifier_hash_is_stable_and_short() {
    let identifier = Identifier::new("doi:10.1234/abc").normalized().unwrap();
    assert_eq!(identifier.identifier_hash().len(), 12);
    assert_eq!(identifier.identifier_hash(), identifier.identifier_hash());
}
#[test]
fn test_isni_url_is_not_discovered_as_orcid() {
    let identifiers = Identifier::find_all("https://isni.org/isni/0000000492299539");
    assert_eq!(identifiers.iter().filter(|identifier| identifier.kind == PID::ISNI).count(), 1);
    assert_eq!(identifiers.iter().filter(|identifier| identifier.kind == PID::ORCID).count(), 0);
}
#[test]
fn test_markdown_report_contains_checks_and_summary() {
    let report = Report {
        checks: vec![Record::Check {
            category: CheckCategory::Schema,
            locator: None,
            message: "example".to_string(),
            severity: CheckSeverity::Error,
            success: false,
            uri: None,
        }],
        discoveries: Records(
            vec![Record::Discovery {
                identifier: "10.1234/example".to_string(),
                identifier_type: PID::DOI,
                metadata: None,
                resolution_status: ResolutionStatus::NotRequested,
                source: "example.md".to_string(),
                source_format: "markdown".to_string(),
            }],
            Vec::new(),
        ),
        remote: Vec::new(),
        candidates: Vec::new(),
        summary: Summary {
            discoveries: 1,
            failures: 1,
            inputs: 1,
            matches: 0,
            candidates: PersistenceCounts::default(),
        },
        citation_format: CitationFormat::default(),
    };
    let markdown = report.serialize(OutputFormat::Markdown).unwrap_or_default();
    assert!(markdown.contains("## Summary"));
    assert!(markdown.contains("## Checks"));
    assert!(markdown.contains("- **doi** `10.1234/example` (example.md)"));
    assert!(markdown.contains("- **error** schema: example"));
}
#[test]
fn test_orcid_profile_enriches_person_match() {
    let person = Person {
        name: "Existing Name".to_string(),
        orcid: Some("0000-0002-2057-9115".to_string()),
        email: None,
        affiliations: vec!["Existing Laboratory".to_string()],
        roles: vec!["developer".to_string()],
        project_ids: vec![123],
        project_titles: vec!["Project".to_string()],
    };
    let value = RemoteMatch::init()
        .entity(RemoteEntity::Person)
        .identifier("0000-0002-2057-9115")
        .title(person.name.clone())
        .maybe_pid(person.orcid.clone())
        .metadata(serde_json::to_value(person).unwrap())
        .build();
    let enriched = value.with_orcid(orcid::SearchResult {
        orcid_id: Some("0000-0002-2057-9115".to_string()),
        given_names: Some("Profile".to_string()),
        family_names: Some("Name".to_string()),
        credit_name: Some("Preferred Name".to_string()),
        emails: Some(vec!["person@example.com".to_string()]),
        institution_names: Some(vec!["ORCID Laboratory".to_string(), "Existing Laboratory".to_string()]),
        other_name: None,
    });
    let metadata: Person = serde_json::from_value(enriched.metadata).unwrap();
    assert_eq!(enriched.title, "Preferred Name");
    assert_eq!(metadata.email.as_deref(), Some("person@example.com"));
    assert_eq!(metadata.affiliations, vec!["Existing Laboratory", "ORCID Laboratory"]);
    assert_eq!(metadata.roles, vec!["developer"]);
    assert_eq!(metadata.project_ids, vec![123]);
}
#[test]
fn test_orcid_raw_resolution_prefers_names_then_credit_name() {
    let response = |given_names: Option<&str>, family_names: Option<&str>, credit_name: Option<&str>| {
        serde_json::to_value(orcid::SearchResponse::from(orcid::SearchResult {
            orcid_id: Some("0000-0002-2057-9115".to_string()),
            given_names: given_names.map(str::to_string),
            family_names: family_names.map(str::to_string),
            credit_name: credit_name.map(str::to_string),
            emails: None,
            institution_names: None,
            other_name: None,
        }))
        .unwrap()
    };
    let identifier = "https://orcid.org/0000-0002-2057-9115";
    assert_eq!(
        resolved_pid_output(
            identifier,
            PID::ORCID,
            &response(Some("Jason"), Some("Wohlgemuth"), Some("J. Wohlgemuth")),
            CitationFormat::Ieee
        )
        .unwrap(),
        "Jason Wohlgemuth (https://orcid.org/0000-0002-2057-9115)"
    );
    assert_eq!(
        resolved_pid_output(
            identifier,
            PID::ORCID,
            &response(None, Some("Wohlgemuth"), Some("Jason W.")),
            CitationFormat::Ieee
        )
        .unwrap(),
        "Jason W. (https://orcid.org/0000-0002-2057-9115)"
    );
    assert!(resolved_pid_output(identifier, PID::ORCID, &response(None, None, None), CitationFormat::Ieee).is_err());
}
#[test]
fn test_osti_project_maps_candidate_metadata() {
    let project = osti::Project {
        code_id: 42,
        software_title: "Example software".to_string(),
        doi: Some("10.1234/example".to_string()),
        description: Some("A complete project description".to_string()),
        repository_link: Some("https://github.com/example/project".to_string()),
        landing_page: Some("https://example.org/project".to_string()),
        research_organizations: vec![ProjectOrganization {
            organization_name: "Research Laboratory".to_string(),
            ..ProjectOrganization::default()
        }],
        sponsoring_organizations: vec![ProjectOrganization {
            organization_name: "US Department of Energy".to_string(),
            ..ProjectOrganization::default()
        }],
        links: vec![osti::Link {
            rel: "documentation".to_string(),
            href: "https://example.org/docs".to_string(),
        }],
        ..osti::Project::default()
    };
    let response = RemoteSearchResponse::from_osti(osti::SearchResponse {
        project_total: 1,
        offset: 0,
        has_more: false,
        results: SearchResults::Projects(vec![project]),
    })
    .unwrap();
    let candidate = response.matches.first().expect("OSTI project response should contain one match");
    assert_eq!(candidate.description.as_deref(), Some("A complete project description"));
    assert_eq!(candidate.sponsors, ["US Department of Energy"]);
    assert_eq!(candidate.partners, ["Research Laboratory"]);
    assert_eq!(candidate.websites.len(), 3);
    assert!(candidate.resolution.is_none());
    assert!(candidate.technology.is_empty());
}
#[test]
fn test_partial_rad_mapping_preserves_supported_identifiers() {
    let identifiers = [
        "arXiv:2106.09685",
        "doi:10.1234/example",
        "ISBN 978-0-306-40627-0",
        "US1234567B2",
        "swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a",
    ]
    .into_iter()
    .filter_map(|value| Identifier::new(value).normalized())
    .collect();
    let candidate = ArtifactCandidate {
        identifiers,
        canonical_url: Some("https://example.org/project".to_string()),
        title: Some("Example project".to_string()),
        description: Some("Full provider description".to_string()),
        websites: vec![Candidate::Website {
            description: "Documentation".to_string(),
            url: "https://example.org/docs".to_string(),
        }],
        keywords: vec!["AI".to_string()],
        sponsors: vec!["US Department of Energy".to_string()],
        partners: vec!["Oak Ridge National Laboratory".to_string()],
        related: vec!["https://raid.org/10.99999/example".to_string()],
        technology: vec!["Python".to_string(), "Rust".to_string()],
        ..ArtifactCandidate::default()
    };
    let rad = candidate.to_partial_rad_json();
    assert_eq!(rad["title"], "Example project");
    assert_eq!(rad["notes"], "Full provider description");
    assert_eq!(rad["meta"]["doi"], serde_json::json!(["arXiv:2106.09685", "10.1234/example"]));
    assert_eq!(
        rad["meta"]["swhid"],
        serde_json::json!(["swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a"])
    );
    assert_eq!(rad["meta"]["websites"][0]["url"], "https://example.org/project");
    assert_eq!(rad["meta"]["websites"][1]["url"], "https://example.org/docs");
    assert_eq!(rad["meta"]["keywords"], serde_json::json!(["artificial-intelligence"]));
    assert_eq!(rad["meta"]["sponsors"], serde_json::json!(["US Department of Energy"]));
    assert_eq!(rad["meta"]["partners"], serde_json::json!(["Oak Ridge National Laboratory"]));
    assert_eq!(rad["meta"]["related"], serde_json::json!(["https://raid.org/10.99999/example"]));
    assert_eq!(rad["meta"]["technology"], serde_json::json!(["python", "rust"]));
}
#[test]
fn test_project_identity_excludes_people_organizations_and_urls() {
    let candidate = ArtifactCandidate {
        identifiers: [
            "doi:10.1234/example",
            "https://orcid.org/0000-0002-2057-9115",
            "https://ror.org/01qz5mb56",
            "https://example.org/project",
        ]
        .into_iter()
        .filter_map(|value| Identifier::new(value).normalized())
        .collect(),
        ..ArtifactCandidate::default()
    };
    assert_eq!(candidate.identity_keys(), vec!["doi:10.1234/example"]);
    assert!(ResearchActivityCandidate::try_from(candidate).is_ok());
    let entity_only = ArtifactCandidate {
        identifiers: ["https://orcid.org/0000-0002-2057-9115", "https://ror.org/01qz5mb56"]
            .into_iter()
            .filter_map(|value| Identifier::new(value).normalized())
            .collect(),
        ..ArtifactCandidate::default()
    };
    assert!(ResearchActivityCandidate::try_from(entity_only).is_err());
}
#[test]
fn test_raid_resolution_maps_typed_candidate_metadata() {
    let metadata = serde_json::json!([{
        "title": [{
            "text": "RAiD project",
            "type": {"id": "https://vocabulary.raid.org/title.type.schema/5", "schemaUri": "https://vocabulary.raid.org/title.type.schema/376"}
        }],
        "description": [{
            "text": "RAiD project description",
            "type": {"id": "https://vocabulary.raid.org/description.type.schema/318", "schemaUri": "https://vocabulary.raid.org/description.type.schema/320"}
        }],
        "alternateIdentifier": [{"id": "10.1234/example", "type": "DOI"}],
        "alternateUrl": [{"url": "https://example.org/raid-project"}],
        "organisation": [
            {
                "id": "https://ror.org/01qz5mb56",
                "role": [{"id": "https://vocabulary.raid.org/organisation.role.schema/182"}]
            },
            {
                "id": "https://ror.org/03q1rgc19",
                "role": [{"id": "https://vocabulary.raid.org/organisation.role.schema/186"}]
            }
        ],
        "relatedRaid": [{
            "id": "https://raid.org/10.99999/related",
            "type": {"id": "https://vocabulary.raid.org/relatedRaid.type.schema/204"}
        }],
        "contributor": [
            {
                "id": "https://orcid.org/0000-0002-1825-0097",
                "position": [],
                "leader": false,
                "contact": false
            },
            {
                "id": "https://orcid.org/0000-0002-2057-9115",
                "position": [],
                "leader": false,
                "contact": true,
                "email": "contact@example.org"
            }
        ],
        "subject": [{"id": "subject", "keyword": [{"text": "AI"}]}]
    }]);
    let candidate = ArtifactCandidate {
        identifiers: vec![Identifier::new("https://raid.org/10.83962/fb5be317").normalized().unwrap()],
        ..ArtifactCandidate::default()
    };
    let resolved = candidate.resolved(Some(&metadata.to_string()), &PID::RAID);
    let rad = resolved.to_partial_rad_json();
    assert_eq!(rad["title"], "RAiD project");
    assert_eq!(rad["notes"], "RAiD project description");
    assert_eq!(rad["meta"]["doi"], serde_json::json!(["10.1234/example"]));
    assert_eq!(
        rad["meta"]["ror"],
        serde_json::json!(["https://ror.org/01qz5mb56", "https://ror.org/03q1rgc19"])
    );
    assert_eq!(rad["meta"]["websites"][0]["url"], "https://example.org/raid-project");
    assert_eq!(rad["meta"]["keywords"], serde_json::json!(["artificial-intelligence"]));
    assert_eq!(rad["meta"]["sponsors"], serde_json::json!(["Advanced Research Projects Agency-Energy"]));
    assert_eq!(rad["meta"]["partners"], serde_json::json!(["Oak Ridge National Laboratory"]));
    assert_eq!(rad["meta"]["related"], serde_json::json!(["10.99999/related"]));
    assert_eq!(rad["contact"]["identifier"], "https://orcid.org/0000-0002-2057-9115");
    assert_eq!(rad["contact"]["email"], "contact@example.org");
}
#[test]
fn test_remote_match_analysis_checks_failed_resolution() {
    let value = RemoteMatch::init()
        .entity(RemoteEntity::Project)
        .identifier("123")
        .title("Project")
        .pid("10.1234/example")
        .metadata(serde_json::Value::Null)
        .resolution(
            RemoteResolution::init()
                .provider("citeas")
                .status(ResolutionStatus::Failed)
                .error("resolution error")
                .build(),
        )
        .build();
    let checks = value.link_checks();
    assert_eq!(checks.len(), 2);
    assert_eq!(checks.as_slice()[0].message, "found");
    assert_eq!(checks.as_slice()[1].message, "failed");
    assert_eq!(checks.as_slice()[1].context.as_deref(), Some("resolution error"));
}
#[test]
fn test_remote_persistence_keeps_non_projects_report_only() {
    let path = temp_dir().join(format!("acorn-remote-candidates-{}.db", nanoid::nanoid!()));
    let database_path = Some(path.clone());
    let response = RemoteSearchResponse {
        provider: RemoteProvider::Osti,
        total: 3,
        offset: 0,
        has_more: false,
        matches: vec![
            RemoteMatch::init()
                .entity(RemoteEntity::Project)
                .identifier("123")
                .title("Project")
                .pid("10.1234/example")
                .url("https://example.org/project")
                .metadata(serde_json::json!({"code_id": 123}))
                .description("A provider description")
                .websites(vec![Candidate::Website {
                    description: "Documentation".to_string(),
                    url: "https://example.org/docs".to_string(),
                }])
                .keywords(vec!["AI".to_string()])
                .sponsors(vec!["US Department of Energy".to_string()])
                .partners(vec!["Oak Ridge National Laboratory".to_string()])
                .related(vec!["https://raid.org/10.99999/example".to_string()])
                .technology(vec!["Rust".to_string()])
                .build(),
            RemoteMatch::init()
                .entity(RemoteEntity::Person)
                .identifier("person")
                .title("Person")
                .pid("0000-0002-2057-9115")
                .metadata(serde_json::json!({}))
                .build(),
            RemoteMatch::init()
                .entity(RemoteEntity::Organization)
                .identifier("org")
                .title("Organization")
                .metadata(serde_json::json!({}))
                .build(),
        ],
        resolution_checks: Default::default(),
    };
    let database = Database::<Table>::from_path(database_path);
    let results = response.clone().persist(&database, None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].action, CandidateAction::Created);
    let enriched = response.persist(&database, Some("01qz5mb56")).unwrap();
    assert_eq!(enriched[0].action, CandidateAction::Enriched);
    let rows = database.research_activities_by_identity(&["osti-project:123".to_string()]).unwrap();
    assert_eq!(rows.len(), 1);
    let rad: serde_json::Value = serde_json::from_str(rows[0].rad_json.as_deref().unwrap()).unwrap();
    assert_eq!(rad["notes"], "A provider description");
    assert_eq!(rad["meta"]["ror"], serde_json::json!(["https://ror.org/01qz5mb56"]));
    assert_eq!(rad["meta"]["websites"][1]["url"], "https://example.org/docs");
    assert_eq!(rad["meta"]["keywords"], serde_json::json!(["artificial-intelligence"]));
    assert_eq!(rad["meta"]["sponsors"], serde_json::json!(["US Department of Energy"]));
    assert_eq!(rad["meta"]["partners"], serde_json::json!(["Oak Ridge National Laboratory"]));
    assert_eq!(rad["meta"]["related"], serde_json::json!(["https://raid.org/10.99999/example"]));
    assert_eq!(rad["meta"]["technology"], serde_json::json!(["rust"]));
    let provenance: serde_json::Value = serde_json::from_str(rows[0].provenance_json.as_deref().unwrap()).unwrap();
    assert_eq!(provenance[0]["provider"], "osti");
    assert_eq!(provenance[0]["provider_identifier"], "123");
    assert_eq!(provenance[0]["entity"], "project");
    assert_eq!(provenance[0]["metadata"]["code_id"], 123);
}
#[test]
fn test_remote_request_converts_to_osti_options() {
    let request = RemoteSearchRequest::init()
        .provider(RemoteProvider::Osti)
        .entity(RemoteEntity::Organization)
        .organization_role(acorn_schema::discovery::RemoteOrganizationRole::Research)
        .limit(50)
        .offset(25)
        .all(true)
        .build();
    let options = osti::Options::from(request).with_query("ORNL");
    assert_eq!(options.query, "");
    assert_eq!(options.view, osti::SearchView::Organizations);
    assert_eq!(options.organization.as_deref(), Some("ORNL"));
    assert_eq!(options.organization_role, acorn_schema::discovery::RemoteOrganizationRole::Research);
    assert_eq!(options.limit, 50);
    assert_eq!(options.start, 25);
    assert!(options.all);
}
#[test]
fn test_remote_responses_merge_duplicate_provider_identifiers() {
    let response = |identifier: &str| RemoteSearchResponse {
        provider: RemoteProvider::Osti,
        total: 1,
        offset: 0,
        has_more: false,
        matches: vec![RemoteMatch::init()
            .entity(RemoteEntity::Project)
            .identifier(identifier)
            .title("Example")
            .metadata(serde_json::Value::Null)
            .build()],
        resolution_checks: Default::default(),
    };
    let merged = vec![response("1"), response("1"), response("2")]
        .into_iter()
        .reduce(RemoteSearchResponse::merge)
        .into_iter()
        .collect::<Vec<_>>();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].matches.len(), 2);
    assert_eq!(response("1").merge(response("2")).matches.len(), 2);
}
#[test]
fn test_remote_search_request_builder_and_capabilities() {
    let request = RemoteSearchRequest::init()
        .provider(RemoteProvider::Osti)
        .entity(RemoteEntity::Project)
        .build();
    assert!(request.is_empty());
    assert!(request.supports_entity());
    assert_eq!(request.limit, 20);
    let unsupported = RemoteSearchRequest::init()
        .provider(RemoteProvider::Osti)
        .entity(RemoteEntity::Repository)
        .build();
    assert!(!unsupported.supports_entity());
}
#[test]
fn test_remote_types_convert_to_osti_types() {
    assert_eq!(osti::SearchView::from(RemoteEntity::Person), osti::SearchView::People);
}
#[test]
fn test_report_table_contains_only_discoveries() {
    let report = Report {
        checks: vec![Record::Check {
            category: CheckCategory::Schema,
            locator: None,
            message: "example".to_string(),
            severity: CheckSeverity::Error,
            success: false,
            uri: None,
        }],
        discoveries: Records(
            vec![Record::Discovery {
                identifier: "10.1234/example".to_string(),
                identifier_type: PID::DOI,
                metadata: None,
                resolution_status: ResolutionStatus::NotRequested,
                source: "example.md".to_string(),
                source_format: "markdown".to_string(),
            }],
            Vec::new(),
        ),
        remote: Vec::new(),
        candidates: Vec::new(),
        summary: Summary {
            discoveries: 1,
            failures: 1,
            inputs: 1,
            matches: 0,
            candidates: PersistenceCounts::default(),
        },
        citation_format: CitationFormat::default(),
    };
    let (headers, rows) = report.table();
    assert_eq!(headers, vec!["Type", "Value", "Status", "Source"]);
    assert_eq!(rows.len(), 1);
    let console = report.serialize(OutputFormat::Console).unwrap_or_default();
    assert!(console.contains("10.1234/example"));
    assert!(!console.contains("schema: example"));
}
#[test]
fn test_resolution_outcome_projects_typed_status_and_payload() {
    assert_eq!(ResolutionOutcome::<String>::NotRequested.status(), ResolutionStatus::NotRequested);
    assert_eq!(ResolutionOutcome::<String>::Unsupported.status(), ResolutionStatus::Unsupported);
    assert_eq!(
        ResolutionOutcome::Resolved("metadata".to_string()).into_parts(),
        (Some("metadata".to_string()), ResolutionStatus::Resolved)
    );
    assert_eq!(
        ResolutionOutcome::Failed("diagnostic".to_string()).into_parts(),
        (Some("diagnostic".to_string()), ResolutionStatus::Failed)
    );
}
#[test]
fn test_resolution_state_checks_preserve_nonfatal_warning_states() {
    for state in [LifecycleState::Unsupported, LifecycleState::Conflict] {
        let check = state.check("10.1234/example".to_string(), None, None);
        assert!(check.success);
        assert_eq!(check.category, CheckCategory::Link);
        assert_eq!(check.severity, CheckSeverity::Warning);
        assert_eq!(check.message, state.to_string());
    }
}
#[test]
fn test_resolved_raw_report_renders_citations_and_orcid_names() {
    let citations = api::citeas::Citations {
        citations: vec![api::citeas::Citation {
            text: "IEEE citation".to_string(),
            style_fullname: "Institute of Electrical and Electronics Engineers".to_string(),
            style_shortname: "IEEE".to_string(),
        }],
        ..api::citeas::Citations::default()
    };
    let profile = orcid::SearchResponse::from(orcid::SearchResult {
        orcid_id: Some("0000-0002-2057-9115".to_string()),
        given_names: Some("Jason".to_string()),
        family_names: Some("Wohlgemuth".to_string()),
        credit_name: None,
        emails: None,
        institution_names: None,
        other_name: None,
    });
    let discovery = |identifier: &str, identifier_type: PID, metadata: String| Record::Discovery {
        identifier: identifier.to_string(),
        identifier_type,
        metadata: Some(metadata),
        resolution_status: ResolutionStatus::Resolved,
        source: "example.md".to_string(),
        source_format: "markdown".to_string(),
    };
    let records = Records(
        vec![
            discovery("arXiv:2106.09685", PID::Arxiv, serde_json::to_string(&citations).unwrap()),
            discovery("10.1234/example", PID::DOI, serde_json::to_string(&citations).unwrap()),
            discovery(
                "https://orcid.org/0000-0002-2057-9115",
                PID::ORCID,
                serde_json::to_string(&profile).unwrap(),
            ),
        ],
        Vec::new(),
    );
    let output = Report::new(&[], records, 1).serialize(OutputFormat::Raw).unwrap();
    assert_eq!(
        output,
        "IEEE citation\nIEEE citation\nJason Wohlgemuth (https://orcid.org/0000-0002-2057-9115)"
    );
}
#[test]
fn test_swhid_discovery_is_not_resolved_without_request() {
    let sources = vec![SourceDocument::init()
        .content("See swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2 for reference")
        .format("text")
        .source("example.md")
        .build()];
    let discoveries = Records::from(sources);
    assert_eq!(discoveries.len(), 1);
    let record = discoveries.candidates().first().cloned().unwrap_or_default();
    assert!(record.identifiers.iter().any(|id| id.kind == PID::SWHID));
    let discoveries = Records::from(vec![SourceDocument::init()
        .content("swh:1:cnt:94a9ed024d3859793618152ea559a168bbcbb5e2")
        .format("text")
        .source("example.md")
        .build()]);
    let json = serde_json::to_value(&discoveries).unwrap();
    let swhid = &json[0];
    assert_eq!(swhid["resolution_status"], "not-requested");
    assert!(swhid["metadata"].is_null());
}
#[test]
fn test_rrid_discovery_is_local_and_not_project_identifying() {
    let sources = vec![SourceDocument::init()
        .content("Use RRID:SCR_003512 or https://n2t.net/RRID:SCR_003512.")
        .format("text")
        .source("example.md")
        .build()];
    let discoveries = Records::from(sources);
    assert_eq!(discoveries.len(), 1);
    assert!(discoveries.candidates().is_empty());
    let json = serde_json::to_value(&discoveries).unwrap();
    assert_eq!(json[0]["identifier"], "RRID:SCR_003512");
    assert_eq!(json[0]["identifier_type"], "rrid");
    assert_eq!(json[0]["resolution_status"], "not-requested");
    assert!(json[0]["metadata"].is_null());
}
#[test]
fn test_synthetic_resume_pdf_exercises_pid_edge_cases() {
    let document = SourceDocument::from_path("../../tests/fixtures/gather/resume.pdf").unwrap();
    let identifiers = Identifier::find_all(&document.content)
        .into_iter()
        .filter(|identifier| identifier.kind != PID::URL)
        .map(|identifier| (identifier.kind, identifier.value))
        .collect::<Vec<_>>();
    let expected = [
        (PID::RAID, "10.99999/example"),
        (PID::ARK, "ark:99166/w66d60p2"),
        (PID::ARK, "https://n2t.net/ark:99166/w66d60p2"),
        (PID::ARK, "ark:99166/W66D60P2"),
        (PID::Arxiv, "arXiv:2608.01234"),
        (PID::DOI, "10.1234/alpha.2026.001"),
        (PID::DOI, "10.1234/beta(2026"),
        (PID::DOI, "10.1234/beta(2026))7"),
        (PID::DOI, "10.1234/CaseSensitive.XyZ;"),
        (PID::DOI, "10.1234/CaseSensitive.XyZ"),
        (PID::DOI, "10.1234/gamma.2025.003"),
        (PID::DOI, "10.1234/gamma.2025.003))"),
        (PID::DOI, "10.97812345/99990"),
        (PID::DOI, "10.1234/software_v2.1;"),
        (PID::DOI, "10.1234/software_v2.1"),
        (PID::DOI, "10.1234/software_v2.1)"),
        (PID::DOI, "10.1234/data-set_04"),
        (PID::DOI, "10.99999/example)"),
        (PID::DOI, "10.1234/beta(2026)7"),
        (PID::DOI, "10.1234/wrapped.004"),
        (PID::DOI, "10.1234/trailing-period.005"),
        (PID::DOI, "10.5555/not-a-doi"),
        (PID::DOI, "10.1234/annotation-target.006"),
        (PID::DOI, "10.1234/list-a"),
        (PID::DOI, "10.1234/list-b;"),
        (PID::DOI, "10.1234/list-c"),
        (PID::DOI, "10.1234/brackets"),
        (PID::DOI, "10.1234/braces"),
        (PID::DOI, "10.1234/parentheses)"),
        (PID::DOI, "10.1234/line-break-"),
        (PID::DOI, "10.1234/not-a-path-boundary"),
        (PID::DOI, "10.1234/not-a-path-boundary)"),
        (PID::ISBN, "978-12-3-4-5"),
        (PID::ISBN, "978-0-306-40627-0"),
        (PID::ISBN, "978-03064-062-7-0"),
        (PID::ORCID, "https://orcid.org/0000-0000-0000-0001"),
        (PID::ORCID, "https://orcid.org/0000-0000-0000-0002"),
        (PID::Patent, "US 1234567 B2"),
        (PID::ROR, "https://ror.org/01qz5mb56"),
    ]
    .into_iter()
    .map(|(kind, value)| (kind, value.to_string()))
    .collect::<Vec<_>>();
    assert_eq!(identifiers, expected);
}
#[test]
fn test_typed_gather_record_preserves_lowercase_wire_values() {
    let report = Report::new(
        &[LifecycleState::Found.check("10.1234/example".to_string(), None, Some("example.md".to_string()))],
        Records(
            vec![Record::Discovery {
                identifier: "10.1234/example".to_string(),
                identifier_type: PID::DOI,
                metadata: None,
                resolution_status: ResolutionStatus::NotRequested,
                source: "example.md".to_string(),
                source_format: "markdown".to_string(),
            }],
            Vec::new(),
        ),
        1,
    );
    let value = serde_json::to_value(report).unwrap();
    assert_eq!(value["checks"][0]["category"], "link");
    assert_eq!(value["checks"][0]["severity"], "info");
    assert_eq!(value["checks"][0]["message"], "found");
    assert_eq!(value["discoveries"][0]["identifier_type"], "doi");
    assert_eq!(value["discoveries"][0]["resolution_status"], "not-requested");
}
