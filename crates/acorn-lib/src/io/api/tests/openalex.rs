use crate::io::api::openalex::{self, Options, Work};
use crate::schema::research_activity::ResearchOutput;
use crate::validation::Validate;

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
fn test_endpoint_accepts_one_options_instance() {
    let options = Options::init()
        .domain("openalex.example")
        .api_key("secret")
        .identifier("https://doi.org/10.1038/nature12373")
        .build();
    let endpoint = openalex::endpoint(&options).expect("OpenAlex endpoint should render");
    assert!(endpoint.starts_with("https://openalex.example/works/doi:10.1038%2Fnature12373?select="));
    assert!(endpoint.ends_with("&api_key=secret"));
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
fn test_endpoint_requires_doi() {
    assert!(openalex::endpoint(&Options::init().build()).is_err());
}
