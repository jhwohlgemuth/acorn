#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::io::api::citeas::{self, ToCitations};
use crate::io::api::{
    self, extract_template_keys, orcid, render, require_non_empty_secret, ror, Configuration, Endpoint, IntoBody, IntoHeaders, Param, ParamStyle,
    Params, RemoteResource, Resource, ResponseContent, TreeEntry, TreeEntryType, INCLUDED_ENDPOINTS,
};
use crate::io::read_file;
use crate::param;
use crate::schema::pid::{PersistentIdentifierParse, DOI};
use crate::test::utils::fixture_path;
use crate::util::Searchable;
use crate::{Location, Repository, Scheme};

#[test]
fn test_endpoints_length() {
    assert_eq!(INCLUDED_ENDPOINTS.len(), 14);
}
#[test]
fn test_tree_entry_type_deserializes_provider_aliases() {
    assert_eq!(serde_json::from_str::<TreeEntryType>(r#""file""#).unwrap(), TreeEntryType::File);
    assert_eq!(serde_json::from_str::<TreeEntryType>(r#""blob""#).unwrap(), TreeEntryType::File);
    assert_eq!(serde_json::from_str::<TreeEntryType>(r#""directory""#).unwrap(), TreeEntryType::Directory);
    assert_eq!(serde_json::from_str::<TreeEntryType>(r#""tree""#).unwrap(), TreeEntryType::Directory);
}
#[test]
fn test_tree_entry_deserializes_huggingface_shape() {
    let entries: Vec<TreeEntry> = serde_json::from_str(
        r#"[
            {"path":"model.Q4_K_M.gguf","type":"file","size":123},
            {"path":"refs","type":"directory"}
        ]"#,
    )
    .unwrap();
    assert!(entries[0].is_file());
    assert!(entries[1].is_directory());
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
    let expected_domain = "api.example.com";
    let expected_base = "https://api.example.com";
    let location = Location::Simple(expected_base.to_string());
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, expected_domain);
    assert_eq!(endpoint.base(), expected_base);
    let location = Location::Simple(format!("{expected_base}:8080/v1/data"));
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, expected_domain);
    assert_eq!(endpoint.port, Some(8080));
    assert_eq!(endpoint.base(), "https://api.example.com:8080");
    let location = Location::Detailed {
        scheme: Scheme::HTTPS,
        uri: "http://api.example.com".to_string(),
        revision: None,
    };
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, expected_domain);
    assert_eq!(endpoint.base(), expected_base);
    let location = Location::Detailed {
        scheme: Scheme::HTTPS,
        uri: "http://api.example.com:8080".to_string(),
        revision: None,
    };
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, expected_domain);
    assert_eq!(endpoint.port, Some(8080));
    assert_eq!(endpoint.base(), "https://api.example.com:8080");
}
#[test]
fn test_endpoint_from_repository() {
    let expected_domain = "code.ornl.gov";
    let expected_base = "https://code.ornl.gov";
    let uri = format!("{expected_base}/research-enablement/buckets/nssd");
    let nssd = Repository::GitLab {
        id: Some(1234_u64),
        location: Location::Simple(uri.clone()),
    };
    let endpoint: Endpoint = nssd.into();
    assert_eq!(endpoint.domain, expected_domain);
    assert_eq!(endpoint.base(), expected_base);
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
fn test_render_replaces_known_values() {
    let mut context = tera::Context::new();
    context.insert("base", "https://example.org");
    context.insert("identifier", "abc-123");
    let rendered = render("{{ base }}/items/{{ identifier }}", &context);
    assert_eq!(rendered, "https://example.org/items/abc-123");
}
#[test]
fn test_render_inserts_empty_for_missing_values() {
    let context = tera::Context::new();
    let rendered = render("{{ base }}/items/{{ identifier }}{{ query }}", &context);
    assert_eq!(rendered, "/items/");
}
#[test]
fn test_render_prefills_empty_before_default_filter() {
    let context = tera::Context::new();
    let rendered = render("{{ query | default(value=\"all\") }}", &context);
    assert_eq!(rendered, "");
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
mod gguf {
    use crate::io::api::huggingface::HuggingFaceError;

    #[test]
    fn test_discovery_errors_include_context() {
        assert_eq!(
            HuggingFaceError::NoGgufQuantizationRepository {
                identifier: "openai/gpt-oss-2b".into()
            }
            .to_string(),
            "no GGUF quantization repo found for openai/gpt-oss-2b"
        );
        assert_eq!(
            HuggingFaceError::InvalidBaseModelIdentifier {
                identifier: "openai/gpt-oss-2b".into()
            }
            .to_string(),
            "invalid Hugging Face base model identifier: openai/gpt-oss-2b"
        );
        assert_eq!(
            HuggingFaceError::ClientInitializationFailed {
                reason: "invalid endpoint".into()
            }
            .to_string(),
            "failed to initialize Hugging Face client: invalid endpoint"
        );
        assert_eq!(
            HuggingFaceError::ModelSearchConfigurationFailed {
                reason: "invalid query".into()
            }
            .to_string(),
            "failed to configure Hugging Face model search: invalid query"
        );
        assert_eq!(
            HuggingFaceError::ModelSearchFailed {
                reason: "request failed".into()
            }
            .to_string(),
            "failed to search Hugging Face models: request failed"
        );
    }
}
#[cfg(test)]
mod github_api {
    use super::*;
    use crate::io::api::github::{collect_tree, path_for, TreeEntry};

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
                entry_type: TreeEntryType::File,
                sha: "blob-sha".to_string(),
                size: Some(10),
                url: "https://example.invalid/blob".to_string(),
            },
            TreeEntry {
                path: "content".to_string(),
                mode: "040000".to_string(),
                entry_type: TreeEntryType::Directory,
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
            entry_type: TreeEntryType::File,
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
mod openai_api {
    use super::*;

    #[test]
    fn test_template_endpoint_for() {
        let endpoint = Endpoint::from_template("openai::api").map(|e| e.with_domain("api.openai.com")).unwrap();
        assert_eq!(endpoint.base(), "https://api.openai.com/v1");
        let names: Vec<&str> = endpoint.resources.iter().map(|resource| resource.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "completion",
                "chat-completion",
                "models",
                "response",
                "embedding",
                "audio-speech",
                "audio-transcription",
                "audio-voices",
                "image-generation",
                "image-edit",
            ]
        );
        assert!(!endpoint.resources.iter().any(|r| r.name == "model"));
        assert!(!endpoint.resources.iter().any(|r| r.name == "model::delete"));
        assert!(!endpoint.resources.iter().any(|r| r.name.starts_with("chat-completion::")));
        assert!(!endpoint.resources.iter().any(|r| r.name.starts_with("response::")));
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
    fn test_endpoint_with_domain_accepts_uri() {
        let endpoint = Endpoint::at("original.com").root("api/v1").build();
        let updated = endpoint.with_domain("http://custom.domain.com:8080/path");
        assert_eq!(updated.domain, "custom.domain.com");
        assert_eq!(updated.scheme, Some(crate::Scheme::HTTP));
        assert_eq!(updated.port, Some(8080));
        assert_eq!(updated.root, Some("api/v1".to_string()));
    }
    #[test]
    fn test_endpoint_with_domain_accepts_host_port() {
        let endpoint = Endpoint::at("original.com").build();
        let updated = endpoint.with_domain("localhost:3000");
        assert_eq!(updated.domain, "localhost");
        assert_eq!(updated.scheme, None);
        assert_eq!(updated.port, Some(3000));
    }
    #[test]
    fn test_endpoint_from_parts_sets_explicit_values() {
        let endpoint = Endpoint::from_parts("api.example.com".to_string(), Some(crate::Scheme::HTTP), Some(8080));
        assert_eq!(endpoint.domain, "api.example.com");
        assert_eq!(endpoint.scheme, Some(crate::Scheme::HTTP));
        assert_eq!(endpoint.port, Some(8080));
    }
    #[test]
    fn test_endpoint_from_parts_preserves_defaults_for_missing_values() {
        let endpoint = Endpoint::from_parts("api.example.com".to_string(), None, None);
        assert_eq!(endpoint.domain, "api.example.com");
        assert_eq!(endpoint.scheme, None);
        assert_eq!(endpoint.port, None);
        assert_eq!(endpoint.name, String::new());
        assert_eq!(endpoint.root, None);
        assert!(endpoint.resources.is_empty());
        assert!(endpoint.authentication.is_none());
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
    use crate::io::database::schema::{ModelRow, ProviderRow, Table};
    use crate::io::database::{Database, Operations};
    use crate::schema::agent::{Metric, ModelDetails, ProviderDetails};
    use crate::test::utils::unique_path;
    use std::collections::HashMap;

    fn assert_models_dev_schema_variants(models: &HashMap<String, ModelDetails>) {
        let missing_limit_output = models
            .get("fixture/missing-limit-output")
            .expect("fixture model with missing limit output should exist");
        assert_eq!(missing_limit_output.limit.as_ref().and_then(|limit| limit.output), None);
        assert!(missing_limit_output.benchmarks.is_none());
        let single_benchmark = models
            .get("fixture/single-benchmark-missing-metric")
            .expect("fixture model with single benchmark object should exist");
        let benchmarks = single_benchmark
            .benchmarks
            .as_ref()
            .expect("benchmark object should deserialize as a list")
            .as_slice();
        assert_eq!(benchmarks.len(), 1);
        assert!(benchmarks[0].metric.is_none());
        let unknown_metric = models
            .get("fixture/unknown-benchmark-metric")
            .expect("fixture model with unknown benchmark metrics should exist");
        let benchmarks = unknown_metric.benchmarks.as_ref().expect("benchmark list should deserialize").as_slice();
        assert_eq!(benchmarks.len(), 2);
        match benchmarks[0].metric.as_ref() {
            | Some(Metric::Other(value)) => assert_eq!(value, "percent"),
            | other => panic!("expected percent to deserialize as Metric::Other, got {other:?}"),
        }
        assert!(matches!(benchmarks[1].metric.as_ref(), Some(Metric::PercentResolved)));
    }

    #[test]
    fn test_catalog_response_parse() {
        let json = read_file(fixture_path("").join("catalog.json")).expect("Failed to read catalog fixture");
        let catalog: CatalogResponse = serde_json::from_str(&json).expect("Failed to deserialize catalog fixture");
        assert_eq!(catalog.models.len(), 218);
        assert_eq!(catalog.providers.len(), 145);
        assert_models_dev_schema_variants(&catalog.models);
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
    #[test]
    fn test_models_fixture_covers_schema_variants() {
        let json = read_file(fixture_path("").join("models.json")).expect("Failed to read models fixture");
        let models: HashMap<String, ModelDetails> = serde_json::from_str(&json).expect("Failed to deserialize models fixture");
        assert_models_dev_schema_variants(&models);
    }
    #[test]
    fn test_catalog_response_accepts_current_benchmark_values() {
        let json = r#"{
            "models": {
                "xai/grok-4.3": {
                    "id": "xai/grok-4.3",
                    "benchmarks": [
                        {"name": "Artificial Analysis Intelligence Index", "score": 53, "metric": "index score", "source": "https://example.com"},
                        {"name": "GDPval-AA", "score": 1500, "metric": "Elo", "source": "https://example.com"},
                        {"name": "IFBench", "score": 81, "metric": "accuracy", "source": "https://example.com"},
                        {"name": "DeepSWE", "score": 53, "metric": "resolve rate", "harness": "mini-swe-agent", "source": "https://example.com"},
                        {"name": "Kimi Code Bench", "score": 62, "harness": "Kimi Code CLI", "source": "https://example.com"},
                        {"name": "Numeric Metric", "score": 1, "metric": 1, "source": "https://example.com"},
                        {"name": "Boolean Metric", "score": 1, "metric": true, "source": "https://example.com"},
                        {"name": "Object Metric", "score": 1, "metric": {"kind": "composite"}, "source": "https://example.com"}
                    ]
                }
            },
            "providers": {}
        }"#;
        let catalog: CatalogResponse = serde_json::from_str(json).expect("Failed to deserialize current benchmark values");
        let model = catalog.models.get("xai/grok-4.3").expect("model should deserialize");
        let benchmarks = model.benchmarks.as_ref().expect("benchmarks should deserialize").as_slice();
        match benchmarks[0].metric.as_ref() {
            | Some(Metric::Other(value)) => assert_eq!(value, "index score"),
            | other => panic!("expected new metric label to deserialize as Metric::Other, got {other:?}"),
        }
        match benchmarks[1].metric.as_ref() {
            | Some(Metric::Other(value)) => assert_eq!(value, "Elo"),
            | other => panic!("expected new metric label to deserialize as Metric::Other, got {other:?}"),
        }
    }

    #[test]
    fn test_model_row_infers_quantization_from_model_id() {
        use crate::schema::agent::{Quantization, Weights};
        let model = ModelDetails {
            id: Some("llama-3-2-3b-instruct-q4_k_m".to_string()),
            weights: None,
            ..Default::default()
        };
        let row = ModelRow::from(model);
        let weights_json = row.weights.expect("expected inferred weights for model identifier");
        let weights: Weights = serde_json::from_str(&weights_json).expect("weights JSON should deserialize");
        assert_eq!(weights.0.len(), 1);
        assert_eq!(weights.0[0].quantization, Some(Quantization::Q4kM));
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
    #[test]
    fn test_provider_row_models_are_comma_separated() {
        let provider = ProviderDetails {
            id: Some("openai".to_string()),
            name: Some("OpenAI".to_string()),
            models: Some(vec![
                ModelDetails {
                    id: Some("o3".to_string()),
                    ..Default::default()
                },
                ModelDetails {
                    id: Some("gpt-4o".to_string()),
                    ..Default::default()
                },
                ModelDetails {
                    id: Some("o3-mini".to_string()),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };
        let row = ProviderRow::from(provider);
        assert_eq!(row.models.as_deref(), Some("o3,gpt-4o,o3-mini"));
    }
    #[test]
    fn test_provider_row_models_omitted_when_empty() {
        let provider = ProviderDetails {
            id: Some("openai".to_string()),
            models: Some(vec![]),
            ..Default::default()
        };
        let row = ProviderRow::from(provider);
        assert!(row.models.is_none());
    }
    #[test]
    fn test_provider_row_models_omitted_when_none() {
        let provider = ProviderDetails {
            id: Some("openai".to_string()),
            models: None,
            ..Default::default()
        };
        let row = ProviderRow::from(provider);
        assert!(row.models.is_none());
    }
    #[test]
    fn test_provider_row_env_is_comma_separated() {
        let provider = ProviderDetails {
            id: Some("openai".to_string()),
            env: Some(vec!["OPENAI_API_KEY".to_string(), "OPENAI_ORG_ID".to_string()]),
            ..Default::default()
        };
        let row = ProviderRow::from(provider);
        assert_eq!(row.env.as_deref(), Some("OPENAI_API_KEY,OPENAI_ORG_ID"));
    }
    #[test]
    fn test_provider_row_env_omitted_when_empty() {
        let provider = ProviderDetails {
            id: Some("openai".to_string()),
            env: Some(vec![]),
            ..Default::default()
        };
        let row = ProviderRow::from(provider);
        assert!(row.env.is_none());
    }
    #[test]
    fn test_model_row_modalities_are_comma_separated() {
        use crate::schema::research_activity::aspect::data::Modality;
        let model = ModelDetails {
            id: Some("gpt-4o".to_string()),
            modalities: Some(crate::schema::agent::Modalities {
                input: vec![Modality::Text, Modality::Audio, Modality::Image],
                output: vec![Modality::Text, Modality::Video],
            }),
            ..Default::default()
        };
        let row = ModelRow::from(model);
        assert_eq!(row.modality_input.as_deref(), Some("text,audio,image"));
        assert_eq!(row.modality_output.as_deref(), Some("text,video"));
    }
    #[test]
    fn test_model_row_modalities_omitted_when_empty() {
        let model = ModelDetails {
            id: Some("gpt-4o".to_string()),
            modalities: Some(crate::schema::agent::Modalities {
                input: vec![],
                output: vec![],
            }),
            ..Default::default()
        };
        let row = ModelRow::from(model);
        assert!(row.modality_input.is_none());
        assert!(row.modality_output.is_none());
    }
    #[test]
    fn test_catalog_provider_preserves_model_ids_in_row() {
        let json = read_file(fixture_path("").join("catalog.json")).expect("Failed to read catalog fixture");
        let catalog: CatalogResponse = serde_json::from_str(&json).expect("Failed to deserialize catalog fixture");
        let openai = catalog.providers.get("openai").expect("Missing openai provider");
        let row = ProviderRow::from(openai.clone());
        let models = row.models.expect("openai provider should have models");
        let model_ids: Vec<&str> = models.split(',').collect();
        assert!(model_ids.contains(&"o3"), "models should contain o3, got: {models}");
        assert!(model_ids.contains(&"gpt-4o"), "models should contain gpt-4o, got: {models}");
    }
}
