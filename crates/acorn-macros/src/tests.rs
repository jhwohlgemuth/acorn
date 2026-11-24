#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
use super::*;
use crate::command::{parse_cmd_words, CmdWord};
use syn::{parse_quote, DeriveInput, Type};

#[test]
fn test_cmd_words_keeps_empty_quoted_arguments() {
    let words = parse_cmd_words("echo '' \"\"").expect("empty quoted arguments should parse");
    assert!(matches!(words.first(), Some(CmdWord::Literal(value)) if value == "echo"));
    assert!(matches!(words.get(1), Some(CmdWord::Literal(value)) if value.is_empty()));
    assert!(matches!(words.get(2), Some(CmdWord::Literal(value)) if value.is_empty()));
}
#[test]
fn test_cmd_words_keeps_single_quoted_splat_literal() {
    let words = parse_cmd_words("echo '{args...}'").expect("single quoted splat should parse");
    assert!(matches!(words.first(), Some(CmdWord::Literal(value)) if value == "echo"));
    assert!(matches!(words.get(1), Some(CmdWord::Literal(value)) if value == "{args...}"));
}
#[test]
fn test_cmd_words_mixed_var_and_splat() {
    let words = parse_cmd_words("git {cmd} {args...}").expect("mixed should parse");
    assert!(matches!(words.first(), Some(CmdWord::Literal(value)) if value == "git"));
    assert!(matches!(words.get(1), Some(CmdWord::Var(value)) if value == "cmd"));
    assert!(matches!(words.get(2), Some(CmdWord::Splat(value)) if value == "args"));
}
#[test]
fn test_cmd_words_rejects_double_quoted_splat() {
    let error = parse_cmd_words("echo \"{args...}\"").expect_err("double quoted splat should fail");
    assert!(error.contains("interpolation must occupy an unquoted shell word"));
}
#[test]
fn test_cmd_words_var_must_occupy_full_word() {
    let error = parse_cmd_words("echo{name}").expect_err("joined var should fail");
    assert!(error.contains("interpolation must occupy a full shell word"));
}
#[test]
fn test_cmd_words_var_parses_non_splat() {
    let words = parse_cmd_words("echo {name}").expect("var should parse");
    assert!(matches!(words.first(), Some(CmdWord::Literal(value)) if value == "echo"));
    assert!(matches!(words.get(1), Some(CmdWord::Var(value)) if value == "name"));
}
#[test]
fn test_cmd_words_var_rejects_double_quoted() {
    let error = parse_cmd_words("echo \"{var}\"").expect_err("double quoted var should fail");
    assert!(error.contains("interpolation must occupy an unquoted shell word"));
}
#[test]
fn test_cmd_words_var_rejects_non_ident() {
    let words = parse_cmd_words("echo {123}").expect("non-ident should be literal");
    assert!(matches!(words.first(), Some(CmdWord::Literal(value)) if value == "echo"));
    assert!(matches!(words.get(1), Some(CmdWord::Literal(value)) if value == "{123}"));
}
#[test]
fn test_expand_database_row_generates_expected_tokens() {
    let input: DeriveInput = parse_quote! {
        #[row(order_by = "executed_at DESC", table = catalog)]
        struct ActivityRow {
            id: Option<i64>,
            command: Option<String>,
            executed_at: Option<Timestamp>,
            success: Option<bool>,
        }
    };
    let tokens = expand_database_row(&input).expect("derive expansion should succeed").to_string();
    assert!(tokens.contains("pub const TABLE_NAME"));
    assert!(tokens.contains("catalog"));
    assert!(tokens.contains("pub const ORDER_BY"));
    assert!(tokens.contains("executed_at DESC"));
    assert!(tokens.contains("pub const FIELDS"));
    assert!(tokens.contains("\"id\"") && tokens.contains("\"command\"") && tokens.contains("\"executed_at\"") && tokens.contains("\"success\""));
    assert!(tokens.contains("executed_at => acorn_core :: util :: to_rfc3339"));
    assert!(tokens.contains("success => | value | i32 :: from (value)"));
}
#[test]
fn test_expand_database_row_infers_table_name_when_not_provided() {
    let input: DeriveInput = parse_quote! {
        struct CanonicalRecordRow {
            id: Option<i64>,
        }
    };
    let tokens = expand_database_row(&input).expect("derive expansion should succeed").to_string();
    assert!(tokens.contains("pub const TABLE_NAME"));
    assert!(tokens.contains("canonical_record"));
}
#[test]
fn test_expand_database_row_rejects_enum() {
    let input: DeriveInput = parse_quote! {
        enum NotRow {
            A,
        }
    };
    let error = expand_database_row(&input).expect_err("enum should fail");
    assert!(error.to_string().contains("only works with structs"));
}
#[test]
fn test_expand_database_row_rejects_tuple_struct() {
    let input: DeriveInput = parse_quote! {
        struct TupleRow(Option<i64>);
    };
    let error = expand_database_row(&input).expect_err("tuple struct should fail");
    assert!(error.to_string().contains("named fields"));
}
#[test]
fn test_expand_database_row_uses_explicit_table_name_string() {
    let input: DeriveInput = parse_quote! {
        #[row(table = "validation_history")]
        struct ValidationRow {
            id: Option<i64>,
        }
    };
    let tokens = expand_database_row(&input).expect("derive expansion should succeed").to_string();
    assert!(tokens.contains("pub const TABLE_NAME"));
    assert!(tokens.contains("validation_history"));
}
#[test]
fn test_expand_validate_uses_only_portable_paths() {
    let input: DeriveInput = parse_quote! {
        struct Portable {
            #[validate(length(min = 1), each(email), async(function = "validate_value"))]
            values: Vec<String>,
        }
    };
    let tokens = expand_validate(&input).expect("validation derive should expand").to_string();
    assert!(!tokens.contains(":: std"));
    assert!(!tokens.contains("tokio"));
    assert!(!tokens.contains("async_trait"));
    assert!(!tokens.contains("Box :: pin"));
    assert!(tokens.contains(":: acorn_core :: validation"));
    assert!(tokens.contains("validate_async"));
    assert!(tokens.contains("validate_value") && tokens.contains("await"));
}
#[test]
fn test_extract_order_by_with_value() {
    let attrs = vec![parse_quote!(#[row(order_by = "checked_at DESC")])];
    assert_eq!(extract_row_config(&attrs).unwrap().order_by, Some("checked_at DESC".to_string()));
}
#[test]
fn test_extract_order_by_without_value() {
    let attrs = vec![parse_quote!(#[row(other = "value")])];
    assert_eq!(extract_row_config(&attrs).unwrap().order_by, None);
}
#[test]
fn test_extract_table_with_identifier() {
    let attrs = vec![parse_quote!(#[row(table = catalog)])];
    assert_eq!(extract_row_config(&attrs).unwrap().table, Some("catalog".to_string()));
}
#[test]
fn test_extract_table_with_string() {
    let attrs = vec![parse_quote!(#[row(table = "catalog")])];
    assert_eq!(extract_row_config(&attrs).unwrap().table, Some("catalog".to_string()));
}
#[test]
fn test_infer_table_name_from_struct_name() {
    let ident: syn::Ident = parse_quote!(CatalogRow);
    assert_eq!(infer_table_name(&ident), "catalog");
}
#[test]
fn test_option_bool_detection() {
    let bool_ty: Type = parse_quote!(Option<bool>);
    let string_ty: Type = parse_quote!(Option<String>);
    assert!(is_option_bool(&bool_ty));
    assert!(!is_option_bool(&string_ty));
}
#[test]
fn test_option_timestamp_detection() {
    let timestamp_ty: Type = parse_quote!(Option<Timestamp>);
    let string_ty: Type = parse_quote!(Option<String>);
    assert!(is_option_timestamp(&timestamp_ty));
    assert!(!is_option_timestamp(&string_ty));
}

mod script_tests {
    #[cfg(feature = "js-lint")]
    use crate::script::validate_javascript;
    use crate::script::{parse_js, validate_authored_script, Part};

    #[test]
    fn test_authored_script_uses_one_value_argument() {
        assert!(validate_authored_script("const [name] = __acornArgs; window.good(name)").is_ok());
        assert!(validate_authored_script("window.good({name})").is_err());
    }
    #[test]
    fn test_division_bindings_and_regex_bindings() {
        for script in [
            "return 10 / {divisor}",
            "return amount/{divisor}",
            "return call() / {divisor}",
            "return [10][0] / {divisor}",
            "return /a/.test('a') / {divisor}",
            "const amount = {} / {divisor}",
        ] {
            assert!(parse_js(script).is_ok(), "{script}");
        }
        for script in ["return /{divisor}/", "const pattern = /{divisor}/", "if (ready) /{divisor}/.test(input)"] {
            assert!(parse_js(script).is_err(), "{script}");
        }
    }
    #[cfg(feature = "js-lint")]
    #[test]
    fn test_javascript_parser_rejects_invalid_body() {
        assert!(validate_javascript("await Promise.resolve(); return null;").is_ok());
        assert!(validate_javascript("const = ;").is_err());
    }
    #[test]
    fn test_placeholder_is_a_value() {
        assert_eq!(
            parse_js("window.foo({value});"),
            Ok(vec![
                Part::Literal("window.foo(".into()),
                Part::Binding("value".into()),
                Part::Literal(");".into())
            ])
        );
    }
    #[test]
    fn test_placeholder_rejects_identifier_fragments() {
        for script in ["foo{value}", "{value}bar", "window.{value}"] {
            assert!(parse_js(script).is_err(), "{script}");
        }
    }
    #[test]
    fn test_placeholder_rejects_quoted_contexts() {
        for script in ["'{value}'", "\"{value}\"", "`{value}`", "// {value}\n", "/* {value} */", "/{value}/"] {
            assert!(parse_js(script).is_err(), "{script}");
        }
    }
}
