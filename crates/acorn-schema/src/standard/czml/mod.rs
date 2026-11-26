//! Lossless, portable CZML 1.0 data types for authored time-dynamic scenes.
//!
//! This module represents source packets and values. It does not merge packets,
//! interpolate samples, resolve references, fetch assets, or render a scene.
mod enums;
mod reference;
mod time;
mod values;

pub use enums::{
    ArcTypeValue, ClassificationTypeValue, ClockRange, ClockStep, ColorBlendModeValue, CornerTypeValue, ExtrapolationType, HeightReferenceValue,
    HorizontalOriginValue, InterpolationAlgorithm, LabelStyleValue, PathModeValue, ReferenceFrame, ShadowModeValue, StripeOrientationValue,
    VerticalOriginValue,
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
