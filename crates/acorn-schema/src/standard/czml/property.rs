//! CZML property envelopes and shared property fields.
use super::macros::czml_property;
use crate::validation::{Validate, ValidationReport};
use acorn_core::prelude::alloc::{format, vec, String as RustString, Vec};
use core::{fmt, marker::PhantomData};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::de::{DeserializeOwned, Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The object and optional scalar forms accepted by a CZML property.
pub trait PropertyForm: Sized {
    /// Scalar shorthand type, or `()` when shorthand is unsupported.
    type Scalar: Clone + fmt::Debug + DeserializeOwned + JsonSchema + PartialEq + Serialize;
    /// Whether a bare scalar may represent the property.
    const ALLOW_SCALAR: bool;
}
/// Bare primitive shorthand accepted by custom CZML properties.
#[derive(Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum PrimitiveValue {
    /// Boolean constant.
    Boolean(bool),
    /// Numeric constant.
    Number(f64),
    /// Text constant.
    Text(RustString),
}
/// One CZML property written as interval objects or an allowed scalar shorthand.
#[derive(Clone, Debug, PartialEq)]
pub enum Property<T: PropertyForm> {
    /// Ordered intervals, including an empty list.
    Intervals(Vec<T>),
    /// One property object, optionally restricted by its `interval` field.
    Interval(T),
    /// Constant scalar shorthand for a scalar-valued property.
    Scalar(T::Scalar),
}
struct PropertyVisitor<T>(PhantomData<T>);
impl<'de, T: Deserialize<'de> + PropertyForm> Deserialize<'de> for Property<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(PropertyVisitor(PhantomData))
    }
}
impl<T: PropertyForm + JsonSchema> JsonSchema for Property<T> {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        format!("PropertyOf{}", T::schema_name()).into()
    }
    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let intervals = generator.subschema_for::<Vec<T>>();
        let interval = generator.subschema_for::<T>();
        let forms = if T::ALLOW_SCALAR {
            vec![intervals, interval, generator.subschema_for::<T::Scalar>()]
        } else {
            vec![intervals, interval]
        };
        json_schema!({"oneOf": forms})
    }
}
impl<T: PropertyForm + Serialize> Serialize for Property<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            | Self::Intervals(intervals) => intervals.serialize(serializer),
            | Self::Interval(interval) => interval.serialize(serializer),
            | Self::Scalar(value) if T::ALLOW_SCALAR => value.serialize(serializer),
            | Self::Scalar(_) => Err(serde::ser::Error::custom("composite CZML values require a property object")),
        }
    }
}
impl<T: PropertyForm + Validate> Validate for Property<T> {
    fn validate(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::Intervals(intervals) => intervals.validate(),
            | Self::Interval(interval) => interval.validate(),
            | Self::Scalar(_) if T::ALLOW_SCALAR => Ok(()),
            | Self::Scalar(_) => Err(ValidationReport::from_error(
                "",
                acorn_core::validation::ValidationError::new("czml_property_scalar_form")
                    .with_message("Composite CZML values require a property object"),
            )),
        }
    }
}
impl<'de, T: Deserialize<'de> + PropertyForm> Visitor<'de> for PropertyVisitor<T> {
    type Value = Property<T>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a CZML interval list, property object, or allowed scalar")
    }
    fn visit_bool<E: DeError>(self, value: bool) -> Result<Self::Value, E> {
        scalar::<T, E>(serde_json::Value::Bool(value))
    }
    fn visit_f64<E: DeError>(self, value: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .ok_or_else(|| E::custom("property scalar must be finite"))
            .and_then(scalar::<T, E>)
    }
    fn visit_i64<E: DeError>(self, value: i64) -> Result<Self::Value, E> {
        scalar::<T, E>(serde_json::Value::Number(value.into()))
    }
    fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
        T::deserialize(serde::de::value::MapAccessDeserializer::new(map)).map(Property::Interval)
    }
    fn visit_seq<S: SeqAccess<'de>>(self, sequence: S) -> Result<Self::Value, S::Error> {
        Vec::<T>::deserialize(serde::de::value::SeqAccessDeserializer::new(sequence)).map(Property::Intervals)
    }
    fn visit_str<E: DeError>(self, value: &str) -> Result<Self::Value, E> {
        scalar::<T, E>(serde_json::Value::String(value.into()))
    }
    fn visit_string<E: DeError>(self, value: RustString) -> Result<Self::Value, E> {
        scalar::<T, E>(serde_json::Value::String(value))
    }
    fn visit_u64<E: DeError>(self, value: u64) -> Result<Self::Value, E> {
        scalar::<T, E>(serde_json::Value::Number(value.into()))
    }
}
fn scalar<T: PropertyForm, E: DeError>(value: serde_json::Value) -> Result<Property<T>, E> {
    match T::ALLOW_SCALAR {
        | true => serde_json::from_value::<T::Scalar>(value).map(Property::Scalar).map_err(E::custom),
        | false => Err(E::custom("composite CZML values require a property object")),
    }
}
// Property definitions are sorted by CZML type name.
czml_property! {
    /// CZML ArcType property with constant values.
    pub struct ArcType: super::ArcTypeValue, true, deletable {
        /// Arc type value form.
        arc_type: super::ArcTypeValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML Boolean property with constant values.
    pub struct Boolean: bool, true, deletable {
        /// Boolean value form.
        boolean: super::BooleanValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML ClassificationType property with constant values.
    pub struct ClassificationType: super::ClassificationTypeValue, true, deletable {
        /// Classification type value form.
        classification_type: super::ClassificationTypeValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML ColorBlendMode property with constant values.
    pub struct ColorBlendMode: super::ColorBlendModeValue, true, deletable {
        /// Color blend mode value form.
        color_blend_mode: super::ColorBlendModeValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML CornerType property with constant values.
    pub struct CornerType: super::CornerTypeValue, true, deletable {
        /// Corner type value form.
        corner_type: super::CornerTypeValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML Double property with sampled and constant values.
    pub struct Double: f64, true, interpolatable {
        /// Number value form.
        number: super::DoubleValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML DoubleList property with constant values.
    pub struct DoubleList: (), false, deletable {
        /// Array value form.
        array: super::DoubleListValue,
        /// References value form.
        references: super::ReferenceListValue,
    }
}
czml_property! {
    /// CZML Font property with constant values.
    pub struct Font: alloc::string::String, true, deletable {
        /// Font value form.
        font: super::FontValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML HeightReference property with constant values.
    pub struct HeightReference: super::HeightReferenceValue, true, deletable {
        /// Height reference value form.
        height_reference: super::HeightReferenceValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML HorizontalOrigin property with constant values.
    pub struct HorizontalOrigin: super::HorizontalOriginValue, true, deletable {
        /// Horizontal origin value form.
        horizontal_origin: super::HorizontalOriginValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML Integer property with sampled and constant values.
    pub struct Integer: i64, true, interpolatable {
        /// Number value form.
        number: super::IntegerValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML LabelStyle property with constant values.
    pub struct LabelStyle: super::LabelStyleValue, true, deletable {
        /// Label style value form.
        label_style: super::LabelStyleValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML PathMode property with constant values.
    pub struct PathMode: super::PathModeValue, true, deletable {
        /// Path mode value form.
        path_mode: super::PathModeValue,
        /// Reference value form.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// CZML PositionList property with constant values.
    pub struct PositionList: (), false, deletable,
    metadata {
        /// Reference frame for the encoded coordinates.
        reference_frame: super::ReferenceFrame,
    }
    values {
        /// Cartesian value form.
        cartesian: super::Cartesian3ListValue,
        /// Cartographic degrees value form.
        cartographic_degrees: super::CartographicDegreesListValue,
        /// Cartographic radians value form.
        cartographic_radians: super::CartographicRadiansListValue,
        /// References value form.
        references: super::ReferenceListValue,
    }
}
czml_property! {
    /// CZML PositionListOfLists property with constant values.
    pub struct PositionListOfLists: (), false, deletable {
        /// Cartesian value form.
        cartesian: super::Cartesian3ListOfListsValue,
        /// Cartographic degrees value form.
        cartographic_degrees: super::CartographicDegreesListOfListsValue,
        /// Cartographic radians value form.
        cartographic_radians: super::CartographicRadiansListOfListsValue,
        /// References value form.
        references: super::ReferenceListOfListsValue,
    }
}
czml_property! {
    /// CZML ShadowMode property with constant values.
    pub struct ShadowMode: super::ShadowModeValue, true, deletable {
        /// Reference value form.
        reference: super::ReferenceValue,
        /// Shadow mode value form.
        shadow_mode: super::ShadowModeValue,
    }
}
czml_property! {
    /// CZML Shape property with constant values.
    pub struct Shape: (), false, deletable {
        /// Cartesian2 value form.
        cartesian2: super::Cartesian2ListValue,
    }
}
czml_property! {
    /// CZML String property with constant values.
    pub struct String: alloc::string::String, true, deletable {
        /// Reference value form.
        reference: super::ReferenceValue,
        /// String value form.
        string: super::StringValue,
    }
}
czml_property! {
    /// CZML StripeOrientation property with constant values.
    pub struct StripeOrientation: super::StripeOrientationValue, true, deletable {
        /// Reference value form.
        reference: super::ReferenceValue,
        /// Stripe orientation value form.
        stripe_orientation: super::StripeOrientationValue,
    }
}
czml_property! {
    /// CZML Uri property with constant values.
    pub struct Uri: alloc::string::String, true, deletable {
        /// Reference value form.
        reference: super::ReferenceValue,
        /// Uri value form.
        uri: super::UriValue,
    }
}
czml_property! {
    /// CZML VerticalOrigin property with constant values.
    pub struct VerticalOrigin: super::VerticalOriginValue, true, deletable {
        /// Reference value form.
        reference: super::ReferenceValue,
        /// Vertical origin value form.
        vertical_origin: super::VerticalOriginValue,
    }
}
czml_property! {
    /// AlignedAxis property with time-varying values and interpolation settings.
    pub struct AlignedAxis: (), false, interpolatable {
        /// Cartesian spelling retained by the pinned Billboard example.
        cartesian: super::Cartesian3Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
        /// Unit cartesian encoded value.
        unit_cartesian: super::UnitCartesian3Value,
        /// Unit spherical encoded value.
        unit_spherical: super::UnitSphericalValue,
        /// Direction derived from a referenced position velocity.
        velocity_reference: super::VelocityReference,
    }
}
czml_property! {
    /// Articulation property with time-varying values and interpolation settings.
    pub struct Articulation: f64, true, interpolatable {
        /// Number encoded value.
        number: super::DoubleValue,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// BackgroundPadding property with time-varying values and interpolation settings.
    pub struct BackgroundPadding: (), false, interpolatable {
        /// Cartesian2 encoded value.
        cartesian2: super::Cartesian2Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// BoundingRectangle property with time-varying values and interpolation settings.
    pub struct BoundingRectangle: (), false, interpolatable {
        /// Bounding rectangle encoded value.
        bounding_rectangle: super::BoundingRectangleValue,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// BoxDimensions property with time-varying values and interpolation settings.
    pub struct BoxDimensions: (), false, interpolatable {
        /// Cartesian encoded value.
        cartesian: super::Cartesian3Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// Color property with time-varying values and interpolation settings.
    pub struct Color: (), false, interpolatable {
        /// Reference to another entity property.
        reference: super::ReferenceValue,
        /// Rgba encoded value.
        rgba: super::RgbaValue,
        /// Rgbaf encoded value.
        rgbaf: super::RgbafValue,
    }
}
czml_property! {
    /// CustomProperty property with time-varying values and interpolation settings.
    pub struct CustomProperty: PrimitiveValue, true, interpolatable {
        /// Boolean encoded value.
        boolean: super::BooleanValue,
        /// Bounding rectangle encoded value.
        bounding_rectangle: super::BoundingRectangleValue,
        /// Cartesian encoded value.
        cartesian: super::Cartesian3Value,
        /// Cartesian2 encoded value.
        cartesian2: super::Cartesian2Value,
        /// Cartographic degrees encoded value.
        cartographic_degrees: super::CartographicDegreesValue,
        /// Cartographic radians encoded value.
        cartographic_radians: super::CartographicRadiansValue,
        /// Color blend mode encoded value.
        color_blend_mode: super::ColorBlendModeValue,
        /// Corner type encoded value.
        corner_type: super::CornerTypeValue,
        /// Height reference encoded value.
        height_reference: super::HeightReferenceValue,
        /// Horizontal origin encoded value.
        horizontal_origin: super::HorizontalOriginValue,
        /// Label style encoded value.
        label_style: super::LabelStyleValue,
        /// Near far scalar encoded value.
        near_far_scalar: super::NearFarScalarValue,
        /// Number encoded value.
        number: super::DoubleValue,
        /// Path mode encoded value.
        path_mode: super::PathModeValue,
        /// Rgba encoded value.
        rgba: super::RgbaValue,
        /// Rgbaf encoded value.
        rgbaf: super::RgbafValue,
        /// Shadow mode encoded value.
        shadow_mode: super::ShadowModeValue,
        /// Spherical encoded value.
        spherical: super::SphericalValue,
        /// String encoded value.
        string: super::StringValue,
        /// Stripe orientation encoded value.
        stripe_orientation: super::StripeOrientationValue,
        /// Unit cartesian encoded value.
        unit_cartesian: super::UnitCartesian3Value,
        /// Unit quaternion encoded value.
        unit_quaternion: super::UnitQuaternionValue,
        /// Unit spherical encoded value.
        unit_spherical: super::UnitSphericalValue,
        /// Uri encoded value.
        uri: super::UriValue,
        /// Vertical origin encoded value.
        vertical_origin: super::VerticalOriginValue,
        /// Wsen encoded value.
        wsen: super::CartographicRectangleRadiansValue,
        /// Wsen degrees encoded value.
        wsen_degrees: super::CartographicRectangleDegreesValue,
    }
}
czml_property! {
    /// DistanceDisplayCondition property with time-varying values and interpolation settings.
    pub struct DistanceDisplayCondition: (), false, interpolatable {
        /// Distance display condition encoded value.
        distance_display_condition: super::DistanceDisplayConditionValue,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// EllipsoidRadii property with time-varying values and interpolation settings.
    pub struct EllipsoidRadii: (), false, interpolatable {
        /// Cartesian encoded value.
        cartesian: super::Cartesian3Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// EyeOffset property with time-varying values and interpolation settings.
    pub struct EyeOffset: (), false, interpolatable {
        /// Cartesian encoded value.
        cartesian: super::Cartesian3Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// LineCount property with time-varying values and interpolation settings.
    pub struct LineCount: (), false, interpolatable {
        /// Cartesian2 encoded value.
        cartesian2: super::Cartesian2Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// LineOffset property with time-varying values and interpolation settings.
    pub struct LineOffset: (), false, interpolatable {
        /// Cartesian2 encoded value.
        cartesian2: super::Cartesian2Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// LineThickness property with time-varying values and interpolation settings.
    pub struct LineThickness: (), false, interpolatable {
        /// Cartesian2 encoded value.
        cartesian2: super::Cartesian2Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// NearFarScalar property with time-varying values and interpolation settings.
    pub struct NearFarScalar: (), false, interpolatable {
        /// Near far scalar encoded value.
        near_far_scalar: super::NearFarScalarValue,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// Orientation property with time-varying values and interpolation settings.
    pub struct Orientation: (), false, interpolatable {
        /// Reference to another entity property.
        reference: super::ReferenceValue,
        /// Unit quaternion encoded value.
        unit_quaternion: super::UnitQuaternionValue,
        /// Direction derived from a referenced position velocity.
        velocity_reference: super::VelocityReference,
    }
}
czml_property! {
    /// PixelOffset property with time-varying values and interpolation settings.
    pub struct PixelOffset: (), false, interpolatable {
        /// Cartesian2 encoded value.
        cartesian2: super::Cartesian2Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// Position property with time-varying values and interpolation settings.
    pub struct Position: (), false, interpolatable,
    metadata {
        /// Frame used to interpret coordinate samples.
        reference_frame: super::ReferenceFrame,
    }
    values {
        /// Cartesian encoded value.
        cartesian: super::Cartesian3Value,
        /// Cartesian velocity encoded value.
        cartesian_velocity: super::Cartesian3VelocityValue,
        /// Cartographic degrees encoded value.
        cartographic_degrees: super::CartographicDegreesValue,
        /// Cartographic radians encoded value.
        cartographic_radians: super::CartographicRadiansValue,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// RectangleCoordinates property with time-varying values and interpolation settings.
    pub struct RectangleCoordinates: (), false, interpolatable {
        /// Reference to another entity property.
        reference: super::ReferenceValue,
        /// Wsen encoded value.
        wsen: super::CartographicRectangleRadiansValue,
        /// Wsen degrees encoded value.
        wsen_degrees: super::CartographicRectangleDegreesValue,
    }
}
czml_property! {
    /// Repeat property with time-varying values and interpolation settings.
    pub struct Repeat: (), false, interpolatable {
        /// Cartesian2 encoded value.
        cartesian2: super::Cartesian2Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// Rotation property with time-varying values and interpolation settings.
    pub struct Rotation: (), false, interpolatable {
        /// Reference to another entity property.
        reference: super::ReferenceValue,
        /// Unit quaternion encoded value.
        unit_quaternion: super::UnitQuaternionValue,
    }
}
czml_property! {
    /// Scale property with time-varying values and interpolation settings.
    pub struct Scale: (), false, interpolatable {
        /// Cartesian encoded value.
        cartesian: super::Cartesian3Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// Translation property with time-varying values and interpolation settings.
    pub struct Translation: (), false, interpolatable {
        /// Cartesian encoded value.
        cartesian: super::Cartesian3Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
czml_property! {
    /// ViewFrom property with time-varying values and interpolation settings.
    pub struct ViewFrom: (), false, interpolatable {
        /// Cartesian encoded value.
        cartesian: super::Cartesian3Value,
        /// Reference to another entity property.
        reference: super::ReferenceValue,
    }
}
