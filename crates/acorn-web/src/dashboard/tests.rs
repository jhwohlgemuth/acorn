use super::*;

#[test]
fn test_donut_gradient_handles_counts_and_empty_data() {
    let segments = vec![
        ChartSegment {
            label: "first".into(),
            count: 1,
            color: "red".into(),
        },
        ChartSegment {
            label: "second".into(),
            count: 3,
            color: "blue".into(),
        },
    ];
    assert_eq!(donut_gradient(&segments), "red 0.00% 25.00%, blue 25.00% 100.00%");
    assert!(donut_gradient(&[]).is_empty());
}
