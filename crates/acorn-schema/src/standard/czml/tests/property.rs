use super::super::macros::czml_property;
use super::super::{Cartesian3Value, DoubleValue, Property, Reference};
use crate::validation::Validate;
use serde_json::Value;

czml_property! {
    /// Boolean property used to test scalar and interval forms.
    struct BooleanExample: bool, true, deletable {
        /// Constant Boolean value.
        boolean: bool,
        /// Reference to another property.
        reference: Reference,
    }
}
czml_property! {
    /// Cartesian property used to test composite wrapping.
    struct CartesianExample: (), false, interpolatable {
        /// Three-dimensional Cartesian value.
        cartesian: Cartesian3Value,
        /// Reference to another property.
        reference: Reference,
    }
}
czml_property! {
    /// Numeric property used to test interpolation-only updates.
    struct NumericExample: f64, true, interpolatable {
        /// Numeric value.
        number: DoubleValue,
        /// Reference to another property.
        reference: Reference,
    }
}

#[test]
fn test_composite_property_requires_object_wrapper() {
    assert!(serde_json::from_str::<Property<CartesianExample>>("[1,2,3]").is_err());
    let source = serde_json::json!({"cartesian": [1.0, 2.0, 3.0]});
    let property: Property<CartesianExample> = serde_json::from_value(source.clone()).expect("wrapped composite property");
    assert!(property.validate().is_ok());
    assert_eq!(serde_json::to_value(property).expect("serialize composite"), source);
}
#[test]
fn test_composite_scalar_variant_is_rejected() {
    use super::super::{DoubleList, PositionList};

    let positions = Property::<PositionList>::Scalar(());
    let position_error = positions.validate().expect_err("PositionList scalar must be invalid");
    assert_eq!(
        position_error.into_issues().first().expect("one issue").error.code,
        "czml_property_scalar_form"
    );
    assert!(serde_json::to_value(&positions).is_err());

    let doubles = Property::<DoubleList>::Scalar(());
    assert!(doubles.validate().is_err());
    assert!(serde_json::to_value(&doubles).is_err());
}
#[test]
fn test_custom_property_accepts_each_value_key_and_rejects_conflicts() {
    use super::super::CustomProperty;

    let examples = [
        ("boolean", serde_json::json!(true)),
        ("boundingRectangle", serde_json::json!([1.0, 2.0, 3.0, 4.0])),
        ("cartesian", serde_json::json!([1.0, 2.0, 3.0])),
        ("cartesian2", serde_json::json!([1.0, 2.0])),
        ("cartographicDegrees", serde_json::json!([1.0, 2.0, 3.0])),
        ("cartographicRadians", serde_json::json!([1.0, 2.0, 3.0])),
        ("colorBlendMode", serde_json::json!("MIX")),
        ("cornerType", serde_json::json!("ROUNDED")),
        ("heightReference", serde_json::json!("NONE")),
        ("horizontalOrigin", serde_json::json!("CENTER")),
        ("labelStyle", serde_json::json!("FILL")),
        ("nearFarScalar", serde_json::json!([1.0, 2.0, 3.0, 4.0])),
        ("number", serde_json::json!(1.5)),
        ("pathMode", serde_json::json!("WHOLE")),
        ("rgba", serde_json::json!([255, 0, 0, 255])),
        ("rgbaf", serde_json::json!([1.0, 0.0, 0.0, 1.0])),
        ("shadowMode", serde_json::json!("ENABLED")),
        ("spherical", serde_json::json!([1.0, 2.0, 3.0])),
        ("string", serde_json::json!("example")),
        ("stripeOrientation", serde_json::json!("HORIZONTAL")),
        ("unitCartesian", serde_json::json!([1.0, 0.0, 0.0])),
        ("unitQuaternion", serde_json::json!([0.0, 0.0, 0.0, 1.0])),
        ("unitSpherical", serde_json::json!([1.0, 2.0])),
        ("uri", serde_json::json!("https://example.org")),
        ("verticalOrigin", serde_json::json!("CENTER")),
        ("wsen", serde_json::json!([1.0, 2.0, 3.0, 4.0])),
        ("wsenDegrees", serde_json::json!([1.0, 2.0, 3.0, 4.0])),
    ];
    for (key, value) in examples {
        let source = Value::Object(serde_json::Map::from_iter([(key.to_string(), value)]));
        let property: Property<CustomProperty> = serde_json::from_value(source.clone()).expect("custom value form");
        assert!(property.validate().is_ok(), "{key}");
        assert_eq!(serde_json::to_value(property).expect("serialize custom value form"), source);
    }
    let conflict: Property<CustomProperty> = serde_json::from_str(r#"{"boolean":true,"number":1}"#).expect("decode conflict");
    assert!(conflict.validate().is_err());
}
#[test]
fn test_delete_only_interval_and_interpolation_only_update_are_valid() {
    let deletion = serde_json::json!({"delete": true, "interval": "2016-01-01T00:00:00Z/2017-01-01T00:00:00Z"});
    let property: Property<BooleanExample> = serde_json::from_value(deletion.clone()).expect("delete-only interval");
    assert!(property.validate().is_ok());
    assert_eq!(serde_json::to_value(property).expect("serialize deletion"), deletion);
    let interpolation = serde_json::json!({"interpolationAlgorithm": "LINEAR", "interpolationDegree": 1});
    let property: Property<NumericExample> = serde_json::from_value(interpolation.clone()).expect("interpolation-only update");
    assert!(property.validate().is_ok());
    assert_eq!(serde_json::to_value(property).expect("serialize interpolation"), interpolation);
}
#[test]
fn test_position_color_and_orientation_examples_round_trip() {
    use super::super::{Color, Orientation, Position};

    let cartographic = serde_json::json!({"referenceFrame":"FIXED","cartographicDegrees":[-75.0,40.0,100.0]});
    let position: Property<Position> = serde_json::from_value(cartographic.clone()).expect("cartographic position");
    assert!(position.validate().is_ok());
    assert_eq!(serde_json::to_value(position).expect("serialize cartographic position"), cartographic);
    let sampled = serde_json::json!({"cartesian":["2016-01-01T00:00:00Z",1.0,2.0,3.0,"2016-01-01T00:00:01Z",4.0,5.0,6.0]});
    let position: Property<Position> = serde_json::from_value(sampled.clone()).expect("sampled Cartesian position");
    assert_eq!(serde_json::to_value(position).expect("serialize sampled position"), sampled);
    for color in [serde_json::json!({"rgba":[255,0,0,255]}), serde_json::json!({"rgbaf":[1.0,0.0,0.0,1.0]})] {
        let property: Property<Color> = serde_json::from_value(color.clone()).expect("color value form");
        assert_eq!(serde_json::to_value(property).expect("serialize color"), color);
    }
    let orientation = serde_json::json!({"velocityReference":"entity#position"});
    let property: Property<Orientation> = serde_json::from_value(orientation.clone()).expect("velocity orientation");
    assert_eq!(serde_json::to_value(property).expect("serialize orientation"), orientation);
}
#[test]
fn test_position_list_setting_does_not_conflict_with_its_value_form() {
    use super::super::{PositionList, Shape};

    let source = serde_json::json!({"referenceFrame":"INERTIAL","cartesian":[1.0,2.0,3.0]});
    let position: Property<PositionList> = serde_json::from_value(source.clone()).expect("PositionList object");
    assert!(position.validate().is_ok());
    assert_eq!(serde_json::to_value(position).expect("serialize position list"), source);
    assert!(serde_json::from_str::<Property<PositionList>>("[1,2,3]").is_err());
    let shape: Property<Shape> = serde_json::from_str(r#"{"cartesian2":[0,0,1,1]}"#).expect("wrapped Shape");
    assert!(shape.validate().is_ok());
    let deletion: Property<Shape> = serde_json::from_str(r#"{"delete":true}"#).expect("delete Shape");
    assert!(deletion.validate().is_ok());
}
#[test]
fn test_property_accepts_scalar_object_and_interval_list() {
    let scalar: Property<BooleanExample> = serde_json::from_str("true").expect("scalar shorthand");
    assert!(matches!(scalar, Property::Scalar(true)));
    let object = serde_json::json!({"boolean": true});
    let property: Property<BooleanExample> = serde_json::from_value(object.clone()).expect("one interval object");
    assert!(matches!(property, Property::Interval(_)));
    let intervals = serde_json::json!([{"boolean": true}, {"delete": true}]);
    let property: Property<BooleanExample> = serde_json::from_value(intervals.clone()).expect("interval list");
    assert!(property.validate().is_ok());
    assert_eq!(serde_json::to_value(property).expect("serialize intervals"), intervals);
}
#[test]
fn test_property_rejects_multiple_value_forms_and_unknown_fields() {
    let conflicting: Property<BooleanExample> =
        serde_json::from_str(r##"{"boolean":true,"reference":"entity#show"}"##).expect("decode conflicting value forms");
    let report = conflicting.validate().expect_err("multiple value forms rejected");
    assert_eq!(report.into_issues().first().expect("one issue").error.code, "czml_property_value_forms");
    assert!(serde_json::from_str::<Property<BooleanExample>>(r#"{"boolean":true,"unexpected":1}"#).is_err());
}
#[test]
fn test_property_schema_lists_only_allowed_envelope_shapes() {
    let boolean = serde_json::to_value(schemars::schema_for!(Property<BooleanExample>)).expect("Boolean property schema");
    let cartesian = serde_json::to_value(schemars::schema_for!(Property<CartesianExample>)).expect("Cartesian property schema");
    assert_eq!(boolean.get("oneOf").and_then(Value::as_array).map(Vec::len), Some(3));
    assert_eq!(cartesian.get("oneOf").and_then(Value::as_array).map(Vec::len), Some(2));
}
#[test]
fn test_real_property_schema_lists_only_supported_envelope_shapes() {
    use super::super::{ArcType, PositionList};

    let arc = serde_json::to_value(schemars::schema_for!(Property<ArcType>)).expect("ArcType schema");
    let positions = serde_json::to_value(schemars::schema_for!(Property<PositionList>)).expect("PositionList schema");
    assert_eq!(arc.get("oneOf").and_then(Value::as_array).map(Vec::len), Some(3));
    assert_eq!(positions.get("oneOf").and_then(Value::as_array).map(Vec::len), Some(2));
}
#[test]
fn test_scalar_and_enum_properties_accept_shorthand_and_wrapped_values() {
    use super::super::{ArcType, Boolean, Double, Integer, String as CzmlString};

    let boolean: Property<Boolean> = serde_json::from_str("true").expect("Boolean shorthand");
    assert!(matches!(boolean, Property::Scalar(true)));
    let text: Property<CzmlString> = serde_json::from_str(r#""example""#).expect("String shorthand");
    assert!(matches!(text, Property::Scalar(value) if value == "example"));
    let arc: Property<ArcType> = serde_json::from_str(r#""GEODESIC""#).expect("ArcType shorthand");
    assert!(matches!(arc, Property::Scalar(super::super::ArcTypeValue::Geodesic)));
    let integer: Property<Integer> = serde_json::from_str("12").expect("Integer shorthand");
    assert!(matches!(integer, Property::Scalar(12)));
    let double: Property<Double> =
        serde_json::from_str(r#"{"number":["2016-01-01T00:00:00Z",1.5],"interpolationAlgorithm":"LINEAR"}"#).expect("sampled Double");
    assert!(double.validate().is_ok());
}
