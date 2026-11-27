//! Typed CZML surface graphics.
use crate::validation::{Validate, ValidationReport};

/// A corridor, which is a shape defined by a centerline and width that conforms to the curvature of the globe.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Corridor {
    /// classificationType property of the corridor graphic.
    pub classification_type: Option<super::super::Property<super::super::ClassificationType>>,
    /// cornerType property of the corridor graphic.
    pub corner_type: Option<super::super::Property<super::super::CornerType>>,
    /// distanceDisplayCondition property of the corridor graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// extrudedHeight property of the corridor graphic.
    pub extruded_height: Option<super::super::Property<super::super::Double>>,
    /// extrudedHeightReference property of the corridor graphic.
    pub extruded_height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// fill property of the corridor graphic.
    pub fill: Option<super::super::Property<super::super::Boolean>>,
    /// granularity property of the corridor graphic.
    pub granularity: Option<super::super::Property<super::super::Double>>,
    /// height property of the corridor graphic.
    pub height: Option<super::super::Property<super::super::Double>>,
    /// heightReference property of the corridor graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// material property of the corridor graphic.
    pub material: Option<super::super::Material>,
    /// outline property of the corridor graphic.
    pub outline: Option<super::super::Property<super::super::Boolean>>,
    /// outlineColor property of the corridor graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the corridor graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// positions property of the corridor graphic.
    pub positions: Option<super::super::Property<super::super::PositionList>>,
    /// shadows property of the corridor graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// show property of the corridor graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// width property of the corridor graphic.
    pub width: Option<super::super::Property<super::super::Double>>,
    /// zIndex property of the corridor graphic.
    pub z_index: Option<super::super::Property<super::super::Integer>>,
}
/// An ellipse, which is a closed curve on or above the surface of the Earth.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Ellipse {
    /// classificationType property of the ellipse graphic.
    pub classification_type: Option<super::super::Property<super::super::ClassificationType>>,
    /// distanceDisplayCondition property of the ellipse graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// extrudedHeight property of the ellipse graphic.
    pub extruded_height: Option<super::super::Property<super::super::Double>>,
    /// extrudedHeightReference property of the ellipse graphic.
    pub extruded_height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// fill property of the ellipse graphic.
    pub fill: Option<super::super::Property<super::super::Boolean>>,
    /// granularity property of the ellipse graphic.
    pub granularity: Option<super::super::Property<super::super::Double>>,
    /// height property of the ellipse graphic.
    pub height: Option<super::super::Property<super::super::Double>>,
    /// heightReference property of the ellipse graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// material property of the ellipse graphic.
    pub material: Option<super::super::Material>,
    /// numberOfVerticalLines property of the ellipse graphic.
    pub number_of_vertical_lines: Option<super::super::Property<super::super::Integer>>,
    /// outline property of the ellipse graphic.
    pub outline: Option<super::super::Property<super::super::Boolean>>,
    /// outlineColor property of the ellipse graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the ellipse graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// rotation property of the ellipse graphic.
    pub rotation: Option<super::super::Property<super::super::Double>>,
    /// semiMajorAxis property of the ellipse graphic.
    pub semi_major_axis: Option<super::super::Property<super::super::Double>>,
    /// semiMinorAxis property of the ellipse graphic.
    pub semi_minor_axis: Option<super::super::Property<super::super::Double>>,
    /// shadows property of the ellipse graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// show property of the ellipse graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// stRotation property of the ellipse graphic.
    pub st_rotation: Option<super::super::Property<super::super::Double>>,
    /// zIndex property of the ellipse graphic.
    pub z_index: Option<super::super::Property<super::super::Integer>>,
}
/// A polygon, which is a closed figure on the surface of the Earth.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Polygon {
    /// arcType property of the polygon graphic.
    pub arc_type: Option<super::super::Property<super::super::ArcType>>,
    /// classificationType property of the polygon graphic.
    pub classification_type: Option<super::super::Property<super::super::ClassificationType>>,
    /// closeBottom property of the polygon graphic.
    pub close_bottom: Option<super::super::Property<super::super::Boolean>>,
    /// closeTop property of the polygon graphic.
    pub close_top: Option<super::super::Property<super::super::Boolean>>,
    /// distanceDisplayCondition property of the polygon graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// extrudedHeight property of the polygon graphic.
    pub extruded_height: Option<super::super::Property<super::super::Double>>,
    /// extrudedHeightReference property of the polygon graphic.
    pub extruded_height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// fill property of the polygon graphic.
    pub fill: Option<super::super::Property<super::super::Boolean>>,
    /// granularity property of the polygon graphic.
    pub granularity: Option<super::super::Property<super::super::Double>>,
    /// height property of the polygon graphic.
    pub height: Option<super::super::Property<super::super::Double>>,
    /// heightReference property of the polygon graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// holes property of the polygon graphic.
    pub holes: Option<super::super::Property<super::super::PositionListOfLists>>,
    /// material property of the polygon graphic.
    pub material: Option<super::super::Material>,
    /// outline property of the polygon graphic.
    pub outline: Option<super::super::Property<super::super::Boolean>>,
    /// outlineColor property of the polygon graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the polygon graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// perPositionHeight property of the polygon graphic.
    pub per_position_height: Option<super::super::Property<super::super::Boolean>>,
    /// positions property of the polygon graphic.
    pub positions: Option<super::super::Property<super::super::PositionList>>,
    /// shadows property of the polygon graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// show property of the polygon graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// stRotation property of the polygon graphic.
    pub st_rotation: Option<super::super::Property<super::super::Double>>,
    /// zIndex property of the polygon graphic.
    pub z_index: Option<super::super::Property<super::super::Integer>>,
}
/// A polyline, which is a line in the scene composed of multiple segments.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Polyline {
    /// arcType property of the polyline graphic.
    pub arc_type: Option<super::super::Property<super::super::ArcType>>,
    /// clampToGround property of the polyline graphic.
    pub clamp_to_ground: Option<super::super::Property<super::super::Boolean>>,
    /// classificationType property of the polyline graphic.
    pub classification_type: Option<super::super::Property<super::super::ClassificationType>>,
    /// depthFailMaterial property of the polyline graphic.
    pub depth_fail_material: Option<super::super::PolylineMaterial>,
    /// distanceDisplayCondition property of the polyline graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// followSurface property of the polyline graphic.
    pub follow_surface: Option<super::super::Property<super::super::Boolean>>,
    /// granularity property of the polyline graphic.
    pub granularity: Option<super::super::Property<super::super::Double>>,
    /// material property of the polyline graphic.
    pub material: Option<super::super::PolylineMaterial>,
    /// positions property of the polyline graphic.
    pub positions: Option<super::super::Property<super::super::PositionList>>,
    /// shadows property of the polyline graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// show property of the polyline graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// width property of the polyline graphic.
    pub width: Option<super::super::Property<super::super::Double>>,
    /// zIndex property of the polyline graphic.
    pub z_index: Option<super::super::Property<super::super::Integer>>,
}
/// A cartographic rectangle, which conforms to the curvature of the globe and can be placed on the surface or at altitude and can optionally be extruded into a volume.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Rectangle {
    /// classificationType property of the rectangle graphic.
    pub classification_type: Option<super::super::Property<super::super::ClassificationType>>,
    /// coordinates property of the rectangle graphic.
    pub coordinates: Option<super::super::Property<super::super::RectangleCoordinates>>,
    /// distanceDisplayCondition property of the rectangle graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// extrudedHeight property of the rectangle graphic.
    pub extruded_height: Option<super::super::Property<super::super::Double>>,
    /// extrudedHeightReference property of the rectangle graphic.
    pub extruded_height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// fill property of the rectangle graphic.
    pub fill: Option<super::super::Property<super::super::Boolean>>,
    /// granularity property of the rectangle graphic.
    pub granularity: Option<super::super::Property<super::super::Double>>,
    /// height property of the rectangle graphic.
    pub height: Option<super::super::Property<super::super::Double>>,
    /// heightReference property of the rectangle graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// material property of the rectangle graphic.
    pub material: Option<super::super::Material>,
    /// outline property of the rectangle graphic.
    pub outline: Option<super::super::Property<super::super::Boolean>>,
    /// outlineColor property of the rectangle graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the rectangle graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// rotation property of the rectangle graphic.
    pub rotation: Option<super::super::Property<super::super::Double>>,
    /// shadows property of the rectangle graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// show property of the rectangle graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// stRotation property of the rectangle graphic.
    pub st_rotation: Option<super::super::Property<super::super::Double>>,
    /// zIndex property of the rectangle graphic.
    pub z_index: Option<super::super::Property<super::super::Integer>>,
}
/// A two-dimensional wall defined as a line strip and optional maximum and minimum heights, which conforms to the curvature of the globe and can be placed along the surface or at altitude.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Wall {
    /// distanceDisplayCondition property of the wall graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// fill property of the wall graphic.
    pub fill: Option<super::super::Property<super::super::Boolean>>,
    /// granularity property of the wall graphic.
    pub granularity: Option<super::super::Property<super::super::Double>>,
    /// material property of the wall graphic.
    pub material: Option<super::super::Material>,
    /// maximumHeights property of the wall graphic.
    pub maximum_heights: Option<super::super::Property<super::super::DoubleList>>,
    /// minimumHeights property of the wall graphic.
    pub minimum_heights: Option<super::super::Property<super::super::DoubleList>>,
    /// outline property of the wall graphic.
    pub outline: Option<super::super::Property<super::super::Boolean>>,
    /// outlineColor property of the wall graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the wall graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// positions property of the wall graphic.
    pub positions: Option<super::super::Property<super::super::PositionList>>,
    /// shadows property of the wall graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// show property of the wall graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
}
impl Validate for Corridor {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("classificationType", super::super::material::validate_optional(&self.classification_type));
        let report = report.merge("cornerType", super::super::material::validate_optional(&self.corner_type));
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("extrudedHeight", super::super::material::validate_optional(&self.extruded_height));
        let report = report.merge(
            "extrudedHeightReference",
            super::super::material::validate_optional(&self.extruded_height_reference),
        );
        let report = report.merge("fill", super::super::material::validate_optional(&self.fill));
        let report = report.merge("granularity", super::super::material::validate_optional(&self.granularity));
        let report = report.merge("height", super::super::material::validate_optional(&self.height));
        let report = report.merge("heightReference", super::super::material::validate_optional(&self.height_reference));
        let report = report.merge("material", super::super::material::validate_optional(&self.material));
        let report = report.merge("outline", super::super::material::validate_optional(&self.outline));
        let report = report.merge("outlineColor", super::super::material::validate_optional(&self.outline_color));
        let report = report.merge("outlineWidth", super::super::material::validate_optional(&self.outline_width));
        let report = report.merge("positions", super::super::material::validate_optional(&self.positions));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        let report = report.merge("width", super::super::material::validate_optional(&self.width));
        let report = report.merge("zIndex", super::super::material::validate_optional(&self.z_index));
        report.finish()
    }
}
impl Validate for Ellipse {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("classificationType", super::super::material::validate_optional(&self.classification_type));
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("extrudedHeight", super::super::material::validate_optional(&self.extruded_height));
        let report = report.merge(
            "extrudedHeightReference",
            super::super::material::validate_optional(&self.extruded_height_reference),
        );
        let report = report.merge("fill", super::super::material::validate_optional(&self.fill));
        let report = report.merge("granularity", super::super::material::validate_optional(&self.granularity));
        let report = report.merge("height", super::super::material::validate_optional(&self.height));
        let report = report.merge("heightReference", super::super::material::validate_optional(&self.height_reference));
        let report = report.merge("material", super::super::material::validate_optional(&self.material));
        let report = report.merge(
            "numberOfVerticalLines",
            super::super::material::validate_optional(&self.number_of_vertical_lines),
        );
        let report = report.merge("outline", super::super::material::validate_optional(&self.outline));
        let report = report.merge("outlineColor", super::super::material::validate_optional(&self.outline_color));
        let report = report.merge("outlineWidth", super::super::material::validate_optional(&self.outline_width));
        let report = report.merge("rotation", super::super::material::validate_optional(&self.rotation));
        let report = report.merge("semiMajorAxis", super::super::material::validate_optional(&self.semi_major_axis));
        let report = report.merge("semiMinorAxis", super::super::material::validate_optional(&self.semi_minor_axis));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        let report = report.merge("stRotation", super::super::material::validate_optional(&self.st_rotation));
        let report = report.merge("zIndex", super::super::material::validate_optional(&self.z_index));
        report.finish()
    }
}
impl Validate for Polygon {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("arcType", super::super::material::validate_optional(&self.arc_type));
        let report = report.merge("classificationType", super::super::material::validate_optional(&self.classification_type));
        let report = report.merge("closeBottom", super::super::material::validate_optional(&self.close_bottom));
        let report = report.merge("closeTop", super::super::material::validate_optional(&self.close_top));
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("extrudedHeight", super::super::material::validate_optional(&self.extruded_height));
        let report = report.merge(
            "extrudedHeightReference",
            super::super::material::validate_optional(&self.extruded_height_reference),
        );
        let report = report.merge("fill", super::super::material::validate_optional(&self.fill));
        let report = report.merge("granularity", super::super::material::validate_optional(&self.granularity));
        let report = report.merge("height", super::super::material::validate_optional(&self.height));
        let report = report.merge("heightReference", super::super::material::validate_optional(&self.height_reference));
        let report = report.merge("holes", super::super::material::validate_optional(&self.holes));
        let report = report.merge("material", super::super::material::validate_optional(&self.material));
        let report = report.merge("outline", super::super::material::validate_optional(&self.outline));
        let report = report.merge("outlineColor", super::super::material::validate_optional(&self.outline_color));
        let report = report.merge("outlineWidth", super::super::material::validate_optional(&self.outline_width));
        let report = report.merge("perPositionHeight", super::super::material::validate_optional(&self.per_position_height));
        let report = report.merge("positions", super::super::material::validate_optional(&self.positions));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        let report = report.merge("stRotation", super::super::material::validate_optional(&self.st_rotation));
        let report = report.merge("zIndex", super::super::material::validate_optional(&self.z_index));
        report.finish()
    }
}
impl Validate for Polyline {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("arcType", super::super::material::validate_optional(&self.arc_type));
        let report = report.merge("clampToGround", super::super::material::validate_optional(&self.clamp_to_ground));
        let report = report.merge("classificationType", super::super::material::validate_optional(&self.classification_type));
        let report = report.merge("depthFailMaterial", super::super::material::validate_optional(&self.depth_fail_material));
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("followSurface", super::super::material::validate_optional(&self.follow_surface));
        let report = report.merge("granularity", super::super::material::validate_optional(&self.granularity));
        let report = report.merge("material", super::super::material::validate_optional(&self.material));
        let report = report.merge("positions", super::super::material::validate_optional(&self.positions));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        let report = report.merge("width", super::super::material::validate_optional(&self.width));
        let report = report.merge("zIndex", super::super::material::validate_optional(&self.z_index));
        report.finish()
    }
}
impl Validate for Rectangle {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("classificationType", super::super::material::validate_optional(&self.classification_type));
        let report = report.merge("coordinates", super::super::material::validate_optional(&self.coordinates));
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("extrudedHeight", super::super::material::validate_optional(&self.extruded_height));
        let report = report.merge(
            "extrudedHeightReference",
            super::super::material::validate_optional(&self.extruded_height_reference),
        );
        let report = report.merge("fill", super::super::material::validate_optional(&self.fill));
        let report = report.merge("granularity", super::super::material::validate_optional(&self.granularity));
        let report = report.merge("height", super::super::material::validate_optional(&self.height));
        let report = report.merge("heightReference", super::super::material::validate_optional(&self.height_reference));
        let report = report.merge("material", super::super::material::validate_optional(&self.material));
        let report = report.merge("outline", super::super::material::validate_optional(&self.outline));
        let report = report.merge("outlineColor", super::super::material::validate_optional(&self.outline_color));
        let report = report.merge("outlineWidth", super::super::material::validate_optional(&self.outline_width));
        let report = report.merge("rotation", super::super::material::validate_optional(&self.rotation));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        let report = report.merge("stRotation", super::super::material::validate_optional(&self.st_rotation));
        let report = report.merge("zIndex", super::super::material::validate_optional(&self.z_index));
        report.finish()
    }
}
impl Validate for Wall {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("fill", super::super::material::validate_optional(&self.fill));
        let report = report.merge("granularity", super::super::material::validate_optional(&self.granularity));
        let report = report.merge("material", super::super::material::validate_optional(&self.material));
        let report = report.merge("maximumHeights", super::super::material::validate_optional(&self.maximum_heights));
        let report = report.merge("minimumHeights", super::super::material::validate_optional(&self.minimum_heights));
        let report = report.merge("outline", super::super::material::validate_optional(&self.outline));
        let report = report.merge("outlineColor", super::super::material::validate_optional(&self.outline_color));
        let report = report.merge("outlineWidth", super::super::material::validate_optional(&self.outline_width));
        let report = report.merge("positions", super::super::material::validate_optional(&self.positions));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        report.finish()
    }
}
