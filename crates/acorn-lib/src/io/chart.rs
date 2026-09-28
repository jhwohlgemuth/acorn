//! Chart generation for export artifacts
use crate::io::ApiResult;
#[cfg(feature = "std")]
use crate::io::InputOutput;
use crate::util::constants::app::{ORNL_COLOR_AQUA, ORNL_COLOR_ENERGY, ORNL_COLOR_GREEN, ORNL_COLOR_INFINITY};
#[cfg(feature = "std")]
use crate::util::StringConversion;
use acorn_core::prelude::HashMap;
use acorn_core::prelude::{vec, Vec};
use acorn_schema::research_activity::aspect::AspectFramework;
#[cfg(feature = "std")]
use acorn_schema::research_activity::ResearchActivity;
use bon::Builder;
use color_eyre::eyre::eyre;
use core::f64::consts::{FRAC_PI_2, FRAC_PI_4};
use resvg::{tiny_skia, usvg};
#[cfg(feature = "std")]
use std::path::PathBuf;

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
fn attributes_from(aspect: &AspectFramework) -> Vec<Attribute> {
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
            let attributes = attributes_from(aspect);
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
mod tests;
