use crate::schema::*;
use glob::glob;
use pretty_assertions::assert_eq;

#[test]
fn test_read() {
    for entry in glob("./**/*").expect("Failed to read glob pattern") {
        match entry {
            | Ok(path) => println!("{:?}", path.display()),
            | Err(e) => println!("{:?}", e),
        }
    }
}
#[test]
fn test_fuzzy_keywords() {
    let exact: Keyword = "nuclear-nonproliferation".into();
    assert_eq!(resolve_keyword(exact.clone()), Some(exact));
    let misspelled: Keyword = "cyb".into();
    assert_eq!(resolve_keyword(misspelled), Some("cyber".into()));
    let lowercase: Keyword = "gears".into();
    assert_eq!(resolve_keyword(lowercase), Some("GEARS".into()));
}
#[test]
fn test_fuzzy_sponsors() {
    assert_eq!(resolve_sponsor("ORNL".into()), Some("Oak Ridge National Laboratory".into()));
    assert_eq!(resolve_sponsor("Dept of Energy".into()), Some("Department of Energy".into()));
    assert_eq!(resolve_sponsor("Dept. of Energy".into()), Some("Department of Energy".into()));
    assert_eq!(resolve_sponsor("DHS ".into()), Some("Department of Homeland Security".into()));
    assert_eq!(resolve_sponsor(" epa".into()), Some("Environmental Protection Agency".into()));
    assert_eq!(resolve_sponsor("darpa".into()), Some("Defense Advanced Research Projects Agency".into()));
}
#[test]
fn test_fuzzy_technologies() {
    assert_eq!(resolve_technology("astro".into()), Some("astro".into()));
    assert_eq!(resolve_technology("React.js".into()), Some("react".into()));
    assert_eq!(resolve_technology("rs".into()), Some("rust".into()));
    assert_eq!(resolve_technology("Geospatial Data Abstraction Lib".into()), Some("gdal".into()));
    assert_eq!(resolve_technology("foobarbaz".into()), None);
}
#[test]
fn test_validate_attribute_areas() {
    let valid = ["x".repeat(10).to_string(), "x".repeat(40).to_string()];
    let invalid = ["x".repeat(41).to_string(), "x".repeat(100).to_string()];
    for x in valid.iter() {
        assert!(validate_attribute_areas(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_areas(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_attribute_facilities() {
    let valid = ["x".repeat(10).to_string(), "x".repeat(40).to_string()];
    let invalid = ["x".repeat(41).to_string(), "x".repeat(100).to_string()];
    for x in valid.iter() {
        assert!(validate_attribute_facilities(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_facilities(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_attribute_capabilities() {
    let valid = ["x".repeat(10).to_string(), "x".repeat(300).to_string()];
    let invalid = ["x".repeat(301).to_string(), "x".repeat(400).to_string()];
    for x in valid.iter() {
        assert!(validate_attribute_capabilities(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_capabilities(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_attribute_impact() {
    let valid = ["x".repeat(10).to_string(), "x".repeat(400).to_string()];
    let invalid = ["x".repeat(401).to_string(), "x".repeat(500).to_string()];
    for x in valid.iter() {
        assert!(validate_attribute_impact(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_impact(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_attribute_methods() {
    let valid = ["x".repeat(10).to_string(), "x".repeat(75).to_string()];
    let invalid = ["x".repeat(76).to_string(), "x".repeat(500).to_string()];
    for x in valid.iter() {
        assert!(validate_attribute_methods(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_methods(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_attribute_outcomes() {
    let valid = ["x".repeat(10).to_string(), "x".repeat(100).to_string()];
    let invalid = ["x".repeat(101).to_string(), "x".repeat(500).to_string()];
    for x in valid.iter() {
        assert!(validate_attribute_outcomes(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_outcomes(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_is_doi() {
    let valid = ["10.1000/182", "10.97812345/99990"];
    let invalid = ["978-12345-99990"];
    for x in valid.iter() {
        assert!(validate_is_doi(x).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_is_doi(x).is_err());
    }
}
#[test]
fn test_validate_is_kebabcase() {
    let valid = ["this-is-valid", "thisisvalid"];
    let invalid = ["this_is_not_valid", "ThisIsNotValid", "this is not valid", "This-Is-Not-Valid"];
    for x in valid.iter() {
        assert!(validate_is_kebabcase(x).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_is_kebabcase(x).is_err());
    }
}
#[test]
fn test_validate_is_phone_number() {
    let valid = [
        "555-123-4567",
        "555.123.4567",
        "+1 (555) 123-4567",
        "+1 (555) 123.4567",
        "+15551234567",
        "(555) 123-4567",
    ];
    let invalid = ["42", "1 (555) 123-4567", "1234-567", "123-4567"];
    for x in valid.iter() {
        assert!(validate_is_phone_number(x).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_is_phone_number(x).is_err());
    }
}
