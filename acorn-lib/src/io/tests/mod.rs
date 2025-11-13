use crate::io::citeas;

#[test]
fn test_citeas() {
    let status = citeas::status();
    assert!(status.is_some());
    if let Some(citeas::Status { documentation_url, .. }) = status {
        assert_eq!(documentation_url, "https://citeas.org/api");
    }
    if let Some(citeas::Citation { text, .. }) = citeas::Citations::from_doi("10.11578/dc.20250604.1").match_style("apa") {
        println!("CiteAs Test Response Received");
        let expected = "Wohlgemuth, J. (2025). Accessible Content Optimization for Research Needs (ACORN). Oak Ridge National Laboratory (ORNL), Oak Ridge, TN (United States). http://doi.org/10.11578/DC.20250604.1";
        assert_eq!(text, expected);
    };
}
