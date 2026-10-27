use super::config::Resource;
use super::{
    config::{GrantFlow, Persistence},
    entra::{Cloud, Profile},
};

mod device;
mod identity;

#[test]
fn test_entra_profile_canonicalizes_uuid_case_for_issuer_and_client() {
    let resource = Resource::try_new("https://graph.microsoft.com", ["Files.Read".to_string()]).expect("valid resource");
    let profile = Profile::try_new(
        Cloud::Public,
        "AAAAAAAA-AAAA-AAAA-AAAA-AAAAAAAAAAAA".to_string(),
        GrantFlow::Browser,
        Persistence::Disabled,
        "BBBBBBBB-BBBB-BBBB-BBBB-BBBBBBBBBBBB".to_string(),
        vec![resource],
    )
    .expect("valid profile");
    assert_eq!(profile.client_id(), "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");
    assert_eq!(profile.tenant_id(), "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb");
    assert_eq!(
        profile.issuer().as_str(),
        "https://login.microsoftonline.com/bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb/v2.0"
    );
}
#[test]
fn test_resource_normalizes_compatible_scopes() {
    let resource = Resource::try_new(
        "https://graph.example.test",
        ["Sites.Read.All", "Files.Read", "Files.Read"].map(str::to_string),
    )
    .expect("valid resource");
    assert_eq!(resource.audience().as_str(), "https://graph.example.test/");
    assert_eq!(resource.scopes(), &["Files.Read", "Sites.Read.All"]);
}
#[test]
fn test_resource_rejects_unsafe_audiences_and_empty_scopes() {
    for audience in [
        "http://graph.example.test",
        "https://user:pass@graph.example.test",
        "https://graph.example.test/file",
        "https://graph.example.test/?token=value",
    ] {
        assert!(Resource::try_new(audience, ["Files.Read".to_string()]).is_err(), "{audience}");
    }
    assert!(Resource::try_new("https://graph.example.test", Vec::new()).is_err());
    assert!(Resource::try_new("https://graph.example.test", ["Files Read".to_string()]).is_err());
}
