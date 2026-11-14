use crate::codec::DocumentCodec;
use crate::research_activity::ResearchActivity;
use crate::validation::Validate;
use acorn_core::util::MimeType;
use serde_json::{json, Value};

fn baseline() -> Value {
    let activity: ResearchActivity = serde_json::from_str(include_str!("../../../../../tests/fixtures/data/valid_project/index.json")).unwrap();
    serde_json::to_value(activity).unwrap()
}
fn replace(pointer: &str, value: Value) -> ResearchActivity {
    let mut document = baseline();
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    document.pointer_mut(parent).unwrap()[key] = value;
    serde_json::from_value(document).unwrap()
}

#[test]
fn test_aspect_rejects_blank_license_and_unknown_fields() {
    for kind in ["Real", "Synthetic"] {
        let data = json!({kind:{"license":" "}});
        let report = replace("/aspect", json!({"data":[data]})).validate().unwrap_err();
        assert!(report.iter().any(|issue| issue.path == format!("aspect.data[0].{kind}.license")));
        let mut document = baseline();
        document["aspect"] = json!({"data":[{kind:{"unexpected":true}}]});
        assert!(ResearchActivity::decode(&document.to_string(), &MimeType::Json).is_err());
        assert!(replace("/aspect", json!({"data":[{kind:{"license":"MIT"}}]})).validate().is_ok());
    }
}
#[test]
fn test_computation_ports_validate_nested_data() {
    for direction in ["inputs", "outputs"] {
        let activity = replace(
            "/aspect",
            json!({"computation":[{
                "identifier":"00000000-0000-4000-8000-000000000001", "stages":[{
                    "identifier":"00000000-0000-4000-8000-000000000002", "operation":"classical",
                    direction:[{"name":"00000000-0000-4000-8000-000000000003", "data":{"Real":{"license":" "}}}]
                }]
            }]}),
        );
        let report = activity.validate().unwrap_err();
        assert!(
            report
                .iter()
                .any(|issue| issue.path == format!("aspect.computation[0].stages[0].{direction}[0].data.Real.license")),
            "{report:?}"
        );
    }
}
#[test]
fn test_contact_collections_and_awards_reject_blank_text() {
    for (pointer, value, path) in [
        ("/contact/givenName", json!(" "), "contact.givenName"),
        ("/contact/familyName", json!(" "), "contact.familyName"),
        ("/contact/jobTitle", json!(" "), "contact.jobTitle"),
        ("/contact/organization", json!(" "), "contact.organization"),
        ("/sections/achievement", json!([" "]), "sections.achievement[0]"),
        ("/sections/approach", json!([" "]), "sections.approach[0]"),
        ("/sections/capabilities", json!([" "]), "sections.capabilities[0]"),
        ("/sections/impact", json!([" "]), "sections.impact[0]"),
        ("/sections/research/areas", json!([" "]), "sections.research.areas[0]"),
        (
            "/meta/outputs/0/funding/0/awards/0/identifier",
            json!(" "),
            "meta.outputs[0].funding[0].awards[0].identifier",
        ),
        ("/meta/spatialCoverage", json!([{"id":"", "place":[]}]), "meta.spatialCoverage[0].id"),
    ] {
        let report = replace(pointer, value).validate().unwrap_err();
        assert!(report.iter().any(|issue| issue.path == path), "{pointer}: {report:?}");
    }
}
#[test]
fn test_dated_roles_compare_instants() {
    for (start, end, valid) in [
        ("2026-01-02T00:00:00Z", "2026-01-01T00:00:00Z", false),
        ("2026-01-01T00:00:00Z", "2026-01-01T00:00:00Z", true),
        ("2026-01-01T02:00:00+02:00", "2026-01-01T01:00:00Z", true),
        ("2026-01-01T01:00:00Z", "2026-01-01T02:00:00+02:00", false),
    ] {
        let identifier = json!({"value":"https://ror.org/01qz5mb56", "scheme":"ROR"});
        for (field, role_key) in [("organizations", "organization"), ("contributors", "agent")] {
            let role = json!({role_key:identifier.clone(), "startDate":start, "endDate":end});
            let mut agent = json!({"name":"Agent", "identifiers":[], "roles":[role]});
            if field == "contributors" {
                agent["affiliations"] = json!([]);
            }
            let activity = replace(&format!("/meta/{field}"), json!([agent]));
            assert_eq!(activity.validate().is_ok(), valid, "{field}: {start}–{end}");
        }
    }
}
#[test]
fn test_decode_rejects_duplicate_fields_and_trailing_content() {
    let document = baseline().to_string();
    let duplicate = document.replacen('{', "{\"title\":\"Duplicate title\",", 1);
    assert!(ResearchActivity::decode(&duplicate, &MimeType::Json).is_err());
    assert!(ResearchActivity::decode(&format!("{document} {{}}"), &MimeType::Json).is_err());
}
#[test]
fn test_decode_reports_real_errors_without_missing_optional_fields() {
    let mut document = baseline();
    document["unexpected"] = json!(true);
    let errors = ResearchActivity::decode(&document.to_string(), &MimeType::Json).unwrap_err();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].path.as_deref(), Some("unexpected"));
    let mut document = baseline();
    document["title"] = json!(42);
    document["meta"]["identifier"] = json!(false);
    let errors = ResearchActivity::decode(&document.to_string(), &MimeType::Json).unwrap_err();
    for path in ["title", "meta.identifier"] {
        assert!(errors.iter().any(|error| error.path.as_deref() == Some(path)), "{errors:?}");
    }
    assert!(errors.iter().all(|error| !error.message.contains("missing field")), "{errors:?}");
}
#[test]
fn test_geographic_bounds_allow_antimeridian_but_not_reversed_latitudes() {
    for (south, north, valid) in [(50, 20, false), (20, 50, true), (20, 20, true)] {
        let activity = replace(
            "/meta/outputs/0/geoLocations",
            json!([{"geoLocationBox":{
                "westBoundLongitude":170, "eastBoundLongitude":-170, "southBoundLatitude":south, "northBoundLatitude":north
            }}]),
        );
        assert_eq!(activity.validate().is_ok(), valid);
    }
}
#[test]
fn test_identifiers_are_unique_in_their_scope() {
    let platform = json!({"identifier":"platform", "name":"Platform", "resources":["CPU"]});
    let environment = json!({"identifier":"environment", "name":"Environment", "platforms":[platform.clone()]});
    let computation = json!({"identifier":"00000000-0000-4000-8000-000000000001", "stages":[{
        "identifier":"00000000-0000-4000-8000-000000000002", "operation":"classical"
    }]});
    for (pointer, value, path) in [
        (
            "/meta/infrastructure",
            json!([environment.clone(), environment.clone()]),
            "meta.infrastructure[1].identifier",
        ),
        (
            "/meta/infrastructure",
            json!([{"identifier":"env", "name":"Env", "platforms":[platform.clone(),platform]}]),
            "meta.infrastructure[0].platforms[1].identifier",
        ),
        (
            "/aspect",
            json!({"computation":[computation.clone(),computation]}),
            "aspect.computation[1].identifier",
        ),
    ] {
        let report = replace(pointer, value).validate().unwrap_err();
        assert!(
            report.iter().any(|issue| issue.path == path && issue.error.code == "duplicate"),
            "{report:?}"
        );
    }
    let mut other = environment.clone();
    other["identifier"] = json!("other-environment");
    assert!(replace("/meta/infrastructure", json!([environment, other])).validate().is_ok());
}
#[test]
fn test_language_codes_use_the_registry() {
    for (code, valid) in [
        ("eng", true),
        ("fra", true),
        ("jpn", true),
        ("ENG", true),
        ("sqi", true),
        ("und", true),
        ("qaa", false),
        ("!!!", false),
        ("zzz", false),
        ("en", false),
    ] {
        assert_eq!(replace("/meta/languages", json!([code])).validate().is_ok(), valid, "{code}");
    }
}
#[test]
fn test_logbook_entries_belong_to_the_containing_project() {
    for (project, valid) in [("valid-project", true), ("different-project", false)] {
        let activity = replace(
            "/logbook",
            json!({"entries":[{
                "identifier":"entry", "projectId":project, "timestamp":"2026-01-01T00:00:00Z",
                "category":"research", "state":"completed", "summary":"Research completed"
            }]}),
        );
        assert_eq!(activity.validate().is_ok(), valid, "{project}");
    }
}
#[test]
fn test_logbook_status_updates_belong_to_the_containing_project() {
    for (project, valid) in [("valid-project", true), ("different-project", false)] {
        let activity = replace(
            "/logbook",
            json!({"entries":[{
                "identifier":"status", "projectId":project, "timestamp":"2026-01-01T00:00:00Z",
                "range":{"left":"2025-12-01T00:00:00Z", "right":"2026-01-01T00:00:00Z"},
                "summary":"Research progressing", "significance":"Milestone delivered", "health":{"status":"on-track"}
            }]}),
        );
        assert_eq!(activity.validate().is_ok(), valid, "{project}");
    }
}
#[test]
fn test_prose_limits_count_characters() {
    for (pointer, limit) in [
        ("/sections/approach", 150),
        ("/sections/capabilities", 300),
        ("/sections/impact", 150),
        ("/sections/research/areas", 40),
    ] {
        assert!(replace(pointer, json!(["É".repeat(limit)])).validate().is_ok(), "{pointer}");
        assert!(replace(pointer, json!(["É".repeat(limit + 1)])).validate().is_err(), "{pointer}");
    }
}
#[test]
fn test_web_locations_require_http_authority() {
    for pointer in [
        "/contact/url",
        "/meta/websites/0/url",
        "/meta/outputs/0/access/url",
        "/meta/outputs/0/funding/0/awards/0/url",
    ] {
        for (value, valid) in [
            ("https:", false),
            ("https:///record", false),
            ("https://@/", false),
            ("https://:8080/", false),
            ("javascript:alert(1)", false),
            ("https://example.org/record", true),
            ("HTTPS://example.org/record", true),
            ("http://[::1]:8080/record", true),
            ("http://localhost:8080/record", true),
        ] {
            assert_eq!(replace(pointer, json!(value)).validate().is_ok(), valid, "{pointer}: {value}");
        }
    }
    assert!(replace("/meta/publications", json!(["https:"])).validate().is_err());
}
#[test]
fn test_yaml_round_trip_preserves_hardware_resources() {
    let original = ResearchActivity::decode(&baseline().to_string(), &MimeType::Json).unwrap();
    let yaml = original.encode(&MimeType::Yaml).unwrap();
    let decoded = ResearchActivity::decode(&yaml, &MimeType::Yaml).unwrap();
    assert_eq!(serde_json::to_value(&decoded).unwrap(), serde_json::to_value(&original).unwrap());
    assert!(decoded.validate().is_ok());
}
