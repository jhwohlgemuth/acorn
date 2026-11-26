use super::super::{
    ArcTypeValue, ClassificationTypeValue, ClockRange, ClockStep, ColorBlendModeValue, CornerTypeValue, ExtrapolationType, HeightReferenceValue,
    HorizontalOriginValue, InterpolationAlgorithm, LabelStyleValue, PathModeValue, ReferenceFrame, ShadowModeValue, StripeOrientationValue,
    VerticalOriginValue,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

fn assert_constants<T: DeserializeOwned + Serialize>(constants: &[&str]) {
    for constant in constants {
        let encoded = format!("\"{constant}\"");
        let decoded: T = serde_json::from_str(&encoded).expect("schema constant must deserialize");
        assert_eq!(serde_json::to_string(&decoded).expect("serialize enum"), encoded);
    }
    assert!(serde_json::from_str::<T>("\"NOT_A_CZML_CONSTANT\"").is_err());
}
#[test]
fn test_all_pinned_enum_constants_round_trip() {
    assert_constants::<ArcTypeValue>(&["NONE", "GEODESIC", "RHUMB"]);
    assert_constants::<ClassificationTypeValue>(&["TERRAIN", "CESIUM_3D_TILE", "BOTH"]);
    assert_constants::<ClockRange>(&["UNBOUNDED", "CLAMPED", "LOOP_STOP"]);
    assert_constants::<ClockStep>(&["TICK_DEPENDENT", "SYSTEM_CLOCK_MULTIPLIER", "SYSTEM_CLOCK"]);
    assert_constants::<ColorBlendModeValue>(&["HIGHLIGHT", "REPLACE", "MIX"]);
    assert_constants::<CornerTypeValue>(&["ROUNDED", "MITERED", "BEVELED"]);
    assert_constants::<HeightReferenceValue>(&["NONE", "CLAMP_TO_GROUND", "RELATIVE_TO_GROUND"]);
    assert_constants::<HorizontalOriginValue>(&["LEFT", "CENTER", "RIGHT"]);
    assert_constants::<LabelStyleValue>(&["FILL", "OUTLINE", "FILL_AND_OUTLINE"]);
    assert_constants::<PathModeValue>(&["WHOLE", "PORTIONS"]);
    assert_constants::<ShadowModeValue>(&["DISABLED", "ENABLED", "CAST_ONLY", "RECEIVE_ONLY"]);
    assert_constants::<StripeOrientationValue>(&["HORIZONTAL", "VERTICAL"]);
    assert_constants::<VerticalOriginValue>(&["BASELINE", "BOTTOM", "CENTER", "TOP"]);
    assert_constants::<InterpolationAlgorithm>(&["LINEAR", "LAGRANGE", "HERMITE"]);
    assert_constants::<ExtrapolationType>(&["NONE", "HOLD", "EXTRAPOLATE"]);
    assert_constants::<ReferenceFrame>(&["FIXED", "INERTIAL"]);
}
#[test]
fn test_interpolation_defaults_match_schema() {
    assert_eq!(InterpolationAlgorithm::default(), InterpolationAlgorithm::Linear);
    assert_eq!(ExtrapolationType::default(), ExtrapolationType::None);
    assert_eq!(ReferenceFrame::default(), ReferenceFrame::Fixed);
}
#[test]
fn test_schema_lists_exact_classification_constants() {
    let schema = serde_json::to_value(schemars::schema_for!(ClassificationTypeValue)).expect("serialize classification schema");
    let constants: Vec<_> = schema
        .get("oneOf")
        .and_then(serde_json::Value::as_array)
        .expect("enum alternatives")
        .iter()
        .filter_map(|variant| variant.get("const").and_then(serde_json::Value::as_str))
        .collect();
    assert_eq!(constants, ["TERRAIN", "CESIUM_3D_TILE", "BOTH"]);
}
