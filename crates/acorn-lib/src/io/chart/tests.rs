use super::ChartOptions;
use acorn_schema::research_activity::aspect::{AspectFramework, Autonomy, Motivity, SoftwarePortability};
use acorn_schema::TechnologyReadinessLevel;
fn render_svg(aspect: Option<&AspectFramework>, options: ChartOptions) -> Option<String> {
    options
        .render(aspect)
        .expect("SVG chart rendering should succeed")
        .map(|svg| String::from_utf8(svg).expect("SVG chart should be UTF-8"))
}
fn show_text() -> ChartOptions {
    ChartOptions::init().show_labels(true).show_scores(true).build()
}
#[test]
fn test_rose_chart_omits_empty_aspect() {
    assert!(render_svg(None, ChartOptions::default()).is_none());
    assert!(render_svg(Some(&AspectFramework::default()), ChartOptions::default()).is_none());
    assert!(ChartOptions::init()
        .file_type("png")
        .build()
        .render(None)
        .expect("empty chart rendering should succeed")
        .is_none());
}
#[test]
fn test_rose_chart_renders_png() {
    let aspect = AspectFramework::init().motivity(Motivity::Adaptive).build();
    let png = ChartOptions::init()
        .file_type("png")
        .build()
        .render(Some(&aspect))
        .expect("chart rendering should succeed")
        .expect("one ASPECT value should produce PNG bytes");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
}
#[test]
fn test_rose_svg_contains_four_normalized_aspect_attributes() {
    let aspect = AspectFramework::init()
        .autonomy(Autonomy::MachinePrimary)
        .motivity(Motivity::Adaptive)
        .portability(SoftwarePortability::Containerized)
        .maturity(TechnologyReadinessLevel::Operational)
        .build();
    let svg = render_svg(Some(&aspect), show_text()).expect("complete ASPECT values should produce a chart");
    ["Autonomy", "Motivity", "Portability", "Maturity", "3/5", "5/5", "2/5", "7/9"]
        .iter()
        .for_each(|value| assert!(svg.contains(value)));
    ["#00BDB5", "#006BA6", "#00662C", "#7DBA00"]
        .iter()
        .for_each(|color| assert!(svg.contains(color)));
    assert!(svg.contains(r#"viewBox="0 0 300 300""#));
    assert_eq!(svg.matches("fill=\"#d9dde2\"").count(), 1);
    assert_eq!(svg.matches("class=\"score\"").count(), 4);
    assert!(svg.contains(r##"<circle cx="150" cy="150" r="120" fill="#d9dde2""##));
    assert!(svg.contains(r#"class="separators" d="M 150 50 V 250 M 50 150 H 250""#));
    assert!(!svg.contains("A 120.000 120.000"));
    assert!(svg.contains("A 100.000 100.000"));
    assert!(!svg.contains("A 24.000 24.000"));
}
#[test]
fn test_rose_svg_distinguishes_zero_from_missing_without_text() {
    let aspect = AspectFramework::init().autonomy(Autonomy::Manual).build();
    let svg = render_svg(Some(&aspect), ChartOptions::default()).expect("zero ASPECT value should produce a chart");
    assert!(svg.contains(r#"viewBox="30 30 240 240""#));
    assert_eq!(svg.matches("class=\"zero-marker\"").count(), 1);
    assert!(!svg.contains("class=\"label\""));
    assert!(!svg.contains("class=\"score\""));
}
#[test]
fn test_rose_svg_hides_labels_and_scores_independently() {
    let aspect = AspectFramework::init().motivity(Motivity::Reactive).build();
    let default = render_svg(Some(&aspect), ChartOptions::default()).expect("one ASPECT value should produce a chart");
    assert!(!default.contains("class=\"label\""));
    assert!(!default.contains("class=\"score\""));
    let no_labels = render_svg(Some(&aspect), ChartOptions::init().show_scores(true).build()).expect("one ASPECT value should produce a chart");
    assert!(!no_labels.contains("class=\"label\""));
    assert!(no_labels.contains("class=\"score\""));
    let no_scores = render_svg(Some(&aspect), ChartOptions::init().show_labels(true).build()).expect("one ASPECT value should produce a chart");
    assert!(no_scores.contains("class=\"label\""));
    assert!(!no_scores.contains("class=\"score\""));
    assert!(!no_scores.contains(">N/A</text>"));
}
#[test]
fn test_rose_svg_preserves_missing_attributes() {
    let aspect = AspectFramework::init().motivity(Motivity::Reactive).build();
    let svg = render_svg(Some(&aspect), show_text()).expect("one ASPECT value should produce a chart");
    assert_eq!(svg.matches(">N/A</text>").count(), 3);
    assert!(svg.contains(">4/5</text>"));
}
