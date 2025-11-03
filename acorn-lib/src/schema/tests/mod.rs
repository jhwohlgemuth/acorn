use crate::prelude::PathBuf;
use crate::schema::validate::*;
use crate::schema::*;
use glob::glob;
use pretty_assertions::assert_eq;

const FIXTURES: &str = "../tests/fixtures";
const VALID_ARK_VALUES: [&str; 8] = [
    "https://n2t.net/ark:99166/w66d60p2",
    "https://n2t.net/ark:/99166/w66d60p2",
    "ark:1234/w5678",
    "https://n2t.net/ark:12148/btv1b8449691v/f29",
    "https://n2t.net/ark:12148/btv1b8449691v/f29/abc.png",
    "ark:12148/btv1b8449691v/f29/abc.TIFF",
    "https://example.org/ark:12345/x6np1wh8k/c3/s5.v7.xsl",
    // https://gallica.bnf.fr/ark:/12148/btv1b8449691v
    "ark:/12148/btv1b8449691v",
];
const VALID_DOI_VALUES: [&str; 31] = [
    // ACORN DOI
    "10.11578/dc.20250604.1",
    "10.1000/182",
    "10.97812345/99990",
    "https://doi.org/10.11578/dc.20250604.1",
    "https://doi.org/10.1000/182",
    "https://doi.org/10.97812345/99990",
    "10.1038/s41562-018-0399-z",
    "10.1038/533452a",
    "10.1109/eScience51609.2021.00010",
    "10.1109/ACCESS.2025.3542334",
    "10.1016/j.jbi.2008.04.010",
    "10.48550/arXiv.2312.10997",
    "10.1098/rsos.171511",
    "10.3233/ISU-2010-0613",
    "10.48550/arXiv.2411.06237",
    "10.2139/ssrn.4900122",
    "10.18653/v1/2025.naacl-long.243",
    "10.1007/978-3-642-38288-8_33",
    "10.48550/arXiv.2501.07391",
    "10.48550/arXiv.2504.01990",
    "10.1109/I-SMAC61858.2024.10714814",
    "10.48550/arXiv.2506.06576",
    "10.1186/s13326-025-00327-4",
    "10.1162/dint_a_00186",
    "10.1609/aaai.v39i24.34743",
    "10.48550/arXiv.2504.16736",
    "10.1002/asi.22636",
    "10.1093/gigascience/giy023",
    "10.1162/99608f92.e1f349c2",
    "10.5479/10088/113528",
    "10.1038/s41597-020-0486-7",
];
const VALID_ORCID_VALUES: [&str; 14] = [
    // Jason Wohlgemuth
    "https://orcid.org/0000-0002-2057-9115",
    // Audrey Carson
    "https://orcid.org/0009-0005-5568-6526",
    "0000-0003-1485-2741",
    "0009-0005-8526-4332",
    "0009-0001-1431-2393",
    "0009-0007-2591-8394",
    "0009-0006-8870-3625",
    "0009-0004-4438-9406",
    "0009-0003-1201-0767",
    "0000-0002-2845-8668",
    "0000-0002-0014-1319",
    "0000-0001-9034-3389",
    "0000-0002-0065-494X",
    "0000-0002-2816-415X",
];
// Downloaded from https://zenodo.org/records/14728473
const VALID_ROR_VALUES: [&str; 29] = [
    "https://ror.org/01qz5mb56",
    "ror.org/01qz5mb56",
    "01qz5mb56",
    "https://ror.org/04ttjf776",
    "https://ror.org/01rxfrp27",
    "https://ror.org/023q4bk22",
    "https://ror.org/006jxzx88",
    "https://ror.org/00wfvh315",
    "https://ror.org/05ktbsm52",
    "https://ror.org/00nx6aa03",
    "https://ror.org/02k3cxs74",
    "https://ror.org/046fa4y88",
    "https://ror.org/02d439m40",
    "https://ror.org/04h08p482",
    "https://ror.org/01zctcs90",
    "https://ror.org/05m7zw681",
    "https://ror.org/03awtex73",
    "https://ror.org/00kv9pj15",
    "https://ror.org/03mjtdk61",
    "https://ror.org/04yyp8h20",
    "https://ror.org/022rkxt86",
    "https://ror.org/01wddqe20",
    "https://ror.org/03kwrfk72",
    "https://ror.org/037mpqg03",
    "https://ror.org/05cggb038",
    "https://ror.org/050b31k83",
    "https://ror.org/01q8f6705",
    "https://ror.org/03t40cz74",
    "https://ror.org/03ebg0v16",
];

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
    assert_eq!(resolve_from_csv_asset(name.clone(), "stats".into()), Some("statistics".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "statistics".into()), Some("statistics".into()));
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
    assert_eq!(
        resolve_from_organization_json("Research Accelerator Division".into()),
        Some("Research Accelerator Division".into())
    );
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
    assert_eq!(resolve_from_csv_asset(name.clone(), "kt".into()), Some("kotlin".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "Redis Open Source".into()), Some("redis".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "scss".into()), Some("sass".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "TypeSpec".into()), Some("typespec".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "bash".into()), Some("shell".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "fish".into()), Some("shell".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "pwsh".into()), Some("shell".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "zsh".into()), Some("shell".into()));
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "https://airflow.apache.org".into()),
        Some("airflow".into())
    );
    assert_eq!(
        resolve_from_csv_asset(name.clone(), "http://airflow.apache.org".into()),
        Some("airflow".into())
    );
    assert_eq!(resolve_from_csv_asset(name.clone(), "airflow.apache.org".into()), Some("airflow".into()));
    assert_eq!(resolve_from_csv_asset(name.clone(), "foobarbaz".into()), None);
    assert_eq!(resolve_from_csv_asset(name.clone(), "".into()), None);
}
#[test]
fn test_metadata() {
    const DEFAULT_HREF: &str = "00.png";
    const DEFAULT_CAPTION: &str = "";
    let meta = Metadata::init().identifier("test-data".to_string()).build();
    assert_eq!(meta.identifier, "test-data".to_string());
    assert_eq!(meta.first_image_content_url(), DEFAULT_HREF);
    let href = "abc.png";
    let caption = "hello world";
    let graphics = vec![MediaObject::Image(
        ImageObject::init().caption(caption.to_owned()).content_url(href.to_owned()).build(),
    )];
    let meta = Metadata::init().identifier("test-data".to_string()).media(graphics).build();
    assert_eq!(meta.clone().first_image_content_url(), href);
    assert_eq!(meta.first_image_caption(), caption);
    let meta = Metadata::init().identifier("test-data".to_string()).media(vec![]).build();
    assert_eq!(meta.clone().first_image_content_url(), DEFAULT_HREF);
    assert_eq!(meta.first_image_caption(), DEFAULT_CAPTION);
    let graphics = vec![MediaObject::Image(ImageObject::init().caption("".to_owned()).build())];
    let meta = Metadata::init().identifier("test-data".to_string()).media(graphics).build();
    assert_eq!(meta.clone().first_image_content_url(), DEFAULT_HREF);
    assert_eq!(meta.first_image_caption(), DEFAULT_CAPTION);
}
#[test]
fn test_organization() {
    const ORNL_ORGANIZATION_COUNT: usize = 368;
    let data = Organization::load();
    let ornl = data[0].clone();
    assert_eq!(data.len(), 1);
    assert_eq!(ornl.name, "Oak Ridge National Laboratory".to_string());
    let ornl_members = ornl.clone().members();
    assert_eq!(ornl_members.len(), ORNL_ORGANIZATION_COUNT);
    let directorates = ornl_members
        .into_iter()
        .filter(|Organization { additional_type, .. }| *additional_type == OrganizationType::Directorate)
        .collect::<Vec<Organization>>();
    assert_eq!(directorates.len(), 8);
    let center = ornl.member("Quantum Science Center").unwrap();
    assert_eq!(center.alternative_name, Some("QSC".to_string()));
    assert!(OrganizationType::Group.order() < OrganizationType::Division.order());
}
#[test]
fn test_organization_graph() {
    let nssd = "National Security Sciences Directorate";
    let gshs = "Geospatial Science and Human Security Division";
    let nssd_group = "Spatial Statistics";
    let ornl = Organization::load()[0].clone();
    let group = ornl.clone().member(nssd_group).unwrap();
    assert_eq!(group.name, nssd_group);
    assert_eq!(group.clone().nearest(OrganizationType::Group).unwrap().name, nssd_group);
    assert_eq!(group.clone().nearest(OrganizationType::Division).unwrap().name, gshs);
    assert_eq!(group.clone().nearest(OrganizationType::Directorate).unwrap().name, nssd);
    assert_eq!(group.nearest(OrganizationType::Ffrdc).unwrap().name, ornl.name);
    assert!(ornl.clone().nearest(OrganizationType::Group).is_none());
    assert!(ornl.clone().nearest(OrganizationType::Directorate).is_none());
    let division = ornl.member("Research Accelerator Division").unwrap();
    assert_eq!(
        division.nearest(OrganizationType::Directorate).unwrap().name,
        "Neutron Sciences Directorate"
    );
}
#[test]
fn test_read() {
    for entry in glob("./**/*").expect("Failed to read glob pattern") {
        match entry {
            | Ok(path) => println!("{:?}", path.display()),
            | Err(e) => println!("{e:?}"),
        }
    }
}
#[test]
fn test_research_activity_default() {
    let data = ResearchActivity::default();
    assert_eq!(data.to_markdown(), "# Research Activity Title");
}
#[test]
fn test_research_activity_format() {
    //
    // with changes
    //
    let path = Some(PathBuf::from(FIXTURES).join("data/format/changes"));
    let pre = ResearchActivity::read(PathBuf::from(FIXTURES).join("data/format/changes/index.json")).unwrap();
    assert!(pre.meta.media.is_none());
    assert!(pre.contact.affiliation.is_none());
    let post = pre.format(path.clone());
    assert_eq!(post.meta.media.unwrap()[0].clone().content_url(), Some("42.png".to_string()));
    assert_eq!(post.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
    //
    // with unresolved changes
    //
    let pre = ResearchActivity::read(PathBuf::from(FIXTURES).join("data/format/unresolved_changes/index.json")).unwrap();
    assert!(pre.meta.media.is_some());
    assert_eq!(pre.contact.affiliation, Some("Not an actual affiliation".to_string()));
    let post = pre.format(None);
    assert!(post.clone().meta.media.unwrap()[0].clone().content_url().is_none());
    assert_eq!(post.clone().meta.first_image().unwrap().description(), "".to_string());
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
    let path = Some(PathBuf::from(FIXTURES).join("data/format/no_changes"));
    let pre = ResearchActivity::read(PathBuf::from(FIXTURES).join("data/format/no_changes/index.json")).unwrap();
    assert_eq!(pre.clone().meta.media.unwrap()[0].clone().content_url(), Some("00.png".to_string()));
    assert_eq!(pre.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
    let post = pre.clone().format(path);
    assert_eq!(post.meta.media.unwrap()[0].clone().content_url(), Some("42.png".to_string()));
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
        assert!(validate_attribute_areas(core::slice::from_ref(x)).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_areas(core::slice::from_ref(x)).is_err());
    }
}
#[test]
fn test_validate_attribute_capabilities() {
    let valid = ["x".repeat(10), "x".repeat(300)];
    let invalid = ["x".repeat(301), "x".repeat(400)];
    for x in valid.iter() {
        assert!(validate_attribute_capabilities(core::slice::from_ref(x)).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_capabilities(core::slice::from_ref(x)).is_err());
    }
}
#[test]
fn test_validate_attribute_doi() {
    let valid = ["10.1000/182".to_string(), "10.97812345/99990".to_string()];
    assert!(validate_attribute_doi(&valid).is_ok());
    let invalid = ["https://not.doi.org/10.1000/182".to_string()];
    assert!(validate_attribute_doi(&invalid).is_err());
}
#[test]
fn test_validate_attribute_impact() {
    let valid = ["x".repeat(10), "x".repeat(150)];
    let invalid = ["x".repeat(151), "x".repeat(500)];
    for x in valid.iter() {
        assert!(validate_attribute_impact(core::slice::from_ref(x)).is_ok());
    }
    for x in invalid.iter() {
        assert!(validate_attribute_impact(core::slice::from_ref(x)).is_err());
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
fn test_validate_attribute_ror() {
    let valid = VALID_ROR_VALUES.into_iter().map(|x| x.to_string()).collect::<Vec<String>>();
    assert!(validate_attribute_ror(&valid).is_ok());
    let invalid = ["https://doi.org/10.1000/182".to_string()];
    assert!(validate_attribute_ror(&invalid).is_err());
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
fn test_validate_is_ark() {
    let valid = VALID_ARK_VALUES;
    let invalid = [
        "978-12345-99990",
        // HTTP not supported
        "http://n2t.net/ark:99166/w66d60p2",
        // Missing ark: label
        "https://n2t.net/99166/w66d60p2",
        // Shoulder starts with number
        "https://n2t.net/99166/9w66d60p2",
        // Shoulder contains letter "l"
        "https://n2t.net/99166/lw66d60p2",
    ];
    for x in valid.iter() {
        assert!(is_ark(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid ARK");
    }
    for x in invalid.iter() {
        assert!(is_ark(x).is_err(), "=> [REASON] \"{x}\" IS a valid ARK");
    }
    match is_ark("https://n2t.net/99166/w66d60p2") {
        | Ok(_) => panic!(),
        | Err(err) => assert_eq!(err.to_string(), "Please provide a valid ARK"),
    }
}
#[test]
fn test_validate_is_doi() {
    let valid = VALID_DOI_VALUES;
    let invalid = [
        "978-12345-99990",
        // Pretty sure 11 is not a valid DOI directory indicator
        "11.11578/dc.20250604.1",
        "<geo coords=\"10.4515260,51.1656910\"></geo>",
        // 10.5555 is not a DOI prefix, but rather a handle prefix
        "10.5555/182",
    ];
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
fn test_is_iso8601() {
    let valid = ["2000-01-01", "2025-06-04", "1950-12-28"];
    let invalid = [
        // "3000-01-01",
        "1949-01-02",
        "01-02-2023",
        "1234",
        "foo",
        "42",
        "2025/06/04",
    ];
    for x in valid.iter() {
        assert!(is_iso8601_date(x).is_ok(), "=> [REASON] \"{x}\" is NOT valid ISO 8601 date (YYYY-MM-DD)");
    }
    for x in invalid.iter() {
        assert!(is_iso8601_date(x).is_err(), "=> [REASON] \"{x}\" IS valid ISO 8601 date (YYYY-MM-DD)");
    }
    let valid_years = ["2000", "2025", "1950"];
    let invalid_years = ["3000", "1949"];
    for x in valid_years.iter() {
        assert!(is_iso8601_year(x).is_ok(), "=> [REASON] \"{x}\" is NOT valid ISO 8601 year (YYYY)");
    }
    for x in invalid_years.iter() {
        assert!(is_iso8601_year(x).is_err(), "=> [REASON] \"{x}\" IS valid ISO 8601 year (YYYY)");
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
fn test_validate_is_list_url() {
    let valid = [
        "http://www.example.com".to_string(),
        "https://example.com".to_string(),
        "https://example.com#foo".to_string(),
        "https://example.com:1337#foo?bar".to_string(),
        "http://example.com:1337#foo?bar=42".to_string(),
    ];
    let invalid = [
        "http://www.example_com".to_string(),
        "https://examp*le.com".to_string(),
        "https://example.com#foo bar".to_string(),
        "https//example.com:1337#foo?bar".to_string(),
    ];
    assert!(is_list_url(&valid).is_ok());
    assert!(is_list_url(&invalid).is_err());
}
#[test]
fn test_validate_is_orcid() {
    let valid = VALID_ORCID_VALUES;
    let invalid = [
        "0000-0002-1823-1234",
        "https://orcid.org/0000-0002-1823-12345",
        "https://orcid.com/0000-0002-1823-1234",
    ];
    for x in valid.iter() {
        assert!(is_orcid(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid ORCiD");
    }
    for x in invalid.iter() {
        assert!(is_orcid(x).is_err(), "=> [REASON] \"{x}\" IS a valid ORCiD");
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
    let valid = VALID_ROR_VALUES;
    let invalid = ["978-12345-99990", "10.1000/182", "10.97812345/99990", "3t40cz74", "03ebg0v16-"];
    for x in valid.iter() {
        assert!(is_ror(x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid ROR");
    }
    for x in invalid.iter() {
        assert!(is_ror(x).is_err(), "=> [REASON] \"{x}\" IS a valid ROR");
    }
}
#[test]
fn test_validate_is_unix_epoch() {
    let valid = [1759017645];
    let invalid = [42, 123456789];
    for x in valid.iter() {
        assert!(is_unix_epoch(*x).is_ok(), "=> [REASON] \"{x}\" is NOT a valid Unix epoch timestamp");
    }
    for x in invalid.iter() {
        assert!(is_unix_epoch(*x).is_err(), "=> [REASON] \"{x}\" IS a valid Unix epoch timestamp");
    }
}
