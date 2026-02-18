use crate::io::api::{self, citeas, orcid, ror, Endpoint, Param, ParamStyle, ResponseContent, RestfulInterface};
use crate::io::read_file;
use crate::schema::pid::{PersistentIdentifierParse, DOI};
use crate::{Location, Repository, Scheme};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("tests/fixtures")
}

#[test]
#[ignore = "Requires internet connection and citeas.org to be up"]
fn test_citeas() {
    let status = citeas::status();
    assert!(status.is_some());
    if let Some(citeas::Status { documentation_url, .. }) = status {
        assert_eq!(documentation_url, "https://citeas.org/api");
    }
    if let Some(citeas::Citation { text, .. }) = citeas::Citations::from_doi("10.11578/dc.20250604.1").match_style("apa") {
        println!("CiteAs Test Response Received");
        let expected = "Wohlgemuth, J. (2025). Accessible Content Optimization for Research Needs (ACORN). Oak Ridge National Laboratory (ORNL), Oak Ridge, TN (United States). http://doi.org/10.11578/DC.20250604.1";
        assert_eq!(text, expected);
    };
    let doi = DOI::from_string("10.11578/dc.20250604.1");
    if let Some(citeas::Citation { text, .. }) = doi.to_citations().match_style("apa") {
        println!("CiteAs Test Response Received");
        let expected = "Wohlgemuth, J. (2025). Accessible Content Optimization for Research Needs (ACORN). Oak Ridge National Laboratory (ORNL), Oak Ridge, TN (United States). http://doi.org/10.11578/DC.20250604.1";
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
fn test_endpoint_context() {
    let params = vec![
        api::Param::of_type(api::ParamStyle::QueryPair)
            .values(vec![
                (Some("affiliation-org-name"), Some("Lyrasis")),
                (Some("ror-org-id"), Some("\"https://ror.org/01qz5mb56\"")),
            ])
            .with_key("q"),
        api::Param::of_type(api::ParamStyle::TemplateValue)
            .values(vec![(Some("two"), None)])
            .with_key("one"),
        api::Param::of_type(api::ParamStyle::TemplateValue)
            .values(vec![(None, Some("four"))])
            .with_key("three"),
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
    let endpoint = Endpoint::at("pub.orcid.org").root("v3.0").build();
    let response = endpoint.parse_xml::<orcid::SearchResponse>(xml).expect("Failed to parse XML");
    assert_eq!(response.num_found, 2);
    assert_eq!(response.namespace, "http://www.orcid.org/ns/expanded-search");
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.results[0].family_names, Some("Carson".to_string()));
    assert_eq!(response.results[1].family_names, Some("Wohlgemuth".to_string()));
    let json = "{\"tomcatUp\":true,\"dbConnectionOk\":true,\"readOnlyDbConnectionOk\":false,\"overallOk\":true}";
    let status = endpoint.parse_json::<orcid::StatusResponse>(json).expect("Failed to parse JSON");
    assert!(status.application);
    assert!(status.database);
    assert!(!status.database_readonly);
    assert!(status.overall);
}
#[test]
fn test_orcid_query_string() {
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
fn test_orcid_response_handler() {
    // Load XML fixture file
    let xml = read_file(fixtures_dir().join("response_orcid_wohlgemuth.xml")).expect("Failed to read XML fixture");
    // Parse XML into OrcidSearchResponse
    let response = api::parse::<orcid::SearchResponse>(&xml).expect("Failed to deserialize XML");
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
fn test_orcid_status_response() {
    let json = r#"{"tomcatUp":true,"dbConnectionOk":true,"readOnlyDbConnectionOk":false,"overallOk":true}"#;
    let response: orcid::StatusResponse = serde_json::from_str(json).expect("Failed to deserialize JSON");
    insta::assert_snapshot!(format!("{:#?}", response));
}
#[test]
fn test_query_params_empty_field() {
    let params = vec![
        Param::of_type(api::ParamStyle::FieldList)
            .values(vec![(Some("Oak Ridge"), None)])
            .with_key("query"),
        Param::of_type(api::ParamStyle::QueryPair)
            .values(vec![(Some("status"), Some("inactive"))])
            .with_key("filter"),
    ];
    let query = Param::to_query_string::<api::EmptyField, api::EmptyField>(params);
    insta::assert_snapshot!(query);
}
#[test]
fn test_query_params_field_list() {
    // Test FieldList style (output columns)
    let param = Param::of_type(ParamStyle::FieldList)
        .values(vec![(Some("orcid"), None), (Some("email"), None), (Some("credit-name"), None)])
        .with_key("fl");
    let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
    let expected = "fl=orcid,email,credit-name";
    assert_eq!(rendered, expected);
    // Test FieldList style with invalid field (should be filtered)
    let param = Param::of_type(ParamStyle::FieldList)
        .values(vec![(Some("orcid"), None), (Some("invalid-column"), None), (Some("email"), None)])
        .with_key("fl");
    let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
    let expected = "fl=orcid,email";
    assert_eq!(rendered, expected);
}
#[test]
fn test_params_query_field() {
    // Test QueryField style (boosted fields)
    let param = Param::of_type(ParamStyle::QueryField)
        .values(vec![(Some("given-names"), None), (Some("family-name"), None)])
        .with_key("qf");
    let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
    let expected = "qf=given-names%5E3.0%20family-name%5E2.0";
    assert_eq!(rendered, expected);
    // Test QueryField style with three fields
    let param = Param::of_type(ParamStyle::QueryField)
        .values(vec![
            (Some("given-names"), None),
            (Some("family-name"), None),
            (Some("affiliation-org-name"), None),
        ])
        .with_key("qf");
    let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
    let expected = "qf=given-names%5E4.0%20family-name%5E3.0%20affiliation-org-name%5E2.0";
    assert_eq!(rendered, expected);
    // Test QueryField with invalid field names (should be filtered)
    let param = Param::of_type(ParamStyle::QueryField)
        .values(vec![
            (Some("invalid-field"), None),
            (Some("given-names"), None),
            (Some("another-invalid"), None),
            (Some("family-name"), None),
        ])
        .with_key("qf");
    let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
    let expected = "qf=given-names%5E3.0%20family-name%5E2.0";
    assert_eq!(rendered, expected);
    // Test QueryField with only invalid field names (should return empty)
    let param = Param::of_type(ParamStyle::QueryField)
        .values(vec![(Some("invalid-field"), None), (Some("another-invalid"), None)])
        .with_key("qf");
    let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
    let expected = "";
    assert_eq!(rendered, expected);
}
#[test]
fn test_params_query_pair() {
    // Test QueryPair style
    let param = Param::of_type(ParamStyle::QueryPair)
        .values(vec![(Some("given-names"), Some("Jason")), (Some("family-name"), Some("Wohlgemuth"))])
        .with_key("q");
    let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
    let expected = "q=given-names:Jason+AND+family-name:Wohlgemuth";
    assert_eq!(rendered, expected);
    // Test QueryPair with invalid field names (should be filtered)
    let param = Param::of_type(ParamStyle::QueryPair)
        .values(vec![
            (Some("invalid-field"), Some("value")),
            (Some("given-names"), Some("Jason")),
            (Some("another-invalid"), Some("data")),
        ])
        .with_key("q");
    let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
    let expected = "q=given-names:Jason";
    assert_eq!(rendered, expected);
    // Test QueryPair with invalid field values (invalid ORCiD should be filtered)
    let param = Param::of_type(ParamStyle::QueryPair)
        .values(vec![(Some("orcid"), Some("0000-0002-1823-1234")), (Some("given-names"), Some("Jason"))])
        .with_key("q");
    let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
    let expected = "q=given-names:Jason";
    assert_eq!(rendered, expected);
    // Test QueryPair with valid ORCiD value (should be included)
    let param = Param::of_type(ParamStyle::QueryPair)
        .values(vec![(Some("orcid"), Some("0000-0002-2057-9115")), (Some("given-names"), Some("Jason"))])
        .with_key("q");
    let rendered = param.to_string::<orcid::SearchField, orcid::OutputColumn>();
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
    let query = api::Param::to_query_string::<orcid::SearchField, orcid::OutputColumn>(params);
    let expected = "?q=given-names:Jason+AND+family-name:Wohlgemuth&fl=orcid,email,credit-name&qf=given-names%5E3.0%20family-name%5E2.0";
    assert_eq!(query, expected);
    // Test rendering with empty params (should return empty string)
    let params = vec![];
    let query = api::Param::to_query_string::<orcid::SearchField, orcid::OutputColumn>(params);
    let expected = "";
    assert_eq!(query, expected);
}
#[test]
fn test_ror_search_response() {
    let json = read_file(fixtures_dir().join("response_ror_ornl.json")).expect("Failed to read JSON fixture");
    let response: ror::SearchResponse = serde_json::from_str(&json).expect("Failed to deserialize JSON");
    insta::assert_snapshot!(format!("{:#?}", response));
}
