//! Chart generation for export artifacts
use crate::io::ApiResult;
#[cfg(feature = "std")]
use crate::io::InputOutput;
use crate::prelude::{vec, Vec};
#[cfg(feature = "std")]
use crate::prelude::{HashMap, PathBuf};
use crate::schema::research_activity::aspect::AspectFramework;
#[cfg(feature = "std")]
use crate::schema::research_activity::ResearchActivity;
use crate::util::constants::app::{ORNL_COLOR_AQUA, ORNL_COLOR_ENERGY, ORNL_COLOR_GREEN, ORNL_COLOR_INFINITY};
#[cfg(feature = "std")]
use crate::util::StringConversion;
use bon::Builder;
use color_eyre::eyre::eyre;
use core::f64::consts::{FRAC_PI_2, FRAC_PI_4};
use resvg::{tiny_skia, usvg};

const CENTER: f64 = 150.0;
const FRAME_RADIUS: f64 = 120.0;
const PNG_SCALE: u32 = 2;
const ROSE_RADIUS: f64 = 100.0;
const SIZE: u32 = 300;
/// File type rendered for an ASPECT chart.
#[derive(Clone, Copy, Default)]
pub enum ChartFileType {
    /// Portable Network Graphics.
    Png,
    /// Scalable Vector Graphics.
    #[default]
    Svg,
}
#[derive(Builder, Clone, Copy)]
#[builder(start_fn = init)]
struct Attribute {
    color: [u8; 3],
    level: Option<u8>,
    maximum: u8,
    name: &'static str,
    order: u8,
}
/// Controls ASPECT chart rendering.
#[derive(Builder, Clone, Copy, Default)]
#[builder(start_fn = init)]
pub struct ChartOptions {
    /// Rendered chart file type.
    #[builder(default, into)]
    pub file_type: ChartFileType,
    /// Show ASPECT attribute labels.
    #[builder(default)]
    pub show_labels: bool,
    /// Show ASPECT numeric scores.
    #[builder(default)]
    pub show_scores: bool,
}
impl Attribute {
    fn render(self, options: ChartOptions) -> String {
        let start = f64::from(self.order) * FRAC_PI_2;
        let end = start + FRAC_PI_2;
        let midpoint = start + FRAC_PI_4;
        let color = color_hex(self.color);
        let foreground = match self.level {
            | Some(0) => {
                let (x, y) = point(midpoint, 9.0);
                format!(r#"<circle class="zero-marker" cx="{x:.3}" cy="{y:.3}" r="4" fill="{color}"/>"#)
            }
            | Some(level) => {
                let normalized = f64::from(level) / f64::from(self.maximum);
                let radius = ROSE_RADIUS * normalized.sqrt();
                let path = sector_path(start, end, radius);
                format!(r##"<path d="{path}" fill="{color}" stroke="#fff" stroke-width="2"/>"##)
            }
            | None => String::new(),
        };
        let (label_x, label_y) = point(midpoint, 138.0);
        let (score_x, score_y) = point(midpoint, 75.0);
        let label = if options.show_labels {
            format!(r#"<text x="{label_x:.3}" y="{label_y:.3}" class="label">{}</text>"#, self.name)
        } else {
            String::new()
        };
        let score = if options.show_scores {
            format!(r#"<text x="{score_x:.3}" y="{score_y:.3}" class="score">{}</text>"#, self.score())
        } else {
            String::new()
        };
        format!("{foreground}{label}{score}")
    }
    fn score(self) -> String {
        self.level
            .map(|level| format!("{level}/{}", self.maximum))
            .unwrap_or_else(|| "N/A".to_string())
    }
}
impl From<&AspectFramework> for Vec<Attribute> {
    fn from(aspect: &AspectFramework) -> Self {
        vec![
            Attribute::init()
                .name("Autonomy")
                .maybe_level(aspect.autonomy.clone().map(|value| value as u8))
                .maximum(5)
                .color(ORNL_COLOR_AQUA)
                .order(0)
                .build(),
            Attribute::init()
                .name("Motivity")
                .maybe_level(aspect.motivity.clone().map(|value| value as u8))
                .maximum(5)
                .color(ORNL_COLOR_INFINITY)
                .order(1)
                .build(),
            Attribute::init()
                .name("Portability")
                .maybe_level(aspect.portability.clone().map(|value| value as u8))
                .maximum(5)
                .color(ORNL_COLOR_GREEN)
                .order(2)
                .build(),
            Attribute::init()
                .name("Maturity")
                .maybe_level(aspect.maturity.clone().map(|value| value as u8))
                .maximum(9)
                .color(ORNL_COLOR_ENERGY)
                .order(3)
                .build(),
        ]
    }
}
impl From<&str> for ChartFileType {
    fn from(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            | "png" => Self::Png,
            | _ => Self::Svg,
        }
    }
}
impl ChartOptions {
    /// Render an ASPECT rose chart when at least one scalar attribute is present.
    pub fn render(self, aspect: Option<&AspectFramework>) -> ApiResult<Option<Vec<u8>>> {
        match (self.file_type, self.render_svg(aspect)) {
            | (_, None) => Ok(None),
            | (ChartFileType::Svg, Some(svg)) => Ok(Some(svg.into_bytes())),
            | (ChartFileType::Png, Some(svg)) => {
                let mut options = usvg::Options::default();
                options.fontdb_mut().load_system_fonts();
                usvg::Tree::from_str(&svg, &options)
                    .map_err(|why| eyre!("Failed to parse ASPECT rose chart SVG — {why}"))
                    .and_then(|tree| {
                        tiny_skia::Pixmap::new(SIZE * PNG_SCALE, SIZE * PNG_SCALE)
                            .ok_or_else(|| eyre!("Failed to allocate ASPECT rose chart image"))
                            .and_then(|mut pixmap| {
                                let scale = PNG_SCALE as f32;
                                resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
                                pixmap
                                    .encode_png()
                                    .map(Some)
                                    .map_err(|why| eyre!("Failed to encode ASPECT rose chart PNG — {why}"))
                            })
                    })
            }
        }
    }
    /// Render ASPECT rose charts for research activity files.
    #[cfg(feature = "std")]
    pub fn render_aspect_charts(self, paths: &[PathBuf]) -> ApiResult<HashMap<PathBuf, Option<Vec<u8>>>> {
        paths
            .iter()
            .map(|path| match ResearchActivity::read(path.clone()) {
                | Ok(data) => self.render(data.aspect.as_ref()).map(|chart| (path.clone(), chart)),
                | Err(why) => Err(eyre!("Read data for ASPECT chart at {} — {why}", path.to_absolute_path())),
            })
            .collect()
    }
    fn render_svg(self, aspect: Option<&AspectFramework>) -> Option<String> {
        aspect.and_then(|aspect| {
            let attributes = Vec::<Attribute>::from(aspect);
            attributes.iter().any(|attribute| attribute.level.is_some()).then(|| {
                let view_box = if self.show_labels { "0 0 300 300" } else { "30 30 240 240" };
                let sectors = attributes
                    .into_iter()
                    .map(|attribute| attribute.render(self))
                    .collect::<String>();
                format!(
                    r##"<svg xmlns="http://www.w3.org/2000/svg" width="{SIZE}" height="{SIZE}" viewBox="{view_box}" role="img" aria-labelledby="title description">
<title id="title">ASPECT rose chart</title><desc id="description">Normalized autonomy, motivity, portability, and maturity scores.</desc>
<style>text{{font-family:Arial,"DejaVu Sans",sans-serif;text-anchor:middle;dominant-baseline:middle;fill:#20242a}}.label{{font-size:12px;font-weight:600}}.score{{font-size:11px;paint-order:stroke;stroke:#fff;stroke-width:3px;stroke-linejoin:round}}</style>
<rect width="{SIZE}" height="{SIZE}" fill="#fff"/><circle cx="{CENTER}" cy="{CENTER}" r="{FRAME_RADIUS}" fill="#d9dde2" fill-opacity=".7"/>
{sectors}<path class="separators" d="M 150 50 V 250 M 50 150 H 250" fill="none" stroke="#fff" stroke-width="2"/></svg>"##
                )
            })
        })
    }
}
fn color_hex([red, green, blue]: [u8; 3]) -> String {
    format!("#{red:02X}{green:02X}{blue:02X}")
}
fn point(angle: f64, radius: f64) -> (f64, f64) {
    (CENTER + radius * angle.sin(), CENTER - radius * angle.cos())
}
fn sector_path(start: f64, end: f64, outer: f64) -> String {
    let (outer_start_x, outer_start_y) = point(start, outer);
    let (outer_end_x, outer_end_y) = point(end, outer);
    format!(
        "M {CENTER:.3} {CENTER:.3} L {outer_start_x:.3} {outer_start_y:.3} \
         A {outer:.3} {outer:.3} 0 0 1 {outer_end_x:.3} {outer_end_y:.3} Z"
    )
}
#[cfg(test)]
mod tests {
    use super::ChartOptions;
    use crate::schema::research_activity::aspect::{AspectFramework, Autonomy, Motivity, SoftwarePortability};
    use crate::schema::TechnologyReadinessLevel;
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
}
