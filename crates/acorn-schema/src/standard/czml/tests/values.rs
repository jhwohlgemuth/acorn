use super::super::{
    Cartesian2ListValue, Cartesian2Value, Cartesian3ListOfListsValue, Cartesian3ListValue, Cartesian3Value, Cartesian3VelocityValue, DoubleListValue,
    DoubleValue, FixedValue, IntegerValue, ReferenceListOfListsValue, ReferenceListValue, RgbaValue, RgbafValue, SampleTime, Time,
    UnitQuaternionValue,
};

fn check_fixed_arity<const N: usize>() {
    let constant = FixedValue::<N>::Constant([1.25; N]);
    let constant_json = serde_json::to_string(&constant).expect("serialize constant");
    assert_eq!(serde_json::from_str::<FixedValue<N>>(&constant_json).expect("decode constant"), constant);

    let iso = FixedValue::<N>::Samples(vec![(SampleTime::Iso(Time("2012-04-30T12:00:00Z".into())), [2.5; N])]);
    let iso_json = serde_json::to_string(&iso).expect("serialize ISO sample");
    assert_eq!(serde_json::from_str::<FixedValue<N>>(&iso_json).expect("decode ISO sample"), iso);

    let epoch = FixedValue::<N>::Samples(vec![(SampleTime::Seconds(0.0), [3.75; N]), (SampleTime::Seconds(60.0), [4.5; N])]);
    let epoch_json = serde_json::to_string(&epoch).expect("serialize epoch samples");
    assert_eq!(serde_json::from_str::<FixedValue<N>>(&epoch_json).expect("decode epoch samples"), epoch);

    let wrong_length = serde_json::to_string(&vec![1.0; N.saturating_add(2)]).expect("serialize invalid value");
    assert!(serde_json::from_str::<FixedValue<N>>(&wrong_length).is_err());
}
#[test]
fn test_eserde_compat_retains_value_field_path() {
    #[derive(Debug, eserde::Deserialize)]
    struct Parent {
        #[eserde(compat)]
        points: Cartesian3ListValue,
    }
    let error = eserde::json::from_str::<Parent>(r#"{"points":[1,2]}"#).expect_err("ragged list");
    assert!(error.to_string().contains("points"), "{error}");
}
#[test]
fn test_fixed_values_accept_constant_and_sampled_arrays() {
    let constant: Cartesian3Value = serde_json::from_str("[1,2,3]").expect("constant Cartesian3");
    assert_eq!(constant, FixedValue::Constant([1.0, 2.0, 3.0]));
    let iso: Cartesian2Value = serde_json::from_str("[\"2012-04-30T12:00:00Z\",1,2]").expect("ISO sample");
    assert!(matches!(iso, FixedValue::Samples(samples) if samples.len() == 1));
    let epoch: RgbafValue = serde_json::from_str("[0,1,0,0,1,60,0,1,0,1]").expect("epoch samples");
    assert!(matches!(epoch, FixedValue::Samples(samples) if samples.len() == 2));
}
#[test]
fn test_fixed_values_cover_every_component_arity() {
    check_fixed_arity::<2>();
    check_fixed_arity::<3>();
    check_fixed_arity::<4>();
    check_fixed_arity::<6>();
}
#[test]
fn test_fixed_values_reject_wrong_arity() {
    for source in ["[]", "[1]", "[1,2,3,4,5]"] {
        let error = serde_json::from_str::<Cartesian3Value>(source).expect_err("invalid Cartesian3 length");
        assert!(error.to_string().contains("3 constant components or samples of 4 items"));
    }
    assert!(serde_json::from_str::<UnitQuaternionValue>("[0,1,2,3,4,5]").is_err());
    assert!(serde_json::from_str::<Cartesian3VelocityValue>("[1,2,3]").is_err());
}
#[test]
fn test_fixed_values_round_trip_numeric_samples() {
    let source = "[0,1.25,2.5,3.75,60,4.25,5.5,6.75]";
    let value: Cartesian3Value = serde_json::from_str(source).expect("two numeric samples");
    let encoded = serde_json::to_string(&value).expect("serialize samples");
    let decoded: Cartesian3Value = serde_json::from_str(&encoded).expect("decode samples");
    assert_eq!(decoded, value);
}
#[test]
fn test_flat_lists_accept_empty_and_reject_ragged_arrays() {
    let empty: Cartesian2ListValue = serde_json::from_str("[]").expect("empty flat list");
    assert!(empty.0.is_empty());
    let triples: Cartesian3ListValue = serde_json::from_str("[1,2,3,4,5,6]").expect("two Cartesian points");
    assert_eq!(triples.0, vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    assert_eq!(
        serde_json::to_string(&triples).expect("flat list serialization"),
        "[1.0,2.0,3.0,4.0,5.0,6.0]"
    );
    assert!(serde_json::from_str::<Cartesian3ListValue>("[1,2,3,4]").is_err());
    assert!(serde_json::from_str::<Cartesian2ListValue>("[1]").is_err());
}
#[test]
fn test_integer_rgba_checks_components_and_preserves_integers() {
    for source in ["[255,0,128,255]", "[0,255,0,128,255,60,0,128,255,255]"] {
        let value: RgbaValue = serde_json::from_str(source).expect("integer RGBA");
        let encoded = serde_json::to_string(&value).expect("serialize integer RGBA");
        assert!(!encoded.contains("255.0"));
        assert_eq!(serde_json::from_str::<RgbaValue>(&encoded).expect("decode integer RGBA"), value);
    }
    for source in ["[256,0,0,0]", "[-1,0,0,0]", "[1.5,0,0,0]", "[0,0,0]", "[0,0,0,0,0,60,256,0,0,0]"] {
        assert!(serde_json::from_str::<RgbaValue>(source).is_err(), "accepted {source}");
    }
}
#[test]
fn test_nested_and_reference_lists_round_trip() {
    let nested: Cartesian3ListOfListsValue = serde_json::from_str("[[1,2,3],[],[4,5,6,7,8,9]]").expect("nested Cartesian points");
    let encoded = serde_json::to_string(&nested).expect("nested list serialization");
    assert_eq!(
        serde_json::from_str::<Cartesian3ListOfListsValue>(&encoded).expect("nested list decoding"),
        nested
    );
    assert!(serde_json::from_str::<Cartesian3ListOfListsValue>("[[1,2]]").is_err());

    let references: ReferenceListValue = serde_json::from_str("[\"a#position\",\"b#position\"]").expect("reference list");
    assert_eq!(references.len(), 2);
    let nested_references: ReferenceListOfListsValue = serde_json::from_str("[[\"a#position\"],[]]").expect("nested references");
    assert_eq!(
        serde_json::to_string(&nested_references).expect("serialize references"),
        "[[\"a#position\"],[]]"
    );
    let doubles: DoubleListValue = serde_json::from_str("[1,2.5]").expect("double list");
    assert_eq!(doubles, vec![1.0, 2.5]);
}
#[test]
fn test_scalar_values_cover_shorthand_constant_and_samples() {
    for source in ["1.25", "[1.25]", "[0,1.25,60,2.5]", "[\"2012-04-30T12:00:00Z\",1.25]"] {
        let value: DoubleValue = serde_json::from_str(source).expect("double value");
        let encoded = serde_json::to_string(&value).expect("serialize double");
        assert_eq!(serde_json::from_str::<DoubleValue>(&encoded).expect("decode double"), value);
    }
    for source in ["-1", "[-1]", "[0,-1,60,2]", "[\"2012-04-30T12:00:00Z\",1]"] {
        let value: IntegerValue = serde_json::from_str(source).expect("integer value");
        let encoded = serde_json::to_string(&value).expect("serialize integer");
        assert_eq!(serde_json::from_str::<IntegerValue>(&encoded).expect("decode integer"), value);
    }
    for source in ["1.5", "[1.5]", "[0,1.5]"] {
        assert!(serde_json::from_str::<IntegerValue>(source).is_err());
    }
}
