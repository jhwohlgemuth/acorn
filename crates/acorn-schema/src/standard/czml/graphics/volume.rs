//! Typed CZML volume graphics.
use crate::validation::{Validate, ValidationReport};

/// A box, which is a closed rectangular cuboid.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Box {
    /// dimensions property of the box graphic.
    pub dimensions: Option<super::super::Property<super::super::BoxDimensions>>,
    /// distanceDisplayCondition property of the box graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// fill property of the box graphic.
    pub fill: Option<super::super::Property<super::super::Boolean>>,
    /// heightReference property of the box graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// material property of the box graphic.
    pub material: Option<super::super::Material>,
    /// outline property of the box graphic.
    pub outline: Option<super::super::Property<super::super::Boolean>>,
    /// outlineColor property of the box graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the box graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// shadows property of the box graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// show property of the box graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
}
/// A cylinder, truncated cone, or cone defined by a length, top radius, and bottom radius.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Cylinder {
    /// bottomRadius property of the cylinder graphic.
    pub bottom_radius: Option<super::super::Property<super::super::Double>>,
    /// distanceDisplayCondition property of the cylinder graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// fill property of the cylinder graphic.
    pub fill: Option<super::super::Property<super::super::Boolean>>,
    /// heightReference property of the cylinder graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// length property of the cylinder graphic.
    pub length: Option<super::super::Property<super::super::Double>>,
    /// material property of the cylinder graphic.
    pub material: Option<super::super::Material>,
    /// numberOfVerticalLines property of the cylinder graphic.
    pub number_of_vertical_lines: Option<super::super::Property<super::super::Integer>>,
    /// outline property of the cylinder graphic.
    pub outline: Option<super::super::Property<super::super::Boolean>>,
    /// outlineColor property of the cylinder graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the cylinder graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// shadows property of the cylinder graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// show property of the cylinder graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// slices property of the cylinder graphic.
    pub slices: Option<super::super::Property<super::super::Integer>>,
    /// topRadius property of the cylinder graphic.
    pub top_radius: Option<super::super::Property<super::super::Double>>,
}
/// A closed quadric surface that is a three-dimensional analogue of an ellipse.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Ellipsoid {
    /// distanceDisplayCondition property of the ellipsoid graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// fill property of the ellipsoid graphic.
    pub fill: Option<super::super::Property<super::super::Boolean>>,
    /// heightReference property of the ellipsoid graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// innerRadii property of the ellipsoid graphic.
    pub inner_radii: Option<super::super::Property<super::super::EllipsoidRadii>>,
    /// material property of the ellipsoid graphic.
    pub material: Option<super::super::Material>,
    /// maximumClock property of the ellipsoid graphic.
    pub maximum_clock: Option<super::super::Property<super::super::Double>>,
    /// maximumCone property of the ellipsoid graphic.
    pub maximum_cone: Option<super::super::Property<super::super::Double>>,
    /// minimumClock property of the ellipsoid graphic.
    pub minimum_clock: Option<super::super::Property<super::super::Double>>,
    /// minimumCone property of the ellipsoid graphic.
    pub minimum_cone: Option<super::super::Property<super::super::Double>>,
    /// outline property of the ellipsoid graphic.
    pub outline: Option<super::super::Property<super::super::Boolean>>,
    /// outlineColor property of the ellipsoid graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the ellipsoid graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// radii property of the ellipsoid graphic.
    pub radii: Option<super::super::Property<super::super::EllipsoidRadii>>,
    /// shadows property of the ellipsoid graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// show property of the ellipsoid graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// slicePartitions property of the ellipsoid graphic.
    pub slice_partitions: Option<super::super::Property<super::super::Integer>>,
    /// stackPartitions property of the ellipsoid graphic.
    pub stack_partitions: Option<super::super::Property<super::super::Integer>>,
    /// subdivisions property of the ellipsoid graphic.
    pub subdivisions: Option<super::super::Property<super::super::Integer>>,
}
/// A polyline with a volume, defined as a 2D shape extruded along a polyline that conforms to the curvature of the globe.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PolylineVolume {
    /// cornerType property of the polylinevolume graphic.
    pub corner_type: Option<super::super::Property<super::super::CornerType>>,
    /// distanceDisplayCondition property of the polylinevolume graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// fill property of the polylinevolume graphic.
    pub fill: Option<super::super::Property<super::super::Boolean>>,
    /// granularity property of the polylinevolume graphic.
    pub granularity: Option<super::super::Property<super::super::Double>>,
    /// material property of the polylinevolume graphic.
    pub material: Option<super::super::Material>,
    /// outline property of the polylinevolume graphic.
    pub outline: Option<super::super::Property<super::super::Boolean>>,
    /// outlineColor property of the polylinevolume graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the polylinevolume graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// positions property of the polylinevolume graphic.
    pub positions: Option<super::super::Property<super::super::PositionList>>,
    /// shadows property of the polylinevolume graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// shape property of the polylinevolume graphic.
    pub shape: Option<super::super::Property<super::super::Shape>>,
    /// show property of the polylinevolume graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
}
impl Validate for Box {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("dimensions", super::super::material::validate_optional(&self.dimensions));
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("fill", super::super::material::validate_optional(&self.fill));
        let report = report.merge("heightReference", super::super::material::validate_optional(&self.height_reference));
        let report = report.merge("material", super::super::material::validate_optional(&self.material));
        let report = report.merge("outline", super::super::material::validate_optional(&self.outline));
        let report = report.merge("outlineColor", super::super::material::validate_optional(&self.outline_color));
        let report = report.merge("outlineWidth", super::super::material::validate_optional(&self.outline_width));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        report.finish()
    }
}
impl Validate for Cylinder {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("bottomRadius", super::super::material::validate_optional(&self.bottom_radius));
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("fill", super::super::material::validate_optional(&self.fill));
        let report = report.merge("heightReference", super::super::material::validate_optional(&self.height_reference));
        let report = report.merge("length", super::super::material::validate_optional(&self.length));
        let report = report.merge("material", super::super::material::validate_optional(&self.material));
        let report = report.merge(
            "numberOfVerticalLines",
            super::super::material::validate_optional(&self.number_of_vertical_lines),
        );
        let report = report.merge("outline", super::super::material::validate_optional(&self.outline));
        let report = report.merge("outlineColor", super::super::material::validate_optional(&self.outline_color));
        let report = report.merge("outlineWidth", super::super::material::validate_optional(&self.outline_width));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        let report = report.merge("slices", super::super::material::validate_optional(&self.slices));
        let report = report.merge("topRadius", super::super::material::validate_optional(&self.top_radius));
        report.finish()
    }
}
impl Validate for Ellipsoid {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("fill", super::super::material::validate_optional(&self.fill));
        let report = report.merge("heightReference", super::super::material::validate_optional(&self.height_reference));
        let report = report.merge("innerRadii", super::super::material::validate_optional(&self.inner_radii));
        let report = report.merge("material", super::super::material::validate_optional(&self.material));
        let report = report.merge("maximumClock", super::super::material::validate_optional(&self.maximum_clock));
        let report = report.merge("maximumCone", super::super::material::validate_optional(&self.maximum_cone));
        let report = report.merge("minimumClock", super::super::material::validate_optional(&self.minimum_clock));
        let report = report.merge("minimumCone", super::super::material::validate_optional(&self.minimum_cone));
        let report = report.merge("outline", super::super::material::validate_optional(&self.outline));
        let report = report.merge("outlineColor", super::super::material::validate_optional(&self.outline_color));
        let report = report.merge("outlineWidth", super::super::material::validate_optional(&self.outline_width));
        let report = report.merge("radii", super::super::material::validate_optional(&self.radii));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        let report = report.merge("slicePartitions", super::super::material::validate_optional(&self.slice_partitions));
        let report = report.merge("stackPartitions", super::super::material::validate_optional(&self.stack_partitions));
        let report = report.merge("subdivisions", super::super::material::validate_optional(&self.subdivisions));
        report.finish()
    }
}
impl Validate for PolylineVolume {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("cornerType", super::super::material::validate_optional(&self.corner_type));
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("fill", super::super::material::validate_optional(&self.fill));
        let report = report.merge("granularity", super::super::material::validate_optional(&self.granularity));
        let report = report.merge("material", super::super::material::validate_optional(&self.material));
        let report = report.merge("outline", super::super::material::validate_optional(&self.outline));
        let report = report.merge("outlineColor", super::super::material::validate_optional(&self.outline_color));
        let report = report.merge("outlineWidth", super::super::material::validate_optional(&self.outline_width));
        let report = report.merge("positions", super::super::material::validate_optional(&self.positions));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("shape", super::super::material::validate_optional(&self.shape));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        report.finish()
    }
}
