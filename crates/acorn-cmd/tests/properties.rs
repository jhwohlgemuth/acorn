//! Generated command rendering and parsing laws.
use acorn_cmd::{command_string, parse_sh};
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};
use std::ffi::OsString;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROPTEST_CASES").ok().and_then(|value| value.parse().ok()).unwrap_or(256),
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/properties.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn test_command_string_parse_sh_round_trips_generated_arguments(
        program in "[A-Za-z0-9]{1,16}",
        arguments in prop::collection::vec(prop_oneof![
            any::<String>(),
            Just(String::new()),
            Just("a 'quote' \"double\" \\ slash".to_string()),
            Just("𝄞\r\nnext".to_string()),
        ], 0..12),
    ) {
        let os_arguments = arguments.iter().map(OsString::from).collect::<Vec<_>>();
        let rendered = command_string(&program, &os_arguments);
        let (parsed_program, parsed_arguments) = parse_sh(&rendered).expect("rendered command parses");
        prop_assert_eq!(parsed_program, program);
        prop_assert_eq!(parsed_arguments, arguments);
    }
}
