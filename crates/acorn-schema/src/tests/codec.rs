use crate::codec::{decode_value, encode_value};
use acorn_core::util::MimeType;
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};
use serde_json::{Map, Number, Value};

fn json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|value| Value::Number(value.into())),
        any::<u64>().prop_map(|value| Value::Number(value.into())),
        any::<f64>().prop_filter_map("finite float", |value| Number::from_f64(value).map(Value::Number)),
        prop_oneof![Just(String::new()), Just("𝄞\r\n".to_string()), any::<String>()].prop_map(Value::String),
    ];
    // The codec accepts JSON trees; bounded recursion keeps whole-document cases practical.
    leaf.prop_recursive(3, 32, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..6).prop_map(Value::Array),
            prop::collection::btree_map(any::<String>(), inner, 0..6).prop_map(|fields| Value::Object(fields.into_iter().collect::<Map<_, _>>())),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/codec/value.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn test_structured_value_encoding_is_deterministic(value in json_value()) {
        for format in [MimeType::Json, MimeType::Jsonc, MimeType::Yaml] {
            let first = encode_value(&value, &format).expect("generated JSON value encodes");
            let second = encode_value(&value, &format).expect("generated JSON value encodes again");
            prop_assert_eq!(first, second, "format: {:?}", format);
        }
    }
    #[test]
    fn test_structured_value_round_trips(value in json_value()) {
        for format in [MimeType::Json, MimeType::Jsonc, MimeType::Yaml] {
            let encoded = encode_value(&value, &format).expect("generated JSON value encodes");
            let decoded = decode_value(&encoded, &format).expect("encoded value decodes");
            prop_assert_eq!(&decoded, &value, "format: {:?}", format);
        }
    }
}
#[test]
fn test_unsupported_structured_formats_return_diagnostics() {
    for format in [
        MimeType::Cff,
        MimeType::Csv,
        MimeType::Doc,
        MimeType::Docx,
        MimeType::Epub,
        MimeType::Excel,
        MimeType::Gguf,
        MimeType::Gzip,
        MimeType::Jpeg,
        MimeType::JsonLines,
        MimeType::LdJson,
        MimeType::Markdown,
        MimeType::ModelCard,
        MimeType::Odp,
        MimeType::Ods,
        MimeType::Odt,
        MimeType::Onnx,
        MimeType::Otf,
        MimeType::Parquet,
        MimeType::Pdf,
        MimeType::Png,
        MimeType::Powerpoint,
        MimeType::Ppt,
        MimeType::Prompt,
        MimeType::Pytorch,
        MimeType::RdfXml,
        MimeType::Rhai,
        MimeType::Ron,
        MimeType::Rtf,
        MimeType::Rust,
        MimeType::Safetensors,
        MimeType::Sbom,
        MimeType::SevenZip,
        MimeType::Svg,
        MimeType::Tar,
        MimeType::Text,
        MimeType::Toml,
        MimeType::Ttf,
        MimeType::Turtle,
        MimeType::Webp,
        MimeType::Zip,
    ] {
        assert_eq!(
            encode_value(&Value::Null, &format).expect_err("unsupported encoding").code,
            "unsupported-format"
        );
        assert_eq!(
            decode_value("null", &format).expect_err("unsupported decoding").code,
            "unsupported-format"
        );
    }
}
