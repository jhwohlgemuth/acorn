use crate::io::api::openalex::{self, Options, Work};
use crate::io::api::Endpoint;
use crate::io::workflow::EnrichmentResult;
use acorn_schema::schema::research_activity::ResearchOutput;
use acorn_schema::validation::Validate;
use color_eyre::eyre::eyre;

fn work() -> Work {
    serde_json::from_value(serde_json::json!({
        "id": "https://openalex.org/W2741809807",
        "doi": "https://doi.org/10.1038/nature12373",
        "title": "Example work",
        "type": "article",
        "publication_date": "2024-01-31",
        "authorships": [{
            "author_position": "first",
            "is_corresponding": true,
            "author": {
                "id": "https://openalex.org/A1",
                "display_name": "Example Researcher",
                "orcid": "https://orcid.org/0000-0002-1825-0097"
            },
            "institutions": [{
                "id": "https://openalex.org/I1",
                "display_name": "Oak Ridge National Laboratory",
                "ror": "https://ror.org/01qz5mb56"
            }]
        }],
        "grants": [{
            "funder": "https://openalex.org/F1",
            "funder_display_name": "Example Funder",
            "award_id": "AWARD-1"
        }],
        "open_access": {
            "is_oa": true,
            "oa_status": "gold",
            "oa_url": "https://example.org/work"
        },
        "primary_location": {
            "landing_page_url": "https://example.org/work",
            "pdf_url": null,
            "license": "cc-by"
        },
        "best_oa_location": {
            "landing_page_url": "https://example.org/work",
            "pdf_url": "https://example.org/work.pdf",
            "license": "cc-by"
        },
        "keywords": [{"display_name": "Research data", "score": 0.8}]
    }))
    .expect("OpenAlex work fixture should deserialize")
}

#[test]
fn test_endpoint_uses_application_configuration() {
    let endpoint = Endpoint::from_template("openalex").expect("OpenAlex endpoint should be configured");
    let resource = endpoint.resources.first().expect("OpenAlex should configure a work resource");
    assert_eq!(endpoint.domain, "api.openalex.org");
    assert_eq!(resource.name, "work");
    assert_eq!(resource.template, "{{ base }}/works/doi:{{ identifier }}{{ query }}");
}

#[test]
fn test_work_maps_to_valid_structured_output() {
    let output = ResearchOutput::from(work());
    assert!(output.validate().is_ok());
    assert_eq!(output.doi.as_deref(), Some("10.1038/nature12373"));
    assert_eq!(output.contributors.as_ref().map(Vec::len), Some(1));
    assert_eq!(output.funding.as_ref().map(Vec::len), Some(1));
    assert_eq!(output.websites.as_ref().map(Vec::len), Some(2));
    assert_eq!(output.keywords, vec!["Research data".to_string()]);
}
#[test]
fn test_work_merges_missing_fields_into_existing_output() {
    let existing = ResearchOutput::init()
        .doi("10.1038/nature12373".to_string())
        .title("Existing title".to_string())
        .build();
    let result = openalex::merge_work_result(
        EnrichmentResult {
            data: vec![existing],
            conflicts: Vec::new(),
            failures: Vec::new(),
        },
        "10.1038/nature12373".to_string(),
        Ok(work()),
    );
    let output = result.data.first().expect("Merged output should be retained");
    assert_eq!(output.title, "Existing title");
    assert_eq!(output.identifier.as_deref(), Some("https://openalex.org/W2741809807"));
    assert_eq!(output.contributors.as_ref().map(Vec::len), Some(1));
    assert_eq!(output.funding.as_ref().map(Vec::len), Some(1));
    assert!(result.failures.is_empty());
}
#[test]
fn test_work_failure_preserves_previous_success() {
    let first = openalex::merge_work_result(
        EnrichmentResult {
            data: Vec::new(),
            conflicts: Vec::new(),
            failures: Vec::new(),
        },
        "10.1038/nature12373".to_string(),
        Ok(work()),
    );
    let result = openalex::merge_work_result(first, "10.1000/missing".to_string(), Err(eyre!("not found")));
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.failures.len(), 1);
    assert!(result.failures[0].contains("10.1000/missing"));
}

#[tokio::test]
async fn test_work_requires_doi() {
    assert!(openalex::work(&Options::init().build()).await.is_err());
}
