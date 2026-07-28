//! ASPECT chart generation for document exports.
use acorn::io::ApiResult;
use acorn::schema::research_activity::aspect::AspectFramework;
use color_eyre::eyre::eyre;
use core::f64::consts::{FRAC_PI_2, FRAC_PI_4};
use resvg::{tiny_skia, usvg};

const SIZE: u32 = 300;
const CENTER: f64 = 150.0;
const ROSE_RADIUS: f64 = 100.0;
const FRAME_RADIUS: f64 = 120.0;
const PNG_SCALE: u32 = 2;

#[derive(Clone, Copy, Default)]
pub(crate) struct ChartOptions {
    pub(crate) show_labels: bool,
    pub(crate) show_scores: bool,
}

#[derive(Clone, Copy)]
struct Attribute {
    name: &'static str,
    level: Option<u8>,
    maximum: u8,
    color: &'static str,
    order: u8,
}

impl Attribute {
    fn normalized(self) -> Option<f64> {
        self.level.map(|level| f64::from(level) / f64::from(self.maximum))
    }

    fn score(self) -> String {
        self.level
            .map(|level| format!("{level}/{}", self.maximum))
            .unwrap_or_else(|| "N/A".to_string())
    }
}

fn attributes(aspect: &AspectFramework) -> [Attribute; 4] {
    [
        Attribute {
            name: "Autonomy",
            level: aspect.autonomy.clone().map(|value| value as u8),
            maximum: 5,
            color: "#16537b",
            order: 0,
        },
        Attribute {
            name: "Motivity",
            level: aspect.motivity.clone().map(|value| value as u8),
            maximum: 5,
            color: "#007833",
            order: 1,
        },
        Attribute {
            name: "Portability",
            level: aspect.portability.clone().map(|value| value as u8),
            maximum: 5,
            color: "#f0a202",
            order: 2,
        },
        Attribute {
            name: "Maturity",
            level: aspect.maturity.clone().map(|value| value as u8),
            maximum: 9,
            color: "#8a4f9e",
            order: 3,
        },
    ]
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

fn attribute_svg(attribute: Attribute, options: ChartOptions) -> String {
    let start = f64::from(attribute.order) * FRAC_PI_2;
    let end = start + FRAC_PI_2;
    let midpoint = start + FRAC_PI_4;
    let foreground = attribute
        .normalized()
        .map(|normalized| ROSE_RADIUS * normalized.sqrt())
        .map(|radius| {
            let path = sector_path(start, end, radius);
            format!(r##"<path d="{path}" fill="{}" stroke="#fff" stroke-width="2"/>"##, attribute.color)
        })
        .unwrap_or_default();
    let (label_x, label_y) = point(midpoint, 138.0);
    let (score_x, score_y) = point(midpoint, 75.0);
    let label = if options.show_labels {
        format!(r#"<text x="{label_x:.3}" y="{label_y:.3}" class="label">{}</text>"#, attribute.name)
    } else {
        String::new()
    };
    let score = if options.show_scores {
        let score = attribute.score();
        format!(r#"<text x="{score_x:.3}" y="{score_y:.3}" class="score">{score}</text>"#)
    } else {
        String::new()
    };
    format!("{foreground}{label}{score}")
}

pub(crate) fn aspect_rose_svg(aspect: Option<&AspectFramework>, options: ChartOptions) -> Option<String> {
    aspect.and_then(|aspect| {
        let attributes = attributes(aspect);
        attributes.iter().any(|attribute| attribute.level.is_some()).then(|| {
            let sectors = attributes
                .into_iter()
                .map(|attribute| attribute_svg(attribute, options))
                .collect::<String>();
            format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" width="{SIZE}" height="{SIZE}" viewBox="0 0 {SIZE} {SIZE}" role="img" aria-labelledby="title description">
<title id="title">ASPECT rose chart</title><desc id="description">Normalized autonomy, motivity, portability, and maturity scores.</desc>
<style>text{{font-family:Arial,"DejaVu Sans",sans-serif;text-anchor:middle;dominant-baseline:middle;fill:#20242a}}.label{{font-size:12px;font-weight:600}}.score{{font-size:11px;paint-order:stroke;stroke:#fff;stroke-width:3px;stroke-linejoin:round}}</style>
<rect width="{SIZE}" height="{SIZE}" fill="#fff"/><circle cx="{CENTER}" cy="{CENTER}" r="{FRAME_RADIUS}" fill="#d9dde2" fill-opacity=".7"/>
{sectors}<path class="separators" d="M 150 50 V 250 M 50 150 H 250" fill="none" stroke="#fff" stroke-width="2"/></svg>"##
            )
        })
    })
}

pub(crate) fn aspect_rose_png(aspect: Option<&AspectFramework>, options: ChartOptions) -> ApiResult<Option<Vec<u8>>> {
    match aspect_rose_svg(aspect, options) {
        | Some(svg) => {
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
        | None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{aspect_rose_png, aspect_rose_svg, ChartOptions};
    use acorn::schema::research_activity::aspect::{AspectFramework, Autonomy, Motivity, SoftwarePortability};
    use acorn::schema::TechnologyReadinessLevel;

    const SHOW_TEXT: ChartOptions = ChartOptions {
        show_labels: true,
        show_scores: true,
    };

    #[test]
    fn rose_svg_contains_four_normalized_aspect_attributes() {
        let aspect = AspectFramework::init()
            .autonomy(Autonomy::MachinePrimary)
            .motivity(Motivity::Adaptive)
            .portability(SoftwarePortability::Containerized)
            .maturity(TechnologyReadinessLevel::Operational)
            .build();
        let svg = aspect_rose_svg(Some(&aspect), SHOW_TEXT).expect("complete ASPECT values should produce a chart");
        ["Autonomy", "Motivity", "Portability", "Maturity", "3/5", "5/5", "2/5", "7/9"]
            .iter()
            .for_each(|value| assert!(svg.contains(value)));
        assert_eq!(svg.matches("fill=\"#d9dde2\"").count(), 1);
        assert_eq!(svg.matches("class=\"score\"").count(), 4);
        assert!(svg.contains(r##"<circle cx="150" cy="150" r="120" fill="#d9dde2""##));
        assert!(svg.contains(r#"class="separators" d="M 150 50 V 250 M 50 150 H 250""#));
        assert!(!svg.contains("A 120.000 120.000"));
        assert!(svg.contains("A 100.000 100.000"));
        assert!(!svg.contains("A 24.000 24.000"));
    }

    #[test]
    fn rose_svg_preserves_missing_attributes() {
        let aspect = AspectFramework::init().motivity(Motivity::Reactive).build();
        let svg = aspect_rose_svg(Some(&aspect), SHOW_TEXT).expect("one ASPECT value should produce a chart");
        assert_eq!(svg.matches(">N/A</text>").count(), 3);
        assert!(svg.contains(">4/5</text>"));
    }

    #[test]
    fn rose_svg_hides_labels_and_scores_independently() {
        let aspect = AspectFramework::init().motivity(Motivity::Reactive).build();
        let default = aspect_rose_svg(Some(&aspect), ChartOptions::default()).expect("one ASPECT value should produce a chart");
        assert!(!default.contains("class=\"label\""));
        assert!(!default.contains("class=\"score\""));
        let no_labels = aspect_rose_svg(
            Some(&aspect),
            ChartOptions {
                show_labels: false,
                show_scores: true,
            },
        )
        .expect("one ASPECT value should produce a chart");
        assert!(!no_labels.contains("class=\"label\""));
        assert!(no_labels.contains("class=\"score\""));
        let no_scores = aspect_rose_svg(
            Some(&aspect),
            ChartOptions {
                show_labels: true,
                show_scores: false,
            },
        )
        .expect("one ASPECT value should produce a chart");
        assert!(no_scores.contains("class=\"label\""));
        assert!(!no_scores.contains("class=\"score\""));
        assert!(!no_scores.contains(">N/A</text>"));
    }

    #[test]
    fn rose_chart_omits_empty_aspect() {
        assert!(aspect_rose_svg(None, ChartOptions::default()).is_none());
        assert!(aspect_rose_svg(Some(&AspectFramework::default()), ChartOptions::default()).is_none());
        assert!(aspect_rose_png(None, ChartOptions::default())
            .expect("empty chart rendering should succeed")
            .is_none());
    }

    #[test]
    fn rose_chart_renders_png() {
        let aspect = AspectFramework::init().motivity(Motivity::Adaptive).build();
        let png = aspect_rose_png(Some(&aspect), ChartOptions::default())
            .expect("chart rendering should succeed")
            .expect("one ASPECT value should produce PNG bytes");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    }
}
