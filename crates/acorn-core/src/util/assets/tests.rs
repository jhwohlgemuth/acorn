use super::*;

#[test]
fn parses_text_assets_without_an_asset_owner() {
    let rows = csv(Some("first,one\nsecond,two".to_string()));
    assert_eq!(last_values(rows.clone()).collect::<Vec<_>>(), vec!["one", "two"]);
    assert_eq!(nth_values(rows, 0).collect::<Vec<_>>(), vec!["first", "second"]);
}
