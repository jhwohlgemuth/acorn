use super::Constant;
use acorn_core::prelude::alloc::{String, Vec};
use acorn_core::util::assets::EmbeddedAssets;

#[test]
fn test_schema_asset_inventory_is_embedded_and_nonempty() {
    for name in [
        "geonames.tsv",
        "keywords.csv",
        "organization.json",
        "partners.csv",
        "sponsors.csv",
        "technology.csv",
    ] {
        assert!(
            Constant::from_asset(name).is_some_and(|content| !content.trim().is_empty()),
            "missing schema asset {name}"
        );
    }
    assert!(Constant::from_asset("unknown.csv").is_none());
}

#[test]
fn test_schema_assets_have_singular_filesystem_ownership() {
    let schema_assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/constants");
    let lib_assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../acorn-lib/assets/constants");
    let lib_prompts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../acorn-lib/assets/prompts");
    assert!(schema_assets.is_dir());
    for name in [
        "geonames.tsv",
        "keywords.csv",
        "organization.json",
        "partners.csv",
        "sponsors.csv",
        "technology.csv",
    ] {
        assert!(!lib_assets.join(name).exists(), "schema asset is duplicated by acorn-lib: {name}");
    }
    for name in [
        "eli5.prompt",
        "extract-claim.prompt",
        "find-gaps.prompt",
        "summarize.prompt",
        "teach.prompt",
        "translate.prompt",
    ] {
        assert!(!lib_prompts.join(name).exists(), "schema prompt is duplicated by acorn-lib: {name}");
    }
}

#[test]
fn test_schema_vocabulary_assets_supply_csv_columns() {
    let technologies = Constant::nth_values("technology", 0).collect::<Vec<String>>();
    assert!(technologies.contains(&"airflow".to_string()));
    let technology_names = Constant::nth_values("technology", 2).collect::<Vec<String>>();
    assert!(technology_names.contains(&"Apache Airflow".to_string()));
    let partner_names = Constant::nth_values("partners", 0).collect::<Vec<String>>();
    assert!(partner_names.contains(&"Ames Laboratory".to_string()));
    let partner_identifiers = Constant::last_values("partners").collect::<Vec<String>>();
    assert!(partner_identifiers.contains(&"041m9xr71".to_string()));
}
