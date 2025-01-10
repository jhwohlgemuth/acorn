use crate::schema::*;
use glob::glob;

#[test]
fn test_read() {
    for entry in glob("./**/*").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => println!("{:?}", path.display()),
            Err(e) => println!("{:?}", e),
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
    assert_eq!(
        resolve_sponsor("ORNL".into()),
        Some("Oak Ridge National Laboratory".into())
    );
    assert_eq!(
        resolve_sponsor("Dept of Energy".into()),
        Some("Department of Energy".into())
    );
    assert_eq!(
        resolve_sponsor("Dept. of Energy".into()),
        Some("Department of Energy".into())
    );
    assert_eq!(
        resolve_sponsor("DHS ".into()),
        Some("Department of Homeland Security".into())
    );
    assert_eq!(
        resolve_sponsor(" epa".into()),
        Some("Environmental Protection Agency".into())
    );
    assert_eq!(
        resolve_sponsor("darpa".into()),
        Some("Defense Advanced Research Projects Agency".into())
    );
}
#[test]
fn test_fuzzy_technologies() {
    assert_eq!(resolve_technology("astro".into()), Some("astro".into()));
    assert_eq!(resolve_technology("React.js".into()), Some("react".into()));
    assert_eq!(resolve_technology("rs".into()), Some("rust".into()));
    assert_eq!(
        resolve_technology("Geospatial Data Abstraction Lib".into()),
        Some("gdal".into())
    );
    assert_eq!(resolve_technology("foobarbaz".into()), None);
}
