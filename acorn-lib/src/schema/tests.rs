use crate::schema::validate::*;
use crate::schema::*;
use glob::glob;
use pretty_assertions::assert_eq;

#[test]
fn test_metadata() {
    const DEFAULT: &str = "00.png";
    let meta = Metadata::init().id("test-data".to_string()).build();
    assert_eq!(meta.id, "test-data".to_string());
    assert_eq!(meta.schema_type, SchemaType::Project);
    assert_eq!(meta.get_first_graphic_href(), DEFAULT);
    let href = "abc.png".to_string();
    let graphics = vec![Graphic {
        href: Some(href.clone()),
        caption: "hello world".to_string(),
    }];
    let meta = Metadata::init().id("test-data".to_string()).graphics(graphics).build();
    assert_eq!(meta.get_first_graphic_href(), href);
    let meta = Metadata::init().id("test-data".to_string()).graphics(vec![]).build();
    assert_eq!(meta.get_first_graphic_href(), DEFAULT);
    let graphics = vec![Graphic {
        href: None,
        caption: "hello world".to_string(),
    }];
    let meta = Metadata::init().id("test-data".to_string()).graphics(graphics).build();
    assert_eq!(meta.get_first_graphic_href(), DEFAULT);
}
#[test]
fn test_organization() {
    let data = Organization::load();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].name, "Oak Ridge National Laboratory".to_string());
}
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
    let exact: String = "nuclear-nonproliferation".into();
    assert_eq!(resolve_keyword(exact.clone()), Some(exact));
    let misspelled: String = "cyb".into();
    assert_eq!(resolve_keyword(misspelled), Some("cyber".into()));
    let lowercase: String = "machine-learn".into();
    assert_eq!(resolve_keyword(lowercase), Some("machine-learning".into()));
    let lowercase: String = "Automation".into();
    assert_eq!(resolve_keyword(lowercase), Some("automation".into()));
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
    let valid = ["x".repeat(10), "x".repeat(40)];
    let invalid = ["x".repeat(41), "x".repeat(100)];
    for x in valid.iter() {
        assert!(validate_attribute_areas(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_areas(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_attribute_facilities() {
    let valid = ["x".repeat(10), "x".repeat(40)];
    let invalid = ["x".repeat(41), "x".repeat(100)];
    for x in valid.iter() {
        assert!(validate_attribute_facilities(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_facilities(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_attribute_capabilities() {
    let valid = ["x".repeat(10), "x".repeat(300)];
    let invalid = ["x".repeat(301), "x".repeat(400)];
    for x in valid.iter() {
        assert!(validate_attribute_capabilities(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_capabilities(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_attribute_impact() {
    let valid = ["x".repeat(10), "x".repeat(400)];
    let invalid = ["x".repeat(401), "x".repeat(500)];
    for x in valid.iter() {
        assert!(validate_attribute_impact(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_impact(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_attribute_technical_approach() {
    let valid = ["x".repeat(10), "x".repeat(100)];
    let invalid = ["x".repeat(101), "x".repeat(500)];
    for x in valid.iter() {
        assert!(validate_attribute_technical(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_technical(&[x.clone()]).is_err());
    }
}
#[test]
fn test_validate_attribute_outcomes() {
    let valid = [&"x".repeat(10), &"x".repeat(150), "this is a valid outcome"];
    let invalid = [&"x".repeat(151), &"x".repeat(500), "outcome should not be a sentence."];
    for x in valid.iter() {
        assert!(validate_attribute_outcomes(&[x.to_string()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_outcomes(&[x.to_string()]).is_err());
    }
}
#[test]
fn test_validate_is_doi() {
    let valid = ["10.1000/182", "10.97812345/99990"];
    let invalid = ["978-12345-99990"];
    for x in valid.iter() {
        assert!(is_doi(x).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_doi(x).is_err());
    }
}
#[test]
fn test_validate_is_ip6() {
    let valid = [
        "2001:0db8:85a3:0000:0000:8a2e:0370:7334",
        "FE80:0000:0000:0000:0202:B3FF:FE1E:8329",
        "::",
        "::::",
    ];
    let invalid = ["192.168.1.1", "127.0.0.1", "test:test:test:test:test:test:test:test"];
    for x in valid.iter() {
        assert!(is_ip6(x).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_ip6(x).is_err());
    }
}
#[test]
fn test_validate_is_kebabcase() {
    let valid = ["this-is-valid", "thisisvalid"];
    let invalid = ["this_is_not_valid", "ThisIsNotValid", "this is not valid", "This-Is-Not-Valid"];
    for x in valid.iter() {
        assert!(is_kebabcase(x).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_kebabcase(x).is_err());
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
        assert!(is_phone_number(x).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_phone_number(x).is_err());
    }
}
#[test]
fn test_validate_is_ror() {
    // Downloaded from https://zenodo.org/records/14728473
    let valid = [
        "04ttjf776",
        "01rxfrp27",
        "023q4bk22",
        "006jxzx88",
        "00wfvh315",
        "05ktbsm52",
        "00nx6aa03",
        "02k3cxs74",
        "046fa4y88",
        "02d439m40",
        "04h08p482",
        "01zctcs90",
        "05m7zw681",
        "03awtex73",
        "00kv9pj15",
        "03mjtdk61",
        "04yyp8h20",
        "022rkxt86",
        "01wddqe20",
        "03kwrfk72",
        "037mpqg03",
        "05cggb038",
        "050b31k83",
        "01q8f6705",
        "03t40cz74",
        "03ebg0v16",
    ];
    let invalid = [
        "978-12345-99990",
        "10.1000/182",
        "10.97812345/99990",
        "3t40cz74",
        "03ebg0v16-",
        "https://ror.org/04ttjf776",
        "https://ror.org/01rxfrp27",
    ];
    for x in valid.iter() {
        assert!(is_ror(x).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_ror(x).is_err());
    }
}
