//! Typed CZML materials and their exclusive material-kind rules.
use super::Property;
use crate::validation::{report_of, Validate, ValidationReport};
use acorn_core::validation::ValidationError;

/// A material that fills the surface with a checkerboard pattern.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CheckerboardMaterial {
    /// evenColor material component.
    pub even_color: Option<Property<super::Color>>,
    /// oddColor material component.
    pub odd_color: Option<Property<super::Color>>,
    /// repeat material component.
    pub repeat: Option<Property<super::Repeat>>,
}
/// A material that fills the surface with a two-dimensional grid.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GridMaterial {
    /// cellAlpha material component.
    pub cell_alpha: Option<Property<super::Double>>,
    /// color material component.
    pub color: Option<Property<super::Color>>,
    /// lineCount material component.
    pub line_count: Option<Property<super::LineCount>>,
    /// lineOffset material component.
    pub line_offset: Option<Property<super::LineOffset>>,
    /// lineThickness material component.
    pub line_thickness: Option<Property<super::LineThickness>>,
}
/// A material that fills the surface with an image.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ImageMaterial {
    /// color material component.
    pub color: Option<Property<super::Color>>,
    /// image material component.
    pub image: Option<Property<super::Uri>>,
    /// repeat material component.
    pub repeat: Option<Property<super::Repeat>>,
    /// transparent material component.
    pub transparent: Option<Property<super::Boolean>>,
}
/// A definition of how a surface is colored or shaded.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Material {
    /// checkerboard material component.
    pub checkerboard: Option<CheckerboardMaterial>,
    /// grid material component.
    pub grid: Option<GridMaterial>,
    /// image material component.
    pub image: Option<ImageMaterial>,
    /// solidColor material component.
    pub solid_color: Option<SolidColorMaterial>,
    /// stripe material component.
    pub stripe: Option<StripeMaterial>,
}
/// A material that fills the surface of a line with an arrow.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PolylineArrowMaterial {
    /// color material component.
    pub color: Option<Property<super::Color>>,
}
/// A material that fills the surface of a line with a pattern of dashes.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PolylineDashMaterial {
    /// color material component.
    pub color: Option<Property<super::Color>>,
    /// dashLength material component.
    pub dash_length: Option<Property<super::Double>>,
    /// dashPattern material component.
    pub dash_pattern: Option<Property<super::Integer>>,
    /// gapColor material component.
    pub gap_color: Option<Property<super::Color>>,
}
/// A material that fills the surface of a line with a glowing color.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PolylineGlowMaterial {
    /// color material component.
    pub color: Option<Property<super::Color>>,
    /// glowPower material component.
    pub glow_power: Option<Property<super::Double>>,
    /// taperPower material component.
    pub taper_power: Option<Property<super::Double>>,
}
/// A definition of how a polyline is colored or shaded.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PolylineMaterial {
    /// checkerboard material component.
    pub checkerboard: Option<CheckerboardMaterial>,
    /// grid material component.
    pub grid: Option<GridMaterial>,
    /// image material component.
    pub image: Option<ImageMaterial>,
    /// polylineArrow material component.
    pub polyline_arrow: Option<PolylineArrowMaterial>,
    /// polylineDash material component.
    pub polyline_dash: Option<PolylineDashMaterial>,
    /// polylineGlow material component.
    pub polyline_glow: Option<PolylineGlowMaterial>,
    /// polylineOutline material component.
    pub polyline_outline: Option<PolylineOutlineMaterial>,
    /// solidColor material component.
    pub solid_color: Option<SolidColorMaterial>,
    /// stripe material component.
    pub stripe: Option<StripeMaterial>,
}
/// A material that fills the surface of a line with an outlined color.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PolylineOutlineMaterial {
    /// color material component.
    pub color: Option<Property<super::Color>>,
    /// outlineColor material component.
    pub outline_color: Option<Property<super::Color>>,
    /// outlineWidth material component.
    pub outline_width: Option<Property<super::Double>>,
}
/// A material that fills the surface with a solid color.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SolidColorMaterial {
    /// color material component.
    pub color: Option<Property<super::Color>>,
}
/// A material that fills the surface with alternating colors.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StripeMaterial {
    /// evenColor material component.
    pub even_color: Option<Property<super::Color>>,
    /// oddColor material component.
    pub odd_color: Option<Property<super::Color>>,
    /// offset material component.
    pub offset: Option<Property<super::Double>>,
    /// orientation material component.
    pub orientation: Option<Property<super::StripeOrientation>>,
    /// repeat material component.
    pub repeat: Option<Property<super::Double>>,
}
impl Validate for CheckerboardMaterial {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("evenColor", validate_optional(&self.even_color));
        let report = report.merge("oddColor", validate_optional(&self.odd_color));
        let report = report.merge("repeat", validate_optional(&self.repeat));
        report.finish()
    }
}
impl Validate for GridMaterial {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("cellAlpha", validate_optional(&self.cell_alpha));
        let report = report.merge("color", validate_optional(&self.color));
        let report = report.merge("lineCount", validate_optional(&self.line_count));
        let report = report.merge("lineOffset", validate_optional(&self.line_offset));
        let report = report.merge("lineThickness", validate_optional(&self.line_thickness));
        report.finish()
    }
}
impl Validate for ImageMaterial {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("color", validate_optional(&self.color));
        let report = report.merge("image", validate_optional(&self.image));
        let report = report.merge("repeat", validate_optional(&self.repeat));
        let report = report.merge("transparent", validate_optional(&self.transparent));
        report.finish()
    }
}
impl Validate for Material {
    fn validate(&self) -> Result<(), ValidationReport> {
        let count = [
            self.checkerboard.is_some(),
            self.grid.is_some(),
            self.image.is_some(),
            self.solid_color.is_some(),
            self.stripe.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        let report = if count > 1 {
            ValidationReport::from_error(
                "",
                ValidationError::new("czml_material_kinds").with_message("At most one material kind is allowed"),
            )
        } else {
            ValidationReport::new()
        };
        let report = report.merge("checkerboard", validate_optional(&self.checkerboard));
        let report = report.merge("grid", validate_optional(&self.grid));
        let report = report.merge("image", validate_optional(&self.image));
        let report = report.merge("solidColor", validate_optional(&self.solid_color));
        let report = report.merge("stripe", validate_optional(&self.stripe));
        report.finish()
    }
}
impl Validate for PolylineArrowMaterial {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("color", validate_optional(&self.color));
        report.finish()
    }
}
impl Validate for PolylineDashMaterial {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("color", validate_optional(&self.color));
        let report = report.merge("dashLength", validate_optional(&self.dash_length));
        let report = report.merge("dashPattern", validate_optional(&self.dash_pattern));
        let report = report.merge("gapColor", validate_optional(&self.gap_color));
        report.finish()
    }
}
impl Validate for PolylineGlowMaterial {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("color", validate_optional(&self.color));
        let report = report.merge("glowPower", validate_optional(&self.glow_power));
        let report = report.merge("taperPower", validate_optional(&self.taper_power));
        report.finish()
    }
}
impl Validate for PolylineMaterial {
    fn validate(&self) -> Result<(), ValidationReport> {
        let count = [
            self.checkerboard.is_some(),
            self.grid.is_some(),
            self.image.is_some(),
            self.polyline_arrow.is_some(),
            self.polyline_dash.is_some(),
            self.polyline_glow.is_some(),
            self.polyline_outline.is_some(),
            self.solid_color.is_some(),
            self.stripe.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        let report = if count > 1 {
            ValidationReport::from_error(
                "",
                ValidationError::new("czml_material_kinds").with_message("At most one material kind is allowed"),
            )
        } else {
            ValidationReport::new()
        };
        let report = report.merge("checkerboard", validate_optional(&self.checkerboard));
        let report = report.merge("grid", validate_optional(&self.grid));
        let report = report.merge("image", validate_optional(&self.image));
        let report = report.merge("polylineArrow", validate_optional(&self.polyline_arrow));
        let report = report.merge("polylineDash", validate_optional(&self.polyline_dash));
        let report = report.merge("polylineGlow", validate_optional(&self.polyline_glow));
        let report = report.merge("polylineOutline", validate_optional(&self.polyline_outline));
        let report = report.merge("solidColor", validate_optional(&self.solid_color));
        let report = report.merge("stripe", validate_optional(&self.stripe));
        report.finish()
    }
}
impl Validate for PolylineOutlineMaterial {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("color", validate_optional(&self.color));
        let report = report.merge("outlineColor", validate_optional(&self.outline_color));
        let report = report.merge("outlineWidth", validate_optional(&self.outline_width));
        report.finish()
    }
}
impl Validate for SolidColorMaterial {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("color", validate_optional(&self.color));
        report.finish()
    }
}
impl Validate for StripeMaterial {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("evenColor", validate_optional(&self.even_color));
        let report = report.merge("oddColor", validate_optional(&self.odd_color));
        let report = report.merge("offset", validate_optional(&self.offset));
        let report = report.merge("orientation", validate_optional(&self.orientation));
        let report = report.merge("repeat", validate_optional(&self.repeat));
        report.finish()
    }
}
pub(crate) fn validate_optional<T: Validate>(value: &Option<T>) -> ValidationReport {
    report_of(value.as_ref().map(Validate::validate).transpose().map(|_| ()))
}
