use super::super::{
    Articulations, Billboard, BoundingRectangleValue, Cartesian3Value, CartographicDegreesValue, CartographicRectangleDegreesValue,
    DistanceDisplayConditionValue, Document, Model, NearFarScalarValue, NodeTransformation, Packet, Property, Reference, Rotation, Tileset,
    UnitQuaternionValue, UriValue,
};
use crate::codec::DocumentCodec;
use crate::validation::Validate;
use acorn_core::util::MimeType;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/schema/czml/");
fn check_fields<T: schemars::JsonSchema>(inventory: &Value, name: &str, property: bool) {
    let schema = serde_json::to_value(schemars::schema_for!(T)).expect("typed schema");
    let mut actual = schema
        .get("properties")
        .and_then(Value::as_object)
        .map(|fields| fields.keys().cloned().collect::<std::collections::BTreeSet<_>>())
        .unwrap_or_default();
    let mut expected = inventory
        .get("types")
        .and_then(|types| types.get(name))
        .and_then(|record| record.get("properties"))
        .and_then(Value::as_array)
        .expect("pinned type")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<std::collections::BTreeSet<_>>();
    if property {
        for base in [
            "interval",
            "delete",
            "epoch",
            "interpolationAlgorithm",
            "interpolationDegree",
            "forwardExtrapolationType",
            "forwardExtrapolationDuration",
            "backwardExtrapolationType",
            "backwardExtrapolationDuration",
        ] {
            actual.remove(base);
        }
    }
    if name == "Packet" {
        expected.retain(|field| !field.starts_with("agi_"));
    }
    if name == "Clock" || name == "NodeTransformation" {
        actual.remove("interval");
    }
    if name == "AlignedAxis" {
        actual.remove("cartesian");
    }
    assert_eq!(actual, expected, "{name} field set");
    let required_for_display = inventory
        .get("types")
        .and_then(|types| types.get(name))
        .and_then(|record| record.get("requiredForDisplay"))
        .and_then(Value::as_array)
        .expect("pinned required-for-display");
    let required = schema.get("required").and_then(Value::as_array);
    for field in required_for_display {
        assert!(
            !required.is_some_and(|required| required.contains(field)),
            "{name}: display field must stay optional"
        );
    }
}
fn numerically_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        | (Value::Number(left), Value::Number(right)) => left.as_f64() == right.as_f64(),
        | (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len() && left.iter().zip(right).all(|(left, right)| numerically_equal(left, right))
        }
        | (Value::Object(left), Value::Object(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .all(|(key, left)| right.get(key).is_some_and(|right| numerically_equal(left, right)))
        }
        | _ => left == right,
    }
}
fn round_trip<T: DeserializeOwned + Serialize>(name: &str, content: &str) -> Value {
    let decoded: T = serde_json::from_str(content).expect(name);
    serde_json::to_value(decoded).expect(name)
}

#[test]
fn test_fictional_scene_and_malformed_cases() {
    let scene = std::fs::read_to_string(std::path::Path::new(ROOT).join("acorn/scene.czml")).expect("fictional scene");
    let document = Document::decode(&scene, &MimeType::Json).expect("decode fictional scene");
    assert!(document.validate().is_ok());
    let malformed = std::fs::read_to_string(std::path::Path::new(ROOT).join("acorn/malformed.json")).expect("malformed cases");
    let cases: serde_json::Map<String, Value> = serde_json::from_str(&malformed).expect("malformed case map");
    for (name, value) in cases {
        let content = serde_json::to_string(&value).expect("serialize malformed case");
        match name.as_str() {
            | "bareComposite" | "rgbaFraction" | "unknownSubproperty" | "wrongArity" => {
                let diagnostics = Document::decode(&content, &MimeType::Json).expect_err("invalid value encoding");
                assert!(
                    diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.code == "deserialize" && diagnostic.path.is_some()),
                    "{name}"
                );
            }
            | _ => {
                let document = Document::decode(&content, &MimeType::Json).expect("valid encoding with invalid semantics");
                let issues = document.validate().expect_err("invalid CZML semantics").into_issues();
                let code = match name.as_str() {
                    | "invalidInterval" => "czml_interval",
                    | "invalidTime" => "czml_time",
                    | "malformedReference" => "czml_reference",
                    | "outOfOrderSamples" => "czml_sample_order",
                    | _ => "czml_document_field",
                };
                assert!(issues.iter().any(|issue| issue.error.code == code && !issue.path.is_empty()), "{name}");
            }
        }
    }
}
#[test]
fn test_pinned_czml_writer_examples_decode_and_round_trip() {
    let directory = std::path::Path::new(ROOT).join("czml-writer");
    let mut paths = std::fs::read_dir(directory)
        .expect("fixture directory")
        .map(|entry| entry.expect("fixture entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 36);
    for path in paths {
        let name = path.file_stem().and_then(|name| name.to_str()).expect("fixture name");
        let content = std::fs::read_to_string(&path).expect("read pinned fixture");
        let original: Value = serde_json::from_str(&content).expect("pinned JSON");
        let encoded = match name {
            | "BoundingRectangleValue" | "BoundingRectangleValueSamples" => round_trip::<BoundingRectangleValue>(name, &content),
            | "Cartesian3Value" | "Cartesian3ValueSamples" => round_trip::<Cartesian3Value>(name, &content),
            | "CartographicDegreesValue" | "CartographicDegreesValueSamples" => round_trip::<CartographicDegreesValue>(name, &content),
            | "CartographicRectangleDegreesValue" | "CartographicRectangleDegreesValueSamples" => {
                round_trip::<CartographicRectangleDegreesValue>(name, &content)
            }
            | "DistanceDisplayConditionValue" | "DistanceDisplayConditionValueSamples" => round_trip::<DistanceDisplayConditionValue>(name, &content),
            | "NearFarScalarValue" | "NearFarScalarValueSamples" => round_trip::<NearFarScalarValue>(name, &content),
            | "UnitQuaternionValue" | "UnitQuaternionValueSamples" => round_trip::<UnitQuaternionValue>(name, &content),
            | "ReferenceValue" => round_trip::<Reference>(name, &content),
            | "UriValue" | "UriValueData" => round_trip::<UriValue>(name, &content),
            | "Uri" => round_trip::<Tileset>(name, &content),
            | "Articulations" | "NodeTransformations" => round_trip::<Model>(name, &content),
            | "Rotation" | "RotationSampled" => round_trip::<NodeTransformation>(name, &content),
            | _ => round_trip::<Packet>(name, &content),
        };
        assert!(numerically_equal(&original, &encoded), "{name}: re-encoded JSON differs");
    }
}
#[test]
fn test_pinned_schema_inventory_matches_typed_fields() {
    let source = std::fs::read_to_string(std::path::Path::new(ROOT).join("schema-inventory.json")).expect("pinned schema inventory");
    let inventory: Value = serde_json::from_str(&source).expect("schema inventory JSON");
    check_fields::<super::super::AlignedAxis>(&inventory, "AlignedAxis", true);
    check_fields::<super::super::ArcType>(&inventory, "ArcType", true);
    check_fields::<super::super::Articulation>(&inventory, "Articulation", true);
    check_fields::<super::super::BackgroundPadding>(&inventory, "BackgroundPadding", true);
    check_fields::<super::super::Billboard>(&inventory, "Billboard", false);
    check_fields::<super::super::Boolean>(&inventory, "Boolean", true);
    check_fields::<super::super::BoundingRectangle>(&inventory, "BoundingRectangle", true);
    check_fields::<super::super::Box>(&inventory, "Box", false);
    check_fields::<super::super::BoxDimensions>(&inventory, "BoxDimensions", true);
    check_fields::<super::super::CheckerboardMaterial>(&inventory, "CheckerboardMaterial", false);
    check_fields::<super::super::ClassificationType>(&inventory, "ClassificationType", true);
    check_fields::<super::super::Clock>(&inventory, "Clock", false);
    check_fields::<super::super::Color>(&inventory, "Color", true);
    check_fields::<super::super::ColorBlendMode>(&inventory, "ColorBlendMode", true);
    check_fields::<super::super::CornerType>(&inventory, "CornerType", true);
    check_fields::<super::super::Corridor>(&inventory, "Corridor", false);
    check_fields::<super::super::CustomProperty>(&inventory, "CustomProperty", true);
    check_fields::<super::super::Cylinder>(&inventory, "Cylinder", false);
    check_fields::<super::super::DistanceDisplayCondition>(&inventory, "DistanceDisplayCondition", true);
    check_fields::<super::super::Double>(&inventory, "Double", true);
    check_fields::<super::super::DoubleList>(&inventory, "DoubleList", true);
    check_fields::<super::super::Ellipse>(&inventory, "Ellipse", false);
    check_fields::<super::super::Ellipsoid>(&inventory, "Ellipsoid", false);
    check_fields::<super::super::EllipsoidRadii>(&inventory, "EllipsoidRadii", true);
    check_fields::<super::super::EyeOffset>(&inventory, "EyeOffset", true);
    check_fields::<super::super::Font>(&inventory, "Font", true);
    check_fields::<super::super::GridMaterial>(&inventory, "GridMaterial", false);
    check_fields::<super::super::HeightReference>(&inventory, "HeightReference", true);
    check_fields::<super::super::HorizontalOrigin>(&inventory, "HorizontalOrigin", true);
    check_fields::<super::super::ImageMaterial>(&inventory, "ImageMaterial", false);
    check_fields::<super::super::Integer>(&inventory, "Integer", true);
    check_fields::<super::super::Label>(&inventory, "Label", false);
    check_fields::<super::super::LabelStyle>(&inventory, "LabelStyle", true);
    check_fields::<super::super::LineCount>(&inventory, "LineCount", true);
    check_fields::<super::super::LineOffset>(&inventory, "LineOffset", true);
    check_fields::<super::super::LineThickness>(&inventory, "LineThickness", true);
    check_fields::<super::super::Material>(&inventory, "Material", false);
    check_fields::<super::super::Model>(&inventory, "Model", false);
    check_fields::<super::super::NearFarScalar>(&inventory, "NearFarScalar", true);
    check_fields::<super::super::NodeTransformation>(&inventory, "NodeTransformation", false);
    check_fields::<super::super::Orientation>(&inventory, "Orientation", true);
    check_fields::<super::super::Packet>(&inventory, "Packet", false);
    check_fields::<super::super::Path>(&inventory, "Path", false);
    check_fields::<super::super::PathMode>(&inventory, "PathMode", true);
    check_fields::<super::super::PixelOffset>(&inventory, "PixelOffset", true);
    check_fields::<super::super::Point>(&inventory, "Point", false);
    check_fields::<super::super::Polygon>(&inventory, "Polygon", false);
    check_fields::<super::super::Polyline>(&inventory, "Polyline", false);
    check_fields::<super::super::PolylineArrowMaterial>(&inventory, "PolylineArrowMaterial", false);
    check_fields::<super::super::PolylineDashMaterial>(&inventory, "PolylineDashMaterial", false);
    check_fields::<super::super::PolylineGlowMaterial>(&inventory, "PolylineGlowMaterial", false);
    check_fields::<super::super::PolylineMaterial>(&inventory, "PolylineMaterial", false);
    check_fields::<super::super::PolylineOutlineMaterial>(&inventory, "PolylineOutlineMaterial", false);
    check_fields::<super::super::PolylineVolume>(&inventory, "PolylineVolume", false);
    check_fields::<super::super::Position>(&inventory, "Position", true);
    check_fields::<super::super::PositionList>(&inventory, "PositionList", true);
    check_fields::<super::super::PositionListOfLists>(&inventory, "PositionListOfLists", true);
    check_fields::<super::super::Rectangle>(&inventory, "Rectangle", false);
    check_fields::<super::super::RectangleCoordinates>(&inventory, "RectangleCoordinates", true);
    check_fields::<super::super::Repeat>(&inventory, "Repeat", true);
    check_fields::<super::super::Rotation>(&inventory, "Rotation", true);
    check_fields::<super::super::Scale>(&inventory, "Scale", true);
    check_fields::<super::super::ShadowMode>(&inventory, "ShadowMode", true);
    check_fields::<super::super::Shape>(&inventory, "Shape", true);
    check_fields::<super::super::SolidColorMaterial>(&inventory, "SolidColorMaterial", false);
    check_fields::<super::super::String>(&inventory, "String", true);
    check_fields::<super::super::StripeMaterial>(&inventory, "StripeMaterial", false);
    check_fields::<super::super::StripeOrientation>(&inventory, "StripeOrientation", true);
    check_fields::<super::super::Tileset>(&inventory, "Tileset", false);
    check_fields::<super::super::Translation>(&inventory, "Translation", true);
    check_fields::<super::super::Uri>(&inventory, "Uri", true);
    check_fields::<super::super::VerticalOrigin>(&inventory, "VerticalOrigin", true);
    check_fields::<super::super::ViewFrom>(&inventory, "ViewFrom", true);
    check_fields::<super::super::Wall>(&inventory, "Wall", false);
}
