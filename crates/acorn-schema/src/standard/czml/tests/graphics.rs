use super::super::{Billboard, Material, Model, Polygon, Wall};
use crate::validation::Validate;
use serde_json::{json, Value};

fn round_trip<T>(source: &Value) -> Value
where
    T: for<'de> serde::Deserialize<'de> + serde::Serialize + Validate,
{
    let decoded: T = serde_json::from_value(source.clone()).expect("decode graphic");
    assert!(decoded.validate().is_ok());
    let encoded = serde_json::to_value(decoded).expect("encode graphic");
    let canonical: T = serde_json::from_value(encoded.clone()).expect("decode canonical graphic");
    assert_eq!(serde_json::to_value(canonical).expect("re-encode graphic"), encoded);
    encoded
}

#[test]
fn test_billboard_and_label_examples_round_trip() {
    let billboard = json!({"image":"image.png","pixelOffset":{"cartesian2":[1.0,2.0]},"alignedAxis":{"unitCartesian":[1.0,0.0,0.0]},"show":true});
    let decoded: Billboard = serde_json::from_value(billboard.clone()).expect("billboard");
    assert!(decoded.validate().is_ok());
    assert_eq!(serde_json::to_value(decoded).expect("serialize billboard"), billboard);
    let label = json!({"text":"Research site","font":"12px sans-serif","fillColor":{"rgba":[255,255,255,255]},"show":true});
    let decoded: super::super::Label = serde_json::from_value(label.clone()).expect("label");
    assert!(decoded.validate().is_ok());
    assert_eq!(serde_json::to_value(decoded).expect("serialize label"), label);
}
#[test]
fn test_material_rejects_two_kinds() {
    let material: Material =
        serde_json::from_value(json!({"solidColor":{"color":{"rgba":[255,0,0,255]}},"grid":{"color":{"rgba":[0,0,0,255]}}})).expect("material");
    assert!(material.validate().is_err());
}
#[test]
fn test_model_preserves_node_order_and_validates_articulation_keys() {
    let source = r#"{"gltf":"model.glb","nodeTransformations":{"tail":{"translation":{"cartesian":[1,2,3]}},"head":{"scale":{"cartesian":[1,1,1]}}},"articulations":{"wing deploy":1.0}}"#;
    let model: Model = serde_json::from_str(source).expect("model with ordered maps");
    assert_eq!(
        model
            .node_transformations
            .as_ref()
            .and_then(|entries| entries.0.first())
            .map(|entry| entry.0.as_str()),
        Some("tail")
    );
    assert!(model.validate().is_ok());
    let encoded = serde_json::to_string(&model).expect("serialize model");
    assert!(encoded.find("tail").expect("tail key") < encoded.find("head").expect("head key"));
    let invalid: Model = serde_json::from_str(r#"{"articulations":{"wing":1.0}}"#).expect("model with invalid articulation key");
    assert!(invalid.validate().is_err());
}
#[test]
fn test_surface_and_volume_graphics_round_trip() {
    let samples = [
        ("Box", json!({"dimensions":{"cartesian":[1,2,3]},"show":true})),
        ("Corridor", json!({"positions":{"cartesian":[1,2,3,4,5,6]},"width":2})),
        ("Cylinder", json!({"length":10,"topRadius":2,"bottomRadius":3})),
        ("Ellipse", json!({"semiMajorAxis":5,"semiMinorAxis":3})),
        ("Ellipsoid", json!({"radii":{"cartesian":[1,2,3]}})),
        (
            "Polygon",
            json!({"positions":{"cartesian":[1,2,3,4,5,6,7,8,9]},"holes":{"cartesian":[[1,2,3,4,5,6,7,8,9]]}}),
        ),
        ("Polyline", json!({"positions":{"cartesian":[1,2,3,4,5,6]},"width":2})),
        (
            "PolylineVolume",
            json!({"positions":{"cartesian":[1,2,3,4,5,6]},"shape":{"cartesian2":[0,0,1,0,1,1]}}),
        ),
        ("Rectangle", json!({"coordinates":{"wsenDegrees":[-80,30,-70,40]}})),
        (
            "Wall",
            json!({"positions":{"cartesian":[1,2,3,4,5,6]},"minimumHeights":{"array":[0,0]},"maximumHeights":{"array":[10,10]}}),
        ),
    ];
    for (name, source) in samples {
        let encoded = match name {
            | "Box" => round_trip::<super::super::Box>(&source),
            | "Corridor" => round_trip::<super::super::Corridor>(&source),
            | "Cylinder" => round_trip::<super::super::Cylinder>(&source),
            | "Ellipse" => round_trip::<super::super::Ellipse>(&source),
            | "Ellipsoid" => round_trip::<super::super::Ellipsoid>(&source),
            | "Polygon" => round_trip::<super::super::Polygon>(&source),
            | "Polyline" => round_trip::<super::super::Polyline>(&source),
            | "PolylineVolume" => round_trip::<super::super::PolylineVolume>(&source),
            | "Rectangle" => round_trip::<super::super::Rectangle>(&source),
            | _ => round_trip::<super::super::Wall>(&source),
        };
        assert_eq!(
            encoded.as_object().map(serde_json::Map::len),
            source.as_object().map(serde_json::Map::len),
            "{name}"
        );
    }
    let polygon: Polygon = serde_json::from_value(json!({"holes":{"cartesian":[[1,2,3,4,5,6]]}})).expect("polygon holes");
    assert!(polygon.validate().is_ok());
    let wall: Wall = serde_json::from_value(json!({"minimumHeights":{"array":[0,0]},"maximumHeights":{"array":[10,10]}})).expect("wall heights");
    assert!(wall.validate().is_ok());
}
#[test]
fn test_unknown_graphics_field_is_rejected() {
    assert!(serde_json::from_str::<Billboard>(r#"{"image":"image.png","unlisted":true}"#).is_err());
}
