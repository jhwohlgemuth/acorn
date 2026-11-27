//! Lossless, portable CZML 1.0 data types for authored time-dynamic scenes.
//!
//! This module represents source packets and values. It does not merge packets, interpolate samples, resolve references, fetch assets, or render a scene.
//! CesiumJS 1.146 requires the first packet to have `id: "document"` and a `version` whose major number is 1.
//! Validation enforces that rule.
mod custom;
mod enums;
mod graphics;
mod macros;
mod material;
mod packet;
mod property;
mod reference;
mod time;
mod validate;
mod values;

pub use custom::CustomProperties;
pub use enums::{
    ArcTypeValue, ClassificationTypeValue, ClockRange, ClockStep, ColorBlendModeValue, CornerTypeValue, ExtrapolationType, HeightReferenceValue,
    HorizontalOriginValue, InterpolationAlgorithm, LabelStyleValue, PathModeValue, ReferenceFrame, ShadowModeValue, StripeOrientationValue,
    VerticalOriginValue,
};
pub use graphics::{
    Articulations, Billboard, Box, Corridor, Cylinder, Ellipse, Ellipsoid, Label, Model, NodeTransformation, NodeTransformations, OrderedMap, Path,
    Point, Polygon, Polyline, PolylineVolume, Rectangle, Tileset, Wall,
};
pub use material::{
    CheckerboardMaterial, GridMaterial, ImageMaterial, Material, PolylineArrowMaterial, PolylineDashMaterial, PolylineGlowMaterial, PolylineMaterial,
    PolylineOutlineMaterial, SolidColorMaterial, StripeMaterial,
};
pub use packet::{Clock, Document, Packet};
pub use property::{
    AlignedAxis, ArcType, Articulation, BackgroundPadding, Boolean, BoundingRectangle, BoxDimensions, ClassificationType, Color, ColorBlendMode,
    CornerType, CustomProperty, DistanceDisplayCondition, Double, DoubleList, EllipsoidRadii, EyeOffset, Font, HeightReference, HorizontalOrigin,
    Integer, LabelStyle, LineCount, LineOffset, LineThickness, NearFarScalar, Orientation, PathMode, PixelOffset, Position, PositionList,
    PositionListOfLists, PrimitiveValue, Property, PropertyForm, RectangleCoordinates, Repeat, Rotation, Scale, ShadowMode, Shape, String,
    StripeOrientation, Translation, Uri, VerticalOrigin, ViewFrom,
};
pub use reference::{Reference, VelocityReference};
pub use time::{SampleTime, Time, TimeInterval, TimeIntervalCollection};
pub use values::{
    BooleanValue, BoundingRectangleValue, Cartesian2ListValue, Cartesian2Value, Cartesian3ListOfListsValue, Cartesian3ListValue, Cartesian3Value,
    Cartesian3VelocityValue, CartographicDegreesListOfListsValue, CartographicDegreesListValue, CartographicDegreesValue,
    CartographicRadiansListOfListsValue, CartographicRadiansListValue, CartographicRadiansValue, CartographicRectangleDegreesValue,
    CartographicRectangleRadiansValue, DistanceDisplayConditionValue, DoubleListValue, DoubleValue, FixedValue, FlatList, FontValue,
    IntegerArrayValue, IntegerValue, ListOfLists, NearFarScalarValue, ReferenceListOfListsValue, ReferenceListValue, ReferenceValue, RgbaValue,
    RgbafValue, ScalarValue, SphericalListValue, SphericalValue, StringValue, UnitCartesian3ListValue, UnitCartesian3Value, UnitQuaternionValue,
    UnitSphericalListValue, UnitSphericalValue, UriValue,
};

/// CZML version modeled by this module.
pub const CZML_VERSION: &str = "1.0";
/// Pinned czml-writer prose revision used for interval and sample rules.
pub const PROSE_REVISION: &str = "017e7dc1616ae9c893d0b5d160b9f27146f6115c";
/// Pinned czml-writer schema revision used for field shapes.
pub const SCHEMA_REVISION: &str = "61cd1d5a890ad955da081250849b99c5fd238921";

#[cfg(test)]
mod tests;
