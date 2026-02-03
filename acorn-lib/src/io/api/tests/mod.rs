use crate::io::api::Endpoint;
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
