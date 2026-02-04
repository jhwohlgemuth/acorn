use crate::io::api::{orcid_query, Endpoint};
use crate::{Location, Repository, Scheme};

#[test]
fn test_endpoint_from_location() {
    let expected = "https://api.example.com";
    let location = Location::Simple(expected.to_string());
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, Some(expected.to_string()));
    let expected = "https://api.example.com";
    let location = Location::Simple(format!("{expected}:8080/v1/data"));
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, Some(expected.to_string()));
    assert_eq!(endpoint.port, Some(8080));
    let location = Location::Detailed {
        scheme: Scheme::HTTPS,
        uri: "http://api.example.com".to_string(),
    };
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, Some(expected.to_string()));
    let location = Location::Detailed {
        scheme: Scheme::HTTPS,
        uri: "http://api.example.com:8080".to_string(),
    };
    let endpoint: Endpoint = location.into();
    assert_eq!(endpoint.domain, Some(expected.to_string()));
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
    assert_eq!(endpoint.domain, Some(expected.to_string()));
}
#[test]
fn test_orcid_query() {
    let pairs = vec![
        ("given-names", "Jason"),
        ("affiliation-org-name", "Oak Ridge National Laboratory"),
        ("family-name", "Wohlgemuth"),
    ];
    let query = orcid_query(pairs, vec![], None);
    let expected = "?q=given-names:Jason+AND+affiliation-org-name:Oak%20Ridge%20National%20Laboratory+AND+family-name:Wohlgemuth";
    assert_eq!(query, expected);
}
#[test]
fn test_orcid_query_with_invalid_field() {
    let pairs = vec![
        ("invalid-field", "should be skipped"),
        ("given-names", "Trista"),
        ("family-name", "Smith"),
    ];
    let query = orcid_query(pairs, vec![], None);
    let expected = "?q=given-names:Trista+AND+family-name:Smith";
    assert_eq!(query, expected);
}
#[test]
fn test_orcid_query_empty() {
    let pairs: Vec<(&str, &str)> = vec![];
    let query = orcid_query(pairs, vec![], None);
    let expected = "";
    assert_eq!(query, expected);
}
#[test]
fn test_orcid_query_with_invalid_orcid() {
    let pairs = vec![
        ("orcid", "0000-0002-1823-1234"), // Invalid checksum
        ("given-names", "Test"),
    ];
    let query = orcid_query(pairs, vec![], None);
    let expected = "?q=given-names:Test";
    assert_eq!(query, expected);
}
#[test]
fn test_orcid_query_with_valid_orcid() {
    let pairs = vec![
        ("orcid", "0000-0002-1825-0097"), // Valid ORCiD
        ("given-names", "Test"),
    ];
    let query = orcid_query(pairs, vec![], None);
    let expected = "?q=orcid:0000-0002-1825-0097+AND+given-names:Test";
    assert_eq!(query, expected);
}
#[test]
fn test_orcid_query_with_invalid_ror() {
    let pairs = vec![
        ("ror-org-id", "03ebg0v16-"), // Invalid ROR
        ("affiliation-org-name", "Test University"),
    ];
    let query = orcid_query(pairs, vec![], None);
    let expected = "?q=affiliation-org-name:Test%20University";
    assert_eq!(query, expected);
}
#[test]
fn test_orcid_query_with_valid_ror() {
    let pairs = vec![
        ("ror-org-id", "https://ror.org/01qz5mb56"), // Valid ROR
        ("affiliation-org-name", "Oak Ridge National Laboratory"),
    ];
    let query = orcid_query(pairs, vec![], None);
    let expected = "?q=ror-org-id:https%3A%2F%2Fror.org%2F01qz5mb56+AND+affiliation-org-name:Oak%20Ridge%20National%20Laboratory";
    assert_eq!(query, expected);
}
#[test]
fn test_orcid_query_with_valid_output_columns() {
    let pairs = vec![("given-names", "Alice"), ("family-name", "Smith")];
    let columns = vec!["orcid", "email", "credit-name"];
    let query = orcid_query(pairs, columns, None);
    let expected = "?q=given-names:Alice+AND+family-name:Smith&fl=orcid,email,credit-name";
    assert_eq!(query, expected);
}
#[test]
fn test_orcid_query_with_invalid_output_columns() {
    let pairs = vec![("given-names", "Bob")];
    let columns = vec!["orcid", "invalid-column", "email"];
    let query = orcid_query(pairs, columns, None);
    let expected = "?q=given-names:Bob&fl=orcid,email";
    assert_eq!(query, expected);
}
#[test]
fn test_orcid_query_with_mixed_invalid_fields_and_columns() {
    let pairs = vec![
        ("given-names", "Jason"),
        ("invalid-field", "should be ignored"),
        ("family-name", "Wohlgemuth"),
    ];
    let columns = vec!["orcid", "invalid-column", "credit-name", "bad-field"];
    let query = orcid_query(pairs, columns, None);
    let expected = "?q=given-names:Jason+AND+family-name:Wohlgemuth&fl=orcid,credit-name";
    assert_eq!(query, expected);
}
#[test]
fn test_orcid_query_with_all_output_columns() {
    let pairs = vec![("family-name", "Wohlgemuth")];
    let columns = vec![
        "orcid",
        "email",
        "credit-name",
        "given-name",
        "family-name",
        "given-and-family-names",
        "other-name",
        "current-institution-affiliation-name",
        "past-institution-affiliation-name",
    ];
    let query = orcid_query(pairs, columns, None);
    let expected = "?q=family-name:Wohlgemuth&fl=orcid,email,credit-name,given-name,family-name,given-and-family-names,other-name,current-institution-affiliation-name,past-institution-affiliation-name";
    assert_eq!(query, expected);
}
