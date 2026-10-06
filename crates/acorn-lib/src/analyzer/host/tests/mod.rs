use super::tabular::{checks_to_csv, csv_field};

#[test]
fn test_csv_fields_escape_delimiters_and_quotes() {
    assert_eq!(csv_field(Some("plain".to_string())), "plain");
    assert_eq!(
        csv_field(Some("comma, quote \" and\nnewline".to_string())),
        "\"comma, quote \"\" and\nnewline\""
    );
    assert_eq!(csv_field(Some(String::new())), "\"\"");
    assert_eq!(csv_field(None), "");
}
#[test]
fn test_empty_csv_can_include_headers() {
    assert_eq!(
        checks_to_csv(&[], true).expect("CSV headers should render"),
        "index,severity,category,uri,locator,message,context,fair\n"
    );
    assert_eq!(checks_to_csv(&[], false).expect("Empty CSV should render"), "");
}
