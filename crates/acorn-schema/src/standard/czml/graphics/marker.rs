//! Typed CZML marker graphics.
use crate::validation::{Validate, ValidationReport};

/// A billboard, or viewport-aligned image.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Billboard {
    /// alignedAxis property of the billboard graphic.
    pub aligned_axis: Option<super::super::Property<super::super::AlignedAxis>>,
    /// color property of the billboard graphic.
    pub color: Option<super::super::Property<super::super::Color>>,
    /// disableDepthTestDistance property of the billboard graphic.
    pub disable_depth_test_distance: Option<super::super::Property<super::super::Double>>,
    /// distanceDisplayCondition property of the billboard graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// eyeOffset property of the billboard graphic.
    pub eye_offset: Option<super::super::Property<super::super::EyeOffset>>,
    /// height property of the billboard graphic.
    pub height: Option<super::super::Property<super::super::Double>>,
    /// heightReference property of the billboard graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// horizontalOrigin property of the billboard graphic.
    pub horizontal_origin: Option<super::super::Property<super::super::HorizontalOrigin>>,
    /// image property of the billboard graphic.
    pub image: Option<super::super::Property<super::super::Uri>>,
    /// imageSubRegion property of the billboard graphic.
    pub image_sub_region: Option<super::super::Property<super::super::BoundingRectangle>>,
    /// pixelOffset property of the billboard graphic.
    pub pixel_offset: Option<super::super::Property<super::super::PixelOffset>>,
    /// pixelOffsetScaleByDistance property of the billboard graphic.
    pub pixel_offset_scale_by_distance: Option<super::super::Property<super::super::NearFarScalar>>,
    /// rotation property of the billboard graphic.
    pub rotation: Option<super::super::Property<super::super::Double>>,
    /// scale property of the billboard graphic.
    pub scale: Option<super::super::Property<super::super::Double>>,
    /// scaleByDistance property of the billboard graphic.
    pub scale_by_distance: Option<super::super::Property<super::super::NearFarScalar>>,
    /// show property of the billboard graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// sizeInMeters property of the billboard graphic.
    pub size_in_meters: Option<super::super::Property<super::super::Boolean>>,
    /// translucencyByDistance property of the billboard graphic.
    pub translucency_by_distance: Option<super::super::Property<super::super::NearFarScalar>>,
    /// verticalOrigin property of the billboard graphic.
    pub vertical_origin: Option<super::super::Property<super::super::VerticalOrigin>>,
    /// width property of the billboard graphic.
    pub width: Option<super::super::Property<super::super::Double>>,
}
/// A string of text.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Label {
    /// backgroundColor property of the label graphic.
    pub background_color: Option<super::super::Property<super::super::Color>>,
    /// backgroundPadding property of the label graphic.
    pub background_padding: Option<super::super::Property<super::super::BackgroundPadding>>,
    /// disableDepthTestDistance property of the label graphic.
    pub disable_depth_test_distance: Option<super::super::Property<super::super::Double>>,
    /// distanceDisplayCondition property of the label graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// eyeOffset property of the label graphic.
    pub eye_offset: Option<super::super::Property<super::super::EyeOffset>>,
    /// fillColor property of the label graphic.
    pub fill_color: Option<super::super::Property<super::super::Color>>,
    /// font property of the label graphic.
    pub font: Option<super::super::Property<super::super::Font>>,
    /// heightReference property of the label graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// horizontalOrigin property of the label graphic.
    pub horizontal_origin: Option<super::super::Property<super::super::HorizontalOrigin>>,
    /// outlineColor property of the label graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the label graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// pixelOffset property of the label graphic.
    pub pixel_offset: Option<super::super::Property<super::super::PixelOffset>>,
    /// pixelOffsetScaleByDistance property of the label graphic.
    pub pixel_offset_scale_by_distance: Option<super::super::Property<super::super::NearFarScalar>>,
    /// scale property of the label graphic.
    pub scale: Option<super::super::Property<super::super::Double>>,
    /// scaleByDistance property of the label graphic.
    pub scale_by_distance: Option<super::super::Property<super::super::NearFarScalar>>,
    /// show property of the label graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// showBackground property of the label graphic.
    pub show_background: Option<super::super::Property<super::super::Boolean>>,
    /// style property of the label graphic.
    pub style: Option<super::super::Property<super::super::LabelStyle>>,
    /// text property of the label graphic.
    pub text: Option<super::super::Property<super::super::String>>,
    /// translucencyByDistance property of the label graphic.
    pub translucency_by_distance: Option<super::super::Property<super::super::NearFarScalar>>,
    /// verticalOrigin property of the label graphic.
    pub vertical_origin: Option<super::super::Property<super::super::VerticalOrigin>>,
}
/// A path, which is a polyline defined by the motion of an object over time.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Path {
    /// distanceDisplayCondition property of the path graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// leadTime property of the path graphic.
    pub lead_time: Option<super::super::Property<super::super::Double>>,
    /// material property of the path graphic.
    pub material: Option<super::super::PolylineMaterial>,
    /// materialMode property of the path graphic.
    pub material_mode: Option<super::super::Property<super::super::PathMode>>,
    /// relativeTo property of the path graphic.
    pub relative_to: Option<super::super::Property<super::super::String>>,
    /// resolution property of the path graphic.
    pub resolution: Option<super::super::Property<super::super::Double>>,
    /// show property of the path graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// trailTime property of the path graphic.
    pub trail_time: Option<super::super::Property<super::super::Double>>,
    /// width property of the path graphic.
    pub width: Option<super::super::Property<super::super::Double>>,
}
/// A point, or viewport-aligned circle.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Point {
    /// color property of the point graphic.
    pub color: Option<super::super::Property<super::super::Color>>,
    /// disableDepthTestDistance property of the point graphic.
    pub disable_depth_test_distance: Option<super::super::Property<super::super::Double>>,
    /// distanceDisplayCondition property of the point graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// heightReference property of the point graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// outlineColor property of the point graphic.
    pub outline_color: Option<super::super::Property<super::super::Color>>,
    /// outlineWidth property of the point graphic.
    pub outline_width: Option<super::super::Property<super::super::Double>>,
    /// pixelSize property of the point graphic.
    pub pixel_size: Option<super::super::Property<super::super::Double>>,
    /// scaleByDistance property of the point graphic.
    pub scale_by_distance: Option<super::super::Property<super::super::NearFarScalar>>,
    /// show property of the point graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// translucencyByDistance property of the point graphic.
    pub translucency_by_distance: Option<super::super::Property<super::super::NearFarScalar>>,
}
/// A 3D Tiles tileset.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Tileset {
    /// maximumScreenSpaceError property of the tileset graphic.
    pub maximum_screen_space_error: Option<super::super::Property<super::super::Double>>,
    /// show property of the tileset graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// uri property of the tileset graphic.
    pub uri: Option<super::super::Property<super::super::Uri>>,
}
impl Validate for Billboard {
    fn validate(&self) -> Result<(), ValidationReport> {
        ValidationReport::new()
            .merge("alignedAxis", super::super::material::validate_optional(&self.aligned_axis))
            .merge("color", super::super::material::validate_optional(&self.color))
            .merge(
                "disableDepthTestDistance",
                super::super::material::validate_optional(&self.disable_depth_test_distance),
            )
            .merge(
                "distanceDisplayCondition",
                super::super::material::validate_optional(&self.distance_display_condition),
            )
            .merge("eyeOffset", super::super::material::validate_optional(&self.eye_offset))
            .merge("height", super::super::material::validate_optional(&self.height))
            .merge("heightReference", super::super::material::validate_optional(&self.height_reference))
            .merge("horizontalOrigin", super::super::material::validate_optional(&self.horizontal_origin))
            .merge("image", super::super::material::validate_optional(&self.image))
            .merge("imageSubRegion", super::super::material::validate_optional(&self.image_sub_region))
            .merge("pixelOffset", super::super::material::validate_optional(&self.pixel_offset))
            .merge(
                "pixelOffsetScaleByDistance",
                super::super::material::validate_optional(&self.pixel_offset_scale_by_distance),
            )
            .merge("rotation", super::super::material::validate_optional(&self.rotation))
            .merge("scale", super::super::material::validate_optional(&self.scale))
            .merge("scaleByDistance", super::super::material::validate_optional(&self.scale_by_distance))
            .merge("show", super::super::material::validate_optional(&self.show))
            .merge("sizeInMeters", super::super::material::validate_optional(&self.size_in_meters))
            .merge(
                "translucencyByDistance",
                super::super::material::validate_optional(&self.translucency_by_distance),
            )
            .merge("verticalOrigin", super::super::material::validate_optional(&self.vertical_origin))
            .merge("width", super::super::material::validate_optional(&self.width))
            .finish()
    }
}
impl Validate for Label {
    fn validate(&self) -> Result<(), ValidationReport> {
        ValidationReport::new()
            .merge("backgroundColor", super::super::material::validate_optional(&self.background_color))
            .merge("backgroundPadding", super::super::material::validate_optional(&self.background_padding))
            .merge(
                "disableDepthTestDistance",
                super::super::material::validate_optional(&self.disable_depth_test_distance),
            )
            .merge(
                "distanceDisplayCondition",
                super::super::material::validate_optional(&self.distance_display_condition),
            )
            .merge("eyeOffset", super::super::material::validate_optional(&self.eye_offset))
            .merge("fillColor", super::super::material::validate_optional(&self.fill_color))
            .merge("font", super::super::material::validate_optional(&self.font))
            .merge("heightReference", super::super::material::validate_optional(&self.height_reference))
            .merge("horizontalOrigin", super::super::material::validate_optional(&self.horizontal_origin))
            .merge("outlineColor", super::super::material::validate_optional(&self.outline_color))
            .merge("outlineWidth", super::super::material::validate_optional(&self.outline_width))
            .merge("pixelOffset", super::super::material::validate_optional(&self.pixel_offset))
            .merge(
                "pixelOffsetScaleByDistance",
                super::super::material::validate_optional(&self.pixel_offset_scale_by_distance),
            )
            .merge("scale", super::super::material::validate_optional(&self.scale))
            .merge("scaleByDistance", super::super::material::validate_optional(&self.scale_by_distance))
            .merge("show", super::super::material::validate_optional(&self.show))
            .merge("showBackground", super::super::material::validate_optional(&self.show_background))
            .merge("style", super::super::material::validate_optional(&self.style))
            .merge("text", super::super::material::validate_optional(&self.text))
            .merge(
                "translucencyByDistance",
                super::super::material::validate_optional(&self.translucency_by_distance),
            )
            .merge("verticalOrigin", super::super::material::validate_optional(&self.vertical_origin))
            .finish()
    }
}
impl Validate for Path {
    fn validate(&self) -> Result<(), ValidationReport> {
        ValidationReport::new()
            .merge(
                "distanceDisplayCondition",
                super::super::material::validate_optional(&self.distance_display_condition),
            )
            .merge("leadTime", super::super::material::validate_optional(&self.lead_time))
            .merge("material", super::super::material::validate_optional(&self.material))
            .merge("materialMode", super::super::material::validate_optional(&self.material_mode))
            .merge("relativeTo", super::super::material::validate_optional(&self.relative_to))
            .merge("resolution", super::super::material::validate_optional(&self.resolution))
            .merge("show", super::super::material::validate_optional(&self.show))
            .merge("trailTime", super::super::material::validate_optional(&self.trail_time))
            .merge("width", super::super::material::validate_optional(&self.width))
            .finish()
    }
}
impl Validate for Point {
    fn validate(&self) -> Result<(), ValidationReport> {
        ValidationReport::new()
            .merge("color", super::super::material::validate_optional(&self.color))
            .merge(
                "disableDepthTestDistance",
                super::super::material::validate_optional(&self.disable_depth_test_distance),
            )
            .merge(
                "distanceDisplayCondition",
                super::super::material::validate_optional(&self.distance_display_condition),
            )
            .merge("heightReference", super::super::material::validate_optional(&self.height_reference))
            .merge("outlineColor", super::super::material::validate_optional(&self.outline_color))
            .merge("outlineWidth", super::super::material::validate_optional(&self.outline_width))
            .merge("pixelSize", super::super::material::validate_optional(&self.pixel_size))
            .merge("scaleByDistance", super::super::material::validate_optional(&self.scale_by_distance))
            .merge("show", super::super::material::validate_optional(&self.show))
            .merge(
                "translucencyByDistance",
                super::super::material::validate_optional(&self.translucency_by_distance),
            )
            .finish()
    }
}
impl Validate for Tileset {
    fn validate(&self) -> Result<(), ValidationReport> {
        ValidationReport::new()
            .merge(
                "maximumScreenSpaceError",
                super::super::material::validate_optional(&self.maximum_screen_space_error),
            )
            .merge("show", super::super::material::validate_optional(&self.show))
            .merge("uri", super::super::material::validate_optional(&self.uri))
            .finish()
    }
}
