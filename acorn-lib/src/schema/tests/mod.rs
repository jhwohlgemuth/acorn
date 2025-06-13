use crate::schema::validate::*;
use crate::schema::*;
use glob::glob;
use pretty_assertions::assert_eq;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_classification() {
    assert_eq!(ClassificationLevel::Secret.to_string(), "SECRET");
    assert_eq!(ClassificationLevel::TopSecret.to_string(), "TOP SECRET");
    assert!(ClassificationLevel::Unclassified < ClassificationLevel::Confidential);
    assert!(ClassificationLevel::Confidential < ClassificationLevel::Secret);
    assert!(ClassificationLevel::Secret < ClassificationLevel::TopSecret);
}
#[test]
fn test_format_phone_number() {
    let no_country_code = ["555-123-4567", "555.123.4567", "(555) 123-4567"];
    for x in no_country_code.iter() {
        assert_eq!(format_phone_number(x), Ok("555.123.4567".to_string()));
    }
    let with_country_code = ["+1 (555) 123-4567", "+1 (555) 123.4567", "+15551234567"];
    for x in with_country_code.iter() {
        assert_eq!(format_phone_number(x), Ok("+1.555.123.4567".to_string()));
    }
}
#[test]
fn test_fuzzy_matching_keywords() {
    let name = "keywords".to_string();
    let exact: String = "critical-infrastructure".into();
    let misspelled: String = "cyb".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    assert_eq!(resolve_from_csv_asset(name.clone(), misspelled), Some("cyber".into()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "machine-learn".into()),
        Some("machine-learning".into())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), "Automation".into()), Some("automation".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "math".into()), Some("mathematics".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "mathematics".into()), Some("mathematics".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "ml".into()), Some("machine-learning".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "ai".into()), Some("artificial-intelligence".into()));
    let exact: String = "high-performance-computing".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "hpc".into()), Some(exact.clone()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "high-performance-computi".into()),
        Some(exact.clone())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
}
#[test]
fn test_fuzzy_matching_organizations() {
    let exact: String = "Oak Ridge National Laboratory".into();
    assert_eq!(resolve_from_organization_json(exact.clone()), Some(exact.clone()));
    assert_eq!(resolve_from_organization_json("ORNL".into()), Some(exact.clone()));
    assert_eq!(resolve_from_organization_json("Oak Ridge National Laborato".into()), Some(exact.clone()));
    let exact: String = "Geospatial Science and Human Security Division".into();
    assert_eq!(resolve_from_organization_json("GSHS".into()), Some(exact.clone()));
    assert_eq!(resolve_from_organization_json(exact.clone()), Some(exact.clone()));
    assert_eq!(
        resolve_from_organization_json("Geospatial Science & Human Security".into()),
        Some(exact.clone())
    );
    assert_eq!(
        resolve_from_organization_json("Geospatial Science and Human Security".into()),
        Some(exact.clone())
    );
    assert_eq!(
        resolve_from_organization_json("Geospatial Science and Human Security".to_lowercase()),
        Some(exact.clone())
    );
    assert_eq!(resolve_from_organization_json("PSD".into()), Some("Physical Sciences Directorate".into()));
}
#[test]
fn test_fuzzy_matching_partners() {
    let name = "partners".to_string();
    let exact: String = "National Renewable Energy Laboratory".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "NREL".into()), Some(exact.clone()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "National Renewable Energy Lab".into()),
        Some(exact.clone())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Indiana University".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "iu".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Kitware Inc".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "kitware".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "Kitware, Inc.".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
}
#[test]
fn test_fuzzy_matching_sponsors() {
    let name = "sponsors".to_string();
    let exact: String = "Oak Ridge National Laboratory".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "ORNL".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Department of Energy".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "Dept of Energy".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "Dept. of Energy".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Department of Homeland Security".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "DHS".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Environmental Protection Agency".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), " epa".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Office of Intelligence and Counterintelligence".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "DOE-IN".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Defense Advanced Research Projects Agency".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "darpa".into()), Some(exact.clone()));
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    let exact: String = "Office of Energy Efficiency and Renewable Energy".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), "eere".into()), Some(exact.clone()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "Energy Efficiency and Renewable Energy".into()),
        Some(exact.clone())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
}
#[test]
fn test_fuzzy_matching_techology() {
    let name = "technology".to_string();
    let exact: String = "react".into();
    assert_eq!(resolve_from_csv_asset(name.clone(), exact.clone()), Some(exact));
    assert_eq!(resolve_from_csv_asset(name.clone(), "astro".into()), Some("astro".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "CSS".into()), Some("css".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "React.js".into()), Some("react".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "ReactJS".into()), Some("react".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "rs".into()), Some("rust".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "jl".into()), Some("julia".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "VHDL".into()), Some("vhdl".into()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "programming_language::Rust".into()),
        Some("rust".into())
    );
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "programming_language::R_language".into()),
        Some("r".into())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), "r".into()), Some("r".into()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "Geospatial Data Abstraction Lib".into()),
        Some("gdal".into())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), "node.js".into()), Some("javascript".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "js".into()), Some("javascript".into()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "language::JavaScript".into()),
        Some("javascript".into())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), "bash".into()), Some("shell".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "fish".into()), Some("shell".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "pwsh".into()), Some("shell".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "zsh".into()), Some("shell".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "foobarbaz".into()), None);
    assert_eq!(resolve_from_csv_asset(name.clone(), "".into()), None);
}
#[test]
fn test_metadata() {
    const DEFAULT_HREF: &str = "00.png";
    const DEFAULT_CAPTION: &str = "";
    let meta = Metadata::init().identifier("test-data".to_string()).build();
    assert_eq!(meta.identifier, "test-data".to_string());
    assert_eq!(meta.schema_type, SchemaType::Project);
    assert_eq!(meta.get_first_graphic_href(), DEFAULT_HREF);
    let href = "abc.png";
    let caption = "hello world";
    let graphics = vec![Graphic {
        href: Some(href.to_owned()),
        caption: caption.to_owned(),
    }];
    let meta = Metadata::init().identifier("test-data".to_string()).graphics(graphics).build();
    assert_eq!(meta.clone().get_first_graphic_href(), href);
    assert_eq!(meta.get_first_graphic_caption(), caption);
    let meta = Metadata::init().identifier("test-data".to_string()).graphics(vec![]).build();
    assert_eq!(meta.clone().get_first_graphic_href(), DEFAULT_HREF);
    assert_eq!(meta.get_first_graphic_caption(), DEFAULT_CAPTION);
    let graphics = vec![Graphic {
        href: None,
        caption: "".to_string(),
    }];
    let meta = Metadata::init().identifier("test-data".to_string()).graphics(graphics).build();
    assert_eq!(meta.clone().get_first_graphic_href(), DEFAULT_HREF);
    assert_eq!(meta.get_first_graphic_caption(), DEFAULT_CAPTION);
}
#[test]
fn test_organization() {
    const ORNL_ORGANIZATION_COUNT: usize = 368;
    let data = Organization::load();
    let ornl = data[0].clone();
    assert_eq!(data.len(), 1);
    assert_eq!(ornl.name, "Oak Ridge National Laboratory".to_string());
    let ornl_members = ornl.clone().get_members();
    assert_eq!(ornl_members.len(), ORNL_ORGANIZATION_COUNT);
    let directorates = ornl_members
        .into_iter()
        .filter(|Organization { additional_type, .. }| *additional_type == OrganizationType::Directorate)
        .collect::<Vec<Organization>>();
    assert_eq!(directorates.len(), 8);
    let center = ornl.get_member("Quantum Science Center").unwrap();
    assert_eq!(center.alternative_name, Some("QSC".to_string()));
    assert!(OrganizationType::Group.order() < OrganizationType::Division.order());
}
#[test]
fn test_organization_graph() {
    let nssd = "National Security Sciences Directorate";
    let gshs = "Geospatial Science and Human Security Division";
    let nssd_group = "Spatial Statistics";
    let ornl = Organization::load()[0].clone();
    let group = ornl.clone().get_member(nssd_group).unwrap();
    assert_eq!(group.name, nssd_group);
    assert_eq!(group.clone().get_nearest(OrganizationType::Group).unwrap().name, nssd_group);
    assert_eq!(group.clone().get_nearest(OrganizationType::Division).unwrap().name, gshs);
    assert_eq!(group.clone().get_nearest(OrganizationType::Directorate).unwrap().name, nssd);
    assert_eq!(group.get_nearest(OrganizationType::Ffrdc).unwrap().name, ornl.name);
    assert!(ornl.clone().get_nearest(OrganizationType::Group).is_none());
    assert!(ornl.get_nearest(OrganizationType::Directorate).is_none());
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
fn test_research_activity_format() {
    let path = Some(PathBuf::from(FIXTURES).join("data/format/"));
    //
    // with changes
    //
    let pre = ResearchActivity::read(PathBuf::from(FIXTURES).join("data/format/changes.json")).unwrap();
    assert!(pre.meta.graphics.is_none());
    assert!(pre.contact.affiliation.is_none());
    let post = pre.format(path.clone());
    assert_eq!(post.meta.graphics.as_ref().unwrap()[0].href, Some("00.png".to_string()));
    assert_eq!(post.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
    //
    // with unresolved changes
    //
    let pre = ResearchActivity::read(PathBuf::from(FIXTURES).join("data/format/unresolved_changes.json")).unwrap();
    assert!(pre.meta.graphics.is_some());
    assert_eq!(pre.contact.affiliation, Some("Not an actual affiliation".to_string()));
    let post = pre.format(None);
    assert_eq!(post.meta.graphics.as_ref().unwrap()[0].href, Some("00.png".to_string()));
    assert_eq!(post.meta.graphics.as_ref().unwrap()[0].caption, "".to_string());
    assert_eq!(post.contact.affiliation, Some("Oak Ridge National Laboratory".to_string()));
    //
    // with more unresolved changes
    //
    let mut pre = post.clone().copy();
    pre.contact.organization = "Not an actual organization".to_string();
    pre.contact.affiliation = None;
    let post = pre.format(None);
    assert_eq!(post.contact.organization, "".to_string());
    assert_eq!(post.contact.affiliation, Some("Oak Ridge National Laboratory".to_string()));
    //
    // with ORNL as organization and no affiliation
    //
    let mut pre = post.clone().copy();
    pre.contact.organization = "Oak Ridge National Laboratory".to_string();
    pre.contact.affiliation = None;
    let post = pre.format(None);
    assert_eq!(post.contact.organization, "Oak Ridge National Laboratory".to_string());
    assert_eq!(post.contact.affiliation, Some("Oak Ridge National Laboratory".to_string()));
    //
    // without changes
    //
    let pre = ResearchActivity::read(PathBuf::from(FIXTURES).join("data/format/no_changes.json")).unwrap();
    assert_eq!(pre.meta.graphics.as_ref().unwrap()[0].href, Some("00.png".to_string()));
    assert_eq!(pre.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
    let post = pre.format(path);
    assert_eq!(post.meta.graphics.as_ref().unwrap()[0].href, Some("00.png".to_string()));
    assert_eq!(post.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
}
#[test]
fn test_trl() {
    assert_eq!(TechnologyReadinessLevel::Principles, serde_json::from_str("0").unwrap());
    assert_eq!(TechnologyReadinessLevel::MissionCapable, serde_json::from_str("9").unwrap());
    assert_eq!(TechnologyReadinessLevel::MissionCapable.to_string(), "Mission Capable");
    assert!(TechnologyReadinessLevel::Research < TechnologyReadinessLevel::MissionCapable);
}
#[test]
#[should_panic]
fn test_trl_panic() {
    assert_eq!(TechnologyReadinessLevel::Principles, serde_json::from_str("10").unwrap());
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
    let valid = ["x".repeat(10), "x".repeat(50)];
    let invalid = ["x".repeat(51), "x".repeat(100)];
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
    let valid = ["x".repeat(10), "x".repeat(150)];
    let invalid = ["x".repeat(151), "x".repeat(500)];
    for x in valid.iter() {
        assert!(validate_attribute_impact(&[x.clone()]).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_impact(&[x.clone()]).is_err());
    }
    assert!(validate_attribute_impact(&[
        "This is an impact statement with no period".to_string(),
        "This is another impact statement with no period".to_string(),
        "This is a third impact statement with no period".to_string(),
    ])
    .is_ok());
    assert!(validate_attribute_impact(&[
        "This is an impact statement with no period".to_string(),
        "This is another impact statement with no period".to_string(),
        "This is an impact statement with a period.".to_string(),
    ])
    .is_err());
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
fn test_validate_has_image_extension() {
    let valid = [
        "foo.png",
        "bar.jpg",
        "baz.jpeg",
        "qux.svg",
        "https://example.com/foo.PNG",
        "https://example.com/bar.JPEG",
    ];
    let invalid = ["foo.pNg", "bar.jpx", "qux.sVG", "https://example.com/fooPNG"];
    for x in valid.iter() {
        assert!(
            has_image_extension(x).is_ok(),
            "=> [REASON] \"{x}\" does NOT HAVE a valid image extension"
        );
    }
    for x in invalid.iter() {
        assert!(has_image_extension(x).is_err(), "=> [REASON] \"{x}\" HAS a valid image extension");
    }
}
#[test]
fn test_validate_is_doi() {
    let valid = ["10.1000/182", "10.97812345/99990"];
    let invalid = ["978-12345-99990"];
    for x in valid.iter() {
        assert!(is_doi(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid DOI");
    }
    for x in invalid.iter() {
        assert!(is_doi(x).is_err(), "=> [REASON] \"{x}\" IS a valid DOI");
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
        assert!(is_ip6(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid IP6 address");
    }
    for x in invalid.iter() {
        assert!(is_ip6(x).is_err(), "=> [REASON] \"{x}\" IS a valid IP6 address");
    }
}
#[test]
fn test_validate_is_kebabcase() {
    let valid = ["this-is-valid", "thisisvalid"];
    let invalid = ["this_is_not_valid", "ThisIsNotValid", "this is not valid", "This-Is-Not-Valid"];
    for x in valid.iter() {
        assert!(is_kebabcase(x).is_ok(), "=> [REASON] \"{x}\" is NOT valid kebab-case");
    }
    for x in invalid.iter() {
        assert!(is_kebabcase(x).is_err(), "=> [REASON] \"{x}\" IS valid kebab-case");
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
    let invalid = [
        // Just a number
        "42",
        // Missing country code sign
        "1 (555) 123-4567",
        // Wrong number of digits
        "1234-567",
        // Missing area code
        "123-4567",
        // Fake
        "555.555.5555",
        // Fake
        "555-555-5555",
    ];
    for x in valid.iter() {
        assert!(is_phone_number(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid phone number");
    }
    for x in invalid.iter() {
        assert!(is_phone_number(x).is_err(), "=> [REASON] \"{x}\" IS a valid phone number");
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
        assert!(is_ror(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid ROR");
    }
    for x in invalid.iter() {
        assert!(is_ror(x).is_err(), "=> [REASON] \"{x}\" IS a valid ROR");
    }
}
