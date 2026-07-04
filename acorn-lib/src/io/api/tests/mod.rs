#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::io::api::citeas::{self, ToCitations};
use crate::io::api::{
    self, extract_template_keys, orcid, require_non_empty_secret, ror, Configuration, Endpoint, IntoBody, IntoHeaders, Param, ParamStyle, Params,
    RemoteResource, Resource, ResponseContent, INCLUDED_ENDPOINTS,
};
use crate::io::read_file;
use crate::param;
use crate::prelude::HashMap;
use crate::schema::pid::{PersistentIdentifierParse, DOI};
use crate::test::utils::fixture_path;
use crate::util::Searchable;
use crate::{Location, Repository, Scheme};

#[test]
fn test_endpoints_length() {
    assert_eq!(INCLUDED_ENDPOINTS.len(), 12);
}
#[tokio::test]
#[ignore = "Requires internet connection and citeas.org to be up"]
async fn test_citeas() {
    let status = citeas::status().await;
    assert!(status.is_ok());
    if let Ok(citeas::StatusResponse { documentation_url, .. }) = status {
        assert_eq!(documentation_url, "https://citeas.org/api");
    }
    let doi = "10.11578/dc.20250604.1";
    let params = vec![param!(TemplateValue, "doi", doi)];
    let expected = "Wohlgemuth, J. (2025). Accessible Content Optimization for Research Needs (ACORN). Oak Ridge National Laboratory (ORNL), Oak Ridge, TN (United States). http://doi.org/10.11578/DC.20250604.1";
    let options = citeas::Options::from_env().with_params(params);
    if let Some(citeas::Citation { text, .. }) = citeas::search(&options).await.unwrap().match_style("apa") {
        println!("CiteAs Test Response Received");
        assert_eq!(text, expected);
    };
    let doi = DOI::from_string("10.11578/dc.20250604.1");
    if let Some(citeas::Citation { text, .. }) = doi.to_citations().await.unwrap().match_style("apa") {
        println!("CiteAs Test Response Received");
        assert_eq!(text, expected);
    };
}
#[test]
fn test_endpoint_from_location() {
    let expected = "https://api.example.com";
    let location = Location::Simple(expected.to_string());
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, expected);
    let expected = "https://api.example.com";
    let location = Location::Simple(format!("{expected}:8080/v1/data"));
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, expected);
    assert_eq!(endpoint.port, Some(8080));
    let location = Location::Detailed {
        scheme: Scheme::HTTPS,
        uri: "http://api.example.com".to_string(),
    };
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, expected);
    let location = Location::Detailed {
        scheme: Scheme::HTTPS,
        uri: "http://api.example.com:8080".to_string(),
    };
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, expected);
    assert_eq!(endpoint.port, Some(8080));
}
#[test]
fn test_endpoint_from_repository() {
    let expected = "https://code.ornl.gov";
    let uri = format!("{expected}/research-enablement/buckets/nssd");
    let nssd = Repository::GitLab {
        id: Some(1234_u64),
        location: Location::Simple(uri.clone()),
    };
    let endpoint: Endpoint = nssd.into();
    assert_eq!(endpoint.domain, expected);
}
#[test]
fn test_search_resource_find_by_name() {
    let resources = vec![
        Resource {
            name: "status".to_string(),
            method: api::HttpMethod::Get,
            template: "/status".to_string(),
        },
        Resource {
            name: "search".to_string(),
            method: api::HttpMethod::Get,
            template: "/search{{ query }}".to_string(),
        },
    ];
    let found = resources.find_by_name("SEARCH");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "search");
    assert!(resources.find_by_name("missing").is_none());
}
#[test]
fn test_endpoint_context() {
    let params = vec![
        param!(
            QueryPair,
            "q",
            (("affiliation-org-name", "Lyrasis"), ("ror-org-id", "\"https://ror.org/01qz5mb56\""))
        ),
        param!(TemplateValue, "one", "two"),
        param!(TemplateValue, "three", "four"),
    ];
    let data = Some(params);
    let endpoint = Endpoint::at("example.org").root("v3.0").resources(vec![]).build();
    let context = endpoint.context_with::<orcid::SearchField, orcid::OutputColumn>(data);
    assert_eq!(context.get("base").unwrap().as_str(), Some("https://example.org/v3.0"));
    assert!(context.get("query").is_some());
    assert_eq!(context.get("one").unwrap().as_str(), Some("two"));
    assert_eq!(context.get("three").unwrap().as_str(), Some("four"));
    let json = serde_json::to_string_pretty(&context.into_json()).unwrap();
    insta::assert_snapshot!(json);
}
#[test]
fn test_endpoint_handle() {
    let xml = Ok(ResponseContent::Xml("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<expanded-search:expanded-search num-found=\"2\" xmlns:expanded-search=\"http://www.orcid.org/ns/expanded-search\">\n    <expanded-search:expanded-result>\n        <expanded-search:family-names>Carson</expanded-search:family-names>\n    </expanded-search:expanded-result>\n    <expanded-search:expanded-result>\n        <expanded-search:family-names>Wohlgemuth</expanded-search:family-names>\n    </expanded-search:expanded-result>\n</expanded-search:expanded-search>\n".to_string()));
    let endpoint = Endpoint::at("pub.orcid.org").root("v3.0").build();
    let response = endpoint.handle::<orcid::SearchResponse>(xml).expect("Failed to parse XML");
    assert_eq!(response.num_found, 2);
    assert_eq!(response.namespace, "http://www.orcid.org/ns/expanded-search");
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.results[0].family_names, Some("Carson".to_string()));
    assert_eq!(response.results[1].family_names, Some("Wohlgemuth".to_string()));
    let json = Ok(ResponseContent::Json(
        "{\"tomcatUp\":true,\"dbConnectionOk\":true,\"readOnlyDbConnectionOk\":false,\"overallOk\":true}".to_string(),
    ));
    let status = endpoint.handle::<orcid::StatusResponse>(json).expect("Failed to parse JSON");
    assert!(status.application);
    assert!(status.database);
    assert!(!status.database_readonly);
    assert!(status.overall);
    let raw = Ok(ResponseContent::Raw("OK".to_string()));
    let response = endpoint.handle::<api::TextResponse>(raw).expect("Failed to parse raw text");
    assert_eq!(response.content, "OK");
}
#[test]
fn test_endpoint_parse() {
    let xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<expanded-search:expanded-search num-found=\"2\" xmlns:expanded-search=\"http://www.orcid.org/ns/expanded-search\">\n    <expanded-search:expanded-result>\n        <expanded-search:family-names>Carson</expanded-search:family-names>\n    </expanded-search:expanded-result>\n    <expanded-search:expanded-result>\n        <expanded-search:family-names>Wohlgemuth</expanded-search:family-names>\n    </expanded-search:expanded-result>\n</expanded-search:expanded-search>\n";
    let response = api::parse_xml::<orcid::SearchResponse>(xml).expect("Failed to parse XML");
    assert_eq!(response.num_found, 2);
    assert_eq!(response.namespace, "http://www.orcid.org/ns/expanded-search");
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.results[0].family_names, Some("Carson".to_string()));
    assert_eq!(response.results[1].family_names, Some("Wohlgemuth".to_string()));
    let json = "{\"tomcatUp\":true,\"dbConnectionOk\":true,\"readOnlyDbConnectionOk\":false,\"overallOk\":true}";
    let status = api::parse_json::<orcid::StatusResponse>(json).expect("Failed to parse JSON");
    assert!(status.application);
    assert!(status.database);
    assert!(!status.database_readonly);
    assert!(status.overall);
}
#[test]
fn test_extract_template_keys() {
    let template = "{{ base }}/organizations/{{ identifier }}{{ query }}";
    let keys = extract_template_keys(template);
    let expected = vec!["base", "identifier", "query"].into_iter().map(String::from).collect::<Vec<String>>();
    assert_eq!(keys, expected);
    let template = "{{ query | default(value=\"\") }} and {{- name -}}";
    let keys = extract_template_keys(template);
    let expected = vec!["query", "name"].into_iter().map(String::from).collect::<Vec<String>>();
    assert_eq!(keys, expected);
    let template = "{{ base }}/{{ base }}/{{ identifier }}";
    let keys = extract_template_keys(template);
    let expected = vec!["base", "identifier"].into_iter().map(String::from).collect::<Vec<String>>();
    assert_eq!(keys, expected);
    let template = "{{ base }}/{{ missing";
    let keys = extract_template_keys(template);
    let expected = vec!["base"].into_iter().map(String::from).collect::<Vec<String>>();
    assert_eq!(keys, expected);
    let template = "";
    let keys = extract_template_keys(template);
    let expected: Vec<String> = vec![];
    assert_eq!(keys, expected);
}
#[test]
fn test_require_non_empty_secret_rejects_empty_values() {
    let why = require_non_empty_secret("   ", "GitLab", &["CI_JOB_TOKEN", "GITLAB_TOKEN"]).expect_err("empty token must fail");
    let message = why.to_string();
    assert!(message.contains("Missing required token for"));
    assert!(message.contains("CI_JOB_TOKEN"));
    assert!(message.contains("GITLAB_TOKEN"));
}
#[test]
fn test_require_non_empty_secret_trims_and_accepts_value() {
    let token = require_non_empty_secret("  secret-token  ", "RAiD", &["RAID_API_TOKEN"]).expect("non-empty token must pass");
    assert_eq!(token, "secret-token");
}
#[test]
fn test_with_auth_bearer_adds_authorization_header() {
    let params = Params::new().with_auth("  secret-token  ", None).build();
    let headers = params.into_headers();
    let value = headers.get("Authorization").expect("Authorization header should exist");
    assert_eq!(value.to_str().expect("header should be visible"), "Bearer secret-token");
    assert!(value.is_sensitive());
}
#[test]
fn test_with_auth_bearer_rejects_empty_token() {
    let params = Params::new().with_auth("   ", None).build();
    assert!(params.is_empty());
}
#[test]
fn test_with_auth_custom_header() {
    let params = Params::new().with_auth("  secret-token  ", Some("PRIVATE-TOKEN")).build();
    let headers = params.into_headers();
    let value = headers.get("PRIVATE-TOKEN").expect("PRIVATE-TOKEN header should exist");
    assert_eq!(value.to_str().expect("header should be visible"), "secret-token");
    assert!(value.is_sensitive());
}
#[test]
fn test_with_auth_custom_rejects_empty_token() {
    let params = Params::new().with_auth("   ", Some("PRIVATE-TOKEN")).build();
    assert!(params.is_empty());
}
#[test]
fn test_into_body() {
    let body = "This is a test comment!!!";
    let payload = vec![param!(Body & body)].into_body();
    assert_eq!(payload, serde_json::json!("This is a test comment!!!"));
}
#[test]
fn test_into_body_parses_json_string_for_body_key() {
    let body = "{\"title\":\"ACORN\",\"active\":true}";
    let payload = vec![param!(Body & body)].into_body();
    assert_eq!(payload, serde_json::json!({ "title": "ACORN", "active": true }));
}
#[test]
fn test_into_body_keyed_multiple_values_as_object_array() {
    let payload = vec![param!(ParamStyle::Body, "items", vec![vec!["one"], vec!["two"]])].into_body();
    assert_eq!(payload, serde_json::json!({ "items": ["one", "two"] }));
}
#[test]
fn test_into_body_keyed_empty_values_as_object_null() {
    let payload = vec![Param::of_type(ParamStyle::Body).values(vec![]).with_key("data")].into_body();
    assert_eq!(payload, serde_json::json!({ "data": null }));
}
#[test]
fn test_query_params_empty_field() {
    let params = vec![
        param!(FieldList, "query", "Oak Ridge"),
        param!(QueryPair, "filter", ("status", "inactive")),
    ];
    let query = Param::to_query_string::<api::EmptyField, api::EmptyField>(params);
    insta::assert_snapshot!(query);
}
#[test]
fn test_ror_search_response() {
    let json = read_file(fixture_path("").join("response_ror_ornl.json")).expect("Failed to read JSON fixture");
    let response: ror::SearchResponse = serde_json::from_str(&json).expect("Failed to deserialize JSON");
    insta::assert_snapshot!(format!("{:#?}", response));
}
#[cfg(test)]
mod github_api {
    use super::*;
    use crate::io::api::github::{collect_tree, path_for, TreeEntry};
    use crate::io::api::TreeEntryType;

    #[test]
    fn test_template_endpoint_for() {
        let endpoint = Endpoint::from_template("github::api").map(|e| e.with_domain("api.github.com")).unwrap();
        assert_eq!(endpoint.base(), "https://api.github.com");
        assert!(endpoint.resources.iter().any(|r| r.name == "tree"));
    }
    #[test]
    fn test_path_for() {
        assert_eq!(path_for("", "README.md"), "README.md");
        assert_eq!(path_for("content", "index.json"), "content/index.json");
    }
    #[test]
    fn test_collect_tree_partitions_paths_and_subtrees() {
        let entries = vec![
            TreeEntry {
                path: "README.md".to_string(),
                mode: "100644".to_string(),
                entry_type: TreeEntryType::Blob,
                sha: "blob-sha".to_string(),
                size: Some(10),
                url: "https://example.invalid/blob".to_string(),
            },
            TreeEntry {
                path: "content".to_string(),
                mode: "040000".to_string(),
                entry_type: TreeEntryType::Tree,
                sha: "tree-sha".to_string(),
                size: None,
                url: "https://example.invalid/tree".to_string(),
            },
        ];
        let (paths, pending) = collect_tree(entries, "");
        assert_eq!(paths, vec!["README.md".to_string()]);
        assert_eq!(pending, vec![("tree-sha".to_string(), "content".to_string())]);
    }
    #[test]
    fn test_collect_tree_prefixes_nested_entries() {
        let entries = vec![TreeEntry {
            path: "index.json".to_string(),
            mode: "100644".to_string(),
            entry_type: TreeEntryType::Blob,
            sha: "blob-sha".to_string(),
            size: Some(10),
            url: "https://example.invalid/blob".to_string(),
        }];
        let (paths, pending) = collect_tree(entries, "content");
        assert_eq!(paths, vec!["content/index.json".to_string()]);
        assert!(pending.is_empty());
    }
}
#[cfg(test)]
mod gitlab_api {
    use super::*;
    use crate::io::api::gitlab::{
        handle_tree_paths_response, PaginationKey, ProgrammingLanguageDetails, ProgrammingLanguageUseResponse, ProgrammingLanguagesResponse,
        TreeResponse,
    };

    #[test]
    fn test_query_string() {
        let param = param!(KeyValuePair, "per_page", "100");
        let query = param.to_string::<PaginationKey, api::EmptyField>();
        assert_eq!(query, "per_page=100");
    }
    #[test]
    fn test_params_to_query_string() {
        let params = vec![param!(KeyValuePair, "per_page", "100"), param!(KeyValuePair, "page", "2")];
        let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
        assert_eq!(query, "?per_page=100&page=2");
    }
    #[test]
    fn test_params_to_query_string_with_invalid_fields() {
        let params = vec![param!(KeyValuePair, "every_page", "100"), param!(KeyValuePair, "page", "42")];
        let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
        assert_eq!(query, "?page=42");
    }
    #[test]
    fn test_params_to_query_string_with_invalid_values() {
        let params = vec![param!(KeyValuePair, "per_page", "100"), param!(KeyValuePair, "page", "not a number")];
        let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
        assert_eq!(query, "?per_page=100");
        let params = vec![param!(KeyValuePair, "per_page", "{}"), param!(KeyValuePair, "page", "not a number")];
        let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
        assert!(query.is_empty());
    }
    #[test]
    fn test_programming_languages_response_parse_filters_programming_only() {
        let data = HashMap::from_iter([
            (
                "Python".to_string(),
                ProgrammingLanguageDetails {
                    language_id: Some(303),
                    language_type: Some("programming".to_string()),
                    color: Some("#3572A5".to_string()),
                    group: None,
                },
            ),
            (
                "YAML".to_string(),
                ProgrammingLanguageDetails {
                    language_id: Some(407),
                    language_type: Some("data".to_string()),
                    color: Some("#cb171e".to_string()),
                    group: None,
                },
            ),
        ]);
        let response = ProgrammingLanguagesResponse::parse(data);
        assert_eq!(response.languages.len(), 1);
        assert_eq!(response.languages[0].name, "Python");
        assert_eq!(response.languages[0].language_id, Some(303));
    }
    #[test]
    fn test_programming_language_use_response_deserializes_map() {
        let json = r#"{"Rust":98.12,"Makefile":0.5,"Python":0.49}"#;
        let response: ProgrammingLanguageUseResponse = serde_json::from_str(json).expect("should deserialize language usage map");
        assert_eq!(response.languages.len(), 3);
        assert_eq!(response.languages[0].name, "Makefile");
        assert_eq!(response.languages[0].percentage, 0.5);
        assert_eq!(response.languages[1].name, "Python");
        assert_eq!(response.languages[2].name, "Rust");
    }
    #[test]
    fn test_parse_tree_paths_response_filters_blob_entries() {
        let json = r#"[
            {"id":"1","name":"README.md","type":"blob","path":"README.md","mode":"100644"},
            {"id":"2","name":"content","type":"tree","path":"content","mode":"040000"}
        ]"#;
        let response: TreeResponse = serde_json::from_str(json).expect("tree entries should parse");
        assert_eq!(response.paths, vec!["README.md".to_string()]);
    }
    #[test]
    fn test_parse_tree_paths_response_treats_later_page_403_as_terminal() {
        let json = r#"{"message":"403 Forbidden"}"#;
        let response: TreeResponse = serde_json::from_str(json).expect("error payload should deserialize as tree response");
        assert!(response.error.is_some());
        let response = handle_tree_paths_response(Ok(response), 3).expect("later-page forbidden should be terminal");
        assert!(response.paths.is_empty());
    }
    #[test]
    fn test_parse_tree_paths_response_returns_error_on_first_page_403() {
        let json = r#"{"message":"403 Forbidden"}"#;
        let response: TreeResponse = serde_json::from_str(json).expect("error payload should deserialize as tree response");
        let why = handle_tree_paths_response(Ok(response), 1).expect_err("first-page forbidden should fail");
        assert!(why.to_string().contains("403 Forbidden"));
    }
    #[test]
    fn test_parse_tree_paths_response_returns_actionable_error_for_non_json() {
        let html = "<!doctype html><html><body>403 Forbidden</body></html>";
        let why = serde_json::from_str::<TreeResponse>(html).expect_err("non-json response should fail with parse error");
        assert!(why.to_string().contains("expected value"));
    }
}
#[cfg(test)]
mod openai_api {
    use super::*;

    #[test]
    fn test_template_endpoint_for() {
        let endpoint = Endpoint::from_template("openai::api").map(|e| e.with_domain("api.openai.com")).unwrap();
        assert_eq!(endpoint.base(), "https://api.openai.com/v1");
        assert!(endpoint.resources.iter().any(|r| r.name == "models"));
        assert!(endpoint.resources.iter().any(|r| r.name == "model"));
        assert!(endpoint.resources.iter().any(|r| r.name == "chat-completion"));
        assert!(endpoint.resources.iter().any(|r| r.name == "chat-completion::delete"));
        assert!(endpoint.resources.iter().any(|r| r.name == "chat-completion::list"));
        assert!(endpoint.resources.iter().any(|r| r.name == "chat-completion::messages"));
        assert!(endpoint.resources.iter().any(|r| r.name == "chat-completion::retrieve"));
        assert!(endpoint.resources.iter().any(|r| r.name == "chat-completion::update"));
        assert!(endpoint.resources.iter().any(|r| r.name == "response"));
        assert!(endpoint.resources.iter().any(|r| r.name == "response::cancel"));
        assert!(endpoint.resources.iter().any(|r| r.name == "response::delete"));
        assert!(endpoint.resources.iter().any(|r| r.name == "response::input-items"));
        assert!(endpoint.resources.iter().any(|r| r.name == "response::retrieve"));
        assert!(endpoint.resources.iter().any(|r| r.name == "model::delete"));
    }
    #[test]
    fn test_from_template_with_domain() {
        let result = Endpoint::from_template("openai::api").map(|e| e.with_domain("proxy.openai.example.com"));
        assert!(result.is_ok());
        let endpoint = result.unwrap();
        assert_eq!(endpoint.domain, "proxy.openai.example.com");
        assert_eq!(endpoint.root, Some("v1".to_string()));
    }
}
#[cfg(test)]
mod openapi_import {
    use super::*;

    #[test]
    fn test_import_resources_from_openapi_snapshot() {
        let spec = r#"
openapi: 3.1.0
info:
    title: Test API
    version: 1.0.0
paths:
    /widgets:
        get:
            operationId: listWidgets
        post:
            operationId: createWidget
    /widgets/{widget_id}:
        delete:
            operationId: deleteWidget
        get:
            operationId: getWidget
    /widgets/{widget_id}/parts/{part_id}:
        patch:
            operationId: updateWidgetPart
"#;
        let resources = api::openapi::import_resources_from_openapi(spec).unwrap();
        insta::assert_json_snapshot!("import_resources_from_openapi_snapshot", resources);
    }
}
#[cfg(test)]
mod orcid_api {
    use super::*;
    use crate::io::api::orcid::{OutputColumn, SearchField};

    #[test]
    fn test_query_string() {
        // Basic query with multiple valid fields
        let pairs = vec![
            ("given-names", "Jason"),
            ("affiliation-org-name", "Oak Ridge National Laboratory"),
            ("family-name", "Wohlgemuth"),
        ];
        let query = orcid::query_string(pairs, vec![], vec![]);
        let expected = "?q=given-names:Jason+AND+affiliation-org-name:Oak%20Ridge%20National%20Laboratory+AND+family-name:Wohlgemuth";
        assert_eq!(query, expected);
        // Query with invalid field name (should be skipped)
        let pairs = vec![
            ("invalid-field", "should be skipped"),
            ("given-names", "Trista"),
            ("family-name", "Smith"),
        ];
        let query = orcid::query_string(pairs, vec![], vec![]);
        let expected = "?q=given-names:Trista+AND+family-name:Smith";
        assert_eq!(query, expected);
        // Empty query
        let pairs: Vec<(&str, &str)> = vec![];
        let query = orcid::query_string(pairs, vec![], vec![]);
        let expected = "";
        assert_eq!(query, expected);
        // Query with invalid ORCiD (invalid checksum, should be skipped)
        let pairs = vec![("orcid", "0000-0002-1823-1234"), ("given-names", "Test")];
        let query = orcid::query_string(pairs, vec![], vec![]);
        let expected = "?q=given-names:Test";
        assert_eq!(query, expected);
        // Query with valid ORCiD
        let pairs = vec![("orcid", "0000-0002-1825-0097"), ("given-names", "Test")];
        let query = orcid::query_string(pairs, vec![], vec![]);
        let expected = "?q=orcid:0000-0002-1825-0097+AND+given-names:Test";
        assert_eq!(query, expected);
        // Query with invalid ROR (should be skipped)
        let pairs = vec![("ror-org-id", "03ebg0v16-"), ("affiliation-org-name", "Test University")];
        let query = orcid::query_string(pairs, vec![], vec![]);
        let expected = "?q=affiliation-org-name:Test%20University";
        assert_eq!(query, expected);
        // Query with valid ROR
        let pairs = vec![
            ("ror-org-id", "https://ror.org/01qz5mb56"),
            ("affiliation-org-name", "Oak Ridge National Laboratory"),
        ];
        let query = orcid::query_string(pairs, vec![], vec![]);
        let expected = "?q=ror-org-id:https%3A%2F%2Fror.org%2F01qz5mb56+AND+affiliation-org-name:Oak%20Ridge%20National%20Laboratory";
        assert_eq!(query, expected);
        // Query with valid output columns
        let pairs = vec![("given-names", "Alice"), ("family-name", "Smith")];
        let columns = vec!["orcid", "email", "credit-name"];
        let query = orcid::query_string(pairs, columns, vec![]);
        let expected = "?q=given-names:Alice+AND+family-name:Smith&fl=orcid,email,credit-name";
        assert_eq!(query, expected);
        // Query with invalid output columns (should be filtered out)
        let pairs = vec![("given-names", "Bob")];
        let columns = vec!["orcid", "invalid-column", "email"];
        let query = orcid::query_string(pairs, columns, vec![]);
        let expected = "?q=given-names:Bob&fl=orcid,email";
        assert_eq!(query, expected);
        // Query with mixed invalid fields and columns
        let pairs = vec![
            ("given-names", "Jason"),
            ("invalid-field", "should be ignored"),
            ("family-name", "Wohlgemuth"),
        ];
        let columns = vec!["orcid", "invalid-column", "credit-name", "bad-field"];
        let query = orcid::query_string(pairs, columns, vec![]);
        let expected = "?q=given-names:Jason+AND+family-name:Wohlgemuth&fl=orcid,credit-name";
        assert_eq!(query, expected);
        // Query with all valid output columns
        let pairs = vec![("family-name", "Wohlgemuth")];
        let columns = vec![
            "orcid",
            "email",
            "credit-name",
            "given-names",
            "family-name",
            "other-name",
            "current-institution-affiliation-name",
            "past-institution-affiliation-name",
        ];
        let query = orcid::query_string(pairs, columns, vec![]);
        let expected = "?q=family-name:Wohlgemuth&fl=orcid,email,credit-name,given-names,family-name,other-name,current-institution-affiliation-name,past-institution-affiliation-name";
        assert_eq!(query, expected);
        // Query with boost for single field
        let pairs = vec![("given-names", "Audrey"), ("family-name", "Carson")];
        let boost = vec!["family-name"];
        let query = orcid::query_string(pairs, vec![], boost);
        let expected = "?q=given-names:Audrey+AND+family-name:Carson&qf=family-name%5E2.0";
        assert_eq!(query, expected);
        // Query with boost for multiple fields (including invalid field which should be ignored)
        let pairs = vec![("given-names", "Jason"), ("family-name", "Wohlgemuth")];
        let boost = vec!["given-names", "family-name", "not-a-valid-field"];
        let query = orcid::query_string(pairs, vec![], boost);
        let expected = "?q=given-names:Jason+AND+family-name:Wohlgemuth&qf=given-names%5E3.0%20family-name%5E2.0";
        assert_eq!(query, expected);
    }
    #[test]
    fn test_response_handler() {
        // Load XML fixture file
        let xml = read_file(fixture_path("").join("response_orcid_wohlgemuth.xml")).expect("Failed to read XML fixture");
        // Parse XML into OrcidSearchResponse
        let response = api::parse_xml::<orcid::SearchResponse>(&xml).expect("Failed to deserialize XML");
        // Verify basic response properties
        let total = 68;
        assert_eq!(response.num_found, total);
        assert_eq!(response.namespace, "http://www.orcid.org/ns/expanded-search");
        assert_eq!(response.results.len(), total);
        // Verify first result (Pierre Wohlgemuth)
        let first = &response.results[0];
        assert_eq!(first.orcid_id, Some("0000-0001-6067-5067".to_string()));
        assert_eq!(first.given_names, Some("Pierre".to_string()));
        assert_eq!(first.family_names, Some("Wohlgemuth".to_string()));
        let institutions = first.institution_names.as_ref().expect("Institution names should exist");
        assert_eq!(institutions.len(), 2);
        assert!(institutions.contains(&"New York University".to_string()));
        assert!(institutions.contains(&"Université de Lorraine".to_string()));
        assert!(first.emails.is_none());
        assert_eq!(first.credit_name, None);
        // Verify second result (Katharina Wohlgemuth - minimal data)
        let second = &response.results[1];
        assert_eq!(second.orcid_id, Some("0000-0002-7238-5566".to_string()));
        assert_eq!(second.given_names, Some("Katharina".to_string()));
        assert_eq!(second.family_names, Some("Wohlgemuth".to_string()));
        assert!(second.institution_names.as_ref().is_none_or(|v| v.is_empty()));
        assert!(second.emails.is_none());
        // Verify result with credit-name (Matthias Wohlgemuth)
        let matthias = &response.results[7];
        assert_eq!(matthias.orcid_id, Some("0000-0001-7018-2944".to_string()));
        assert_eq!(matthias.given_names, Some("Matthias".to_string()));
        assert_eq!(matthias.family_names, Some("Wohlgemuth".to_string()));
        assert_eq!(matthias.credit_name, Some("Matthias Wohlgemuth".to_string()));
        // Verify result with emails (Jorge Marcelo Wohlgemuth)
        let jorge = &response.results[15];
        assert_eq!(jorge.orcid_id, Some("0000-0002-0502-5982".to_string()));
        assert_eq!(jorge.given_names, Some("Jorge Marcelo".to_string()));
        assert_eq!(jorge.family_names, Some("Wohlgemuth".to_string()));
        let emails = jorge.emails.as_ref().expect("Emails should exist");
        assert_eq!(emails.len(), 2);
        assert!(emails.contains(&"jorge.202222692@unilasalle.edu.br".to_string()));
        assert!(emails.contains(&"jorgemarcelow@gmail.com".to_string()));
        // Verify result with other-name (Nicholas Wohlgemuth)
        let nicholas = &response.results[31];
        assert_eq!(nicholas.orcid_id, Some("0000-0002-6450-6452".to_string()));
        assert_eq!(nicholas.given_names, Some("Nicholas".to_string()));
        assert_eq!(nicholas.family_names, Some("Wohlgemuth".to_string()));
        assert!(nicholas
            .other_name
            .as_ref()
            .is_some_and(|v| v.contains(&"Nicholas J. Wohlgemuth".to_string())));
        let institutions = nicholas.institution_names.as_ref().expect("Institution names should exist");
        assert_eq!(institutions.len(), 6);
        // Verify result with many institutions (Sven Wohlgemuth)
        let sven = response
            .results
            .iter()
            .find(|r| r.orcid_id.as_deref() == Some("0000-0001-5276-940X"))
            .expect("Sven Wohlgemuth not found");
        assert_eq!(sven.given_names, Some("Sven".to_string()));
        assert_eq!(sven.credit_name, Some("Dr. Sven Wohlgemuth".to_string()));
        let institutions = sven.institution_names.as_ref().expect("Institution names should exist");
        assert!(institutions.len() >= 10);
    }
    #[test]
    fn test_status_response() {
        let json = r#"{"tomcatUp":true,"dbConnectionOk":true,"readOnlyDbConnectionOk":false,"overallOk":true}"#;
        let response: orcid::StatusResponse = serde_json::from_str(json).expect("Failed to deserialize JSON");
        insta::assert_snapshot!(format!("{:#?}", response));
    }
    #[test]
    fn test_query_param_field_list() {
        // Test FieldList style (output columns)
        let param = Param::of_type(ParamStyle::FieldList)
            .values(vec![
                vec![Some("orcid"), None],
                vec![Some("email"), None],
                vec![Some("credit-name"), None],
            ])
            .with_key("fl");
        let rendered = param.to_string::<SearchField, OutputColumn>();
        let expected = "fl=orcid,email,credit-name";
        assert_eq!(rendered, expected);
        // Test FieldList style with invalid field (should be filtered)
        let param = Param::of_type(ParamStyle::FieldList)
            .values(vec![
                vec![Some("orcid"), None],
                vec![Some("invalid-column"), None],
                vec![Some("email"), None],
            ])
            .with_key("fl");
        let rendered = param.to_string::<SearchField, OutputColumn>();
        let expected = "fl=orcid,email";
        assert_eq!(rendered, expected);
    }
    #[test]
    fn test_query_param_query_field() {
        // Test QueryField style (boosted fields)
        let param = Param::of_type(ParamStyle::QueryField)
            .values(vec![vec![Some("given-names"), None], vec![Some("family-name"), None]])
            .with_key("qf");
        let rendered = param.to_string::<SearchField, OutputColumn>();
        let expected = "qf=given-names%5E3.0%20family-name%5E2.0";
        assert_eq!(rendered, expected);
        // Test QueryField style with three fields
        let param = Param::of_type(ParamStyle::QueryField)
            .values(vec![
                vec![Some("given-names"), None],
                vec![Some("family-name"), None],
                vec![Some("affiliation-org-name"), None],
            ])
            .with_key("qf");
        let rendered = param.to_string::<SearchField, OutputColumn>();
        let expected = "qf=given-names%5E4.0%20family-name%5E3.0%20affiliation-org-name%5E2.0";
        assert_eq!(rendered, expected);
        // Test QueryField with invalid field names (should be filtered)
        let param = Param::of_type(ParamStyle::QueryField)
            .values(vec![
                vec![Some("invalid-field"), None],
                vec![Some("given-names"), None],
                vec![Some("another-invalid"), None],
                vec![Some("family-name"), None],
            ])
            .with_key("qf");
        let rendered = param.to_string::<SearchField, OutputColumn>();
        let expected = "qf=given-names%5E3.0%20family-name%5E2.0";
        assert_eq!(rendered, expected);
        // Test QueryField with only invalid field names (should return empty)
        let param = Param::of_type(ParamStyle::QueryField)
            .values(vec![vec![Some("invalid-field"), None], vec![Some("another-invalid"), None]])
            .with_key("qf");
        let rendered = param.to_string::<SearchField, OutputColumn>();
        let expected = "";
        assert_eq!(rendered, expected);
    }
    #[test]
    fn test_param_query_pair() {
        // Test QueryPair style
        let param = param!(QueryPair, "q", (("given-names", "Jason"), ("family-name", "Wohlgemuth")));
        let rendered = param.to_string::<SearchField, OutputColumn>();
        let expected = "q=given-names:Jason+AND+family-name:Wohlgemuth";
        assert_eq!(rendered, expected);
        // Test QueryPair with invalid field names (should be filtered)
        let param = param!(
            QueryPair,
            "q",
            (("invalid-field", "value"), ("given-names", "Jason"), ("another-invalid", "data"))
        );
        let rendered = param.to_string::<SearchField, OutputColumn>();
        let expected = "q=given-names:Jason";
        assert_eq!(rendered, expected);
        // Test QueryPair with invalid field values (invalid ORCiD should be filtered)
        let param = param!(QueryPair, "q", (("orcid", "0000-0002-1823-1234"), ("given-names", "Jason")));
        let rendered = param.to_string::<SearchField, OutputColumn>();
        let expected = "q=given-names:Jason";
        assert_eq!(rendered, expected);
        // Test QueryPair with valid ORCiD value (should be included)
        let param = param!(QueryPair, "q", (("orcid", "0000-0002-2057-9115"), ("given-names", "Jason")));
        let rendered = param.to_string::<SearchField, OutputColumn>();
        let expected = "q=orcid:0000-0002-2057-9115+AND+given-names:Jason";
        assert_eq!(rendered, expected);
    }
    #[test]
    fn test_params_to_query_string() {
        // Test rendering multiple params to a query string
        let params = vec![
            Param::from_query_pair("q", vec![("given-names", "Jason"), ("family-name", "Wohlgemuth")]),
            Param::from_field_list("fl", vec!["orcid", "email", "credit-name"]),
            Param::from_query_field("qf", vec!["given-names", "family-name"]),
        ];
        let query = api::Param::to_query_string::<SearchField, OutputColumn>(params);
        let expected = "?q=given-names:Jason+AND+family-name:Wohlgemuth&fl=orcid,email,credit-name&qf=given-names%5E3.0%20family-name%5E2.0";
        assert_eq!(query, expected);
        // Test rendering with empty params (should return empty string)
        let params = vec![];
        let query = api::Param::to_query_string::<SearchField, OutputColumn>(params);
        let expected = "";
        assert_eq!(query, expected);
    }
    #[test]
    fn test_params_into_body() {
        use crate::io::api::IntoBody;
        // Test single value body param
        let params = vec![param!(Body, "message", "test value")];
        let body = params.into_body();
        assert_eq!(body["message"], serde_json::Value::String("test value".to_string()));
        // Test multiple values body param (should be an array)
        let params = vec![param!(ParamStyle::Body, "items", vec![vec!["value1"], vec!["value2"]])];
        let body = params.into_body();
        assert_eq!(body["items"], serde_json::json!(["value1", "value2"]));
        // Test empty body param (should be null)
        let params = vec![Param::of_type(ParamStyle::Body).values(vec![]).with_key("empty")];
        let body = params.into_body();
        assert_eq!(body["empty"], serde_json::Value::Null);
        // Test mixed params (only body params should be included)
        let params = vec![
            param!(Body, "body_field", "body value"),
            param!(QueryPair, "q", ("query", "value")),
            param!(Header, "Authorization", "Bearer token"),
        ];
        let body = params.into_body();
        assert_eq!(body["body_field"], serde_json::Value::String("body value".to_string()));
    }
    #[test]
    fn test_endpoint_with_domain() {
        let endpoint = Endpoint::at("original.com").root("api/v1").build();
        let updated = endpoint.with_domain("custom.domain.com");
        assert_eq!(updated.domain, "custom.domain.com");
        assert_eq!(updated.root, Some("api/v1".to_string()));
        assert_eq!(endpoint.domain, "original.com");
    }
    #[test]
    fn test_from_template_with_domain() {
        let result = Endpoint::from_template("gitlab::api").map(|e| e.with_domain("custom-gitlab.example.com"));
        assert!(result.is_ok());
        let endpoint = result.unwrap();
        assert_eq!(endpoint.domain, "custom-gitlab.example.com");
        assert_eq!(endpoint.root, Some("api/v4".to_string()));
        assert!(endpoint.resources.iter().any(|r| r.name == "tree"));
    }
    #[test]
    fn test_from_template_with_domain_not_found() {
        let result = Endpoint::from_template("nonexistent-endpoint").map(|e| e.with_domain("example.com"));
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("nonexistent-endpoint"));
        assert!(error_msg.contains("not found"));
    }
    #[test]
    fn test_from_template_with_domain_preserves_properties() {
        let result = Endpoint::from_template("orcid").map(|e| e.with_domain("custom-orcid.example.com"));
        assert!(result.is_ok());
        let endpoint = result.unwrap();
        assert_eq!(endpoint.domain, "custom-orcid.example.com");
        assert_eq!(endpoint.root, Some("v3.0".to_string()));
        assert!(endpoint.resources.iter().any(|r| r.name == "search"));
    }
}
mod models_dev {
    use super::*;
    use crate::io::api::models_dev::CatalogResponse;
    use crate::io::database::schema::Table;
    use crate::io::database::{Database, Operations};
    use crate::test::utils::unique_path;

    #[test]
    fn test_catalog_response_parse() {
        let json = read_file(fixture_path("").join("catalog.json")).expect("Failed to read catalog fixture");
        let catalog: CatalogResponse = serde_json::from_str(&json).expect("Failed to deserialize catalog fixture");
        assert_eq!(catalog.models.len(), 215);
        assert_eq!(catalog.providers.len(), 145);
        let model = catalog.models.get("openai/o3").expect("Missing openai/o3 model");
        assert_eq!(model.name.as_deref(), Some("o3"));
        assert_eq!(model.family.as_deref(), Some("o"));
        let provider = catalog.providers.get("openai").expect("Missing openai provider");
        assert_eq!(provider.name.as_deref(), Some("OpenAI"));
        assert_eq!(provider.documentation.as_deref(), Some("https://platform.openai.com/docs/models"));
        assert!(provider
            .models
            .as_ref()
            .is_some_and(|models| models.iter().any(|model| model.id.as_deref() == Some("o3"))));
    }
    #[tokio::test]
    async fn test_catalog_fixture_persists_models_and_providers_to_local_database() {
        let json = read_file(fixture_path("").join("catalog.json")).expect("Failed to read catalog fixture");
        let catalog: CatalogResponse = serde_json::from_str(&json).expect("Failed to deserialize catalog fixture");
        let models = catalog.models();
        let providers = catalog.providers();
        let model_count = models.models.len();
        let provider_count = providers.providers.len();
        let path = unique_path("models-dev", "duckdb");
        let database = Database::<Table>::from_path(Some(path));
        database.migrate().expect("Failed to migrate database schema");
        let inserted_models = database.persist(Some(models)).await.expect("Failed to persist model catalog data");
        let inserted_providers = database.persist(Some(providers)).await.expect("Failed to persist provider catalog data");
        assert_eq!(inserted_models, model_count);
        assert_eq!(inserted_providers, provider_count);
        let stored_models = database.row_count(Table::Models).expect("Failed to count model rows");
        let stored_providers = database.row_count(Table::Providers).expect("Failed to count provider rows");
        assert_eq!(stored_models, model_count);
        assert_eq!(stored_providers, provider_count);
    }
}
