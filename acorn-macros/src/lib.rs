//! Procedural macros for ACORN database utilities
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr, Meta};

/// Generates row metadata and query helpers for database row structs.
///
/// Attributes:
/// - `#[row(order_by = "...")]` — optional ORDER BY clause
/// - `#[row(table = ...)]` — optional table name (string or identifier)
///
/// This derive exposes constants `ORDER_BY` and `FIELDS`, and helper methods
/// for generating dynamic WHERE/ORDER BY SQL.
///
/// Example:
/// ```ignore
/// #[derive(DatabaseRow)]
/// #[row(order_by = "executed_at DESC")]
/// pub struct ActivityRow { ... }
///
/// // Use in select method:
/// let (query, params) = self.build_select_query(base_query);
/// ```
///
/// You can also access the generated `FIELDS` constant to get a list of all field names:
/// ```ignore
/// #[derive(DatabaseRow)]
/// #[row(table = catalog)]
/// pub struct UserRow {
///     pub id: i32,
///     pub name: String,
///     pub email: String,
///     pub active: bool,
/// }
///
/// // Access the generated FIELDS constant
/// assert_eq!(UserRow::FIELDS, &["id", "name", "email", "active"]);
/// ```
#[proc_macro_derive(DatabaseRow, attributes(row))]
pub fn derive_database_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_database_row(&input).unwrap_or_else(|why| why.to_compile_error()).into()
}
/// Parses a literal `cmd!("...")` command into shell words with `{var}` and `{args...}` interpolation.
#[proc_macro]
pub fn cmd_sh_words(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    expand_cmd_sh_words(&input).unwrap_or_else(|why| why.to_compile_error()).into()
}
struct RowConfig {
    order_by: Option<String>,
    table: Option<String>,
}
#[derive(Debug)]
enum CmdWord {
    Literal(String),
    Var(String),
    Splat(String),
}
fn expand_cmd_sh_words(input: &LitStr) -> syn::Result<proc_macro2::TokenStream> {
    let words = parse_cmd_words(&input.value()).map_err(|message| syn::Error::new(input.span(), message))?;
    let parts = words.into_iter().map(|word| match word {
        | CmdWord::Literal(value) => quote! {
            ::std::iter::once(::std::ffi::OsString::from(#value)).collect::<::std::vec::Vec<::std::ffi::OsString>>()
        },
        | CmdWord::Var(name) => {
            let ident = syn::Ident::new(&name, input.span());
            quote! {
                ::std::iter::once(
                    ::std::convert::AsRef::<::std::ffi::OsStr>::as_ref(&#ident).to_os_string()
                ).collect::<::std::vec::Vec<::std::ffi::OsString>>()
            }
        }
        | CmdWord::Splat(name) => {
            let ident = syn::Ident::new(&name, input.span());
            quote! {
                (&#ident).into_iter().map(|__x| {
                    ::std::convert::AsRef::<::std::ffi::OsStr>::as_ref(&__x).to_os_string()
                }).collect::<::std::vec::Vec<::std::ffi::OsString>>()
            }
        }
    });
    Ok(quote! {
        [#(#parts),*].concat()
    })
}
fn parse_cmd_words(command: &str) -> Result<Vec<CmdWord>, String> {
    let chars = command.chars().collect::<Vec<_>>();
    let mut words = Vec::new();
    let mut current = String::new();
    let mut index = 0;
    while index < chars.len() {
        match chars[index] {
            | ch if ch.is_whitespace() => {
                push_literal(&mut words, &mut current);
                index += 1;
            }
            | '\'' => {
                index += 1;
                while index < chars.len() && chars[index] != '\'' {
                    current.push(chars[index]);
                    index += 1;
                }
                if index == chars.len() {
                    return Err("cmd!(sh ...): unterminated single quote".to_string());
                }
                index += 1;
            }
            | '"' => {
                index += 1;
                while index < chars.len() && chars[index] != '"' {
                    if chars[index] == '{' && read_interpolation(&chars, index).is_some() {
                        return Err("cmd!(...): interpolation must occupy an unquoted shell word".to_string());
                    }
                    if chars[index] == '\\' {
                        index += 1;
                        if index == chars.len() {
                            return Err("cmd!(...): trailing escape in double quote".to_string());
                        }
                    }
                    current.push(chars[index]);
                    index += 1;
                }
                if index == chars.len() {
                    return Err("cmd!(...): unterminated double quote".to_string());
                }
                index += 1;
            }
            | '\\' => {
                index += 1;
                if index == chars.len() {
                    return Err("cmd!(...): trailing escape".to_string());
                }
                current.push(chars[index]);
                index += 1;
            }
            | '{' => match read_interpolation(&chars, index) {
                | Some((name, false, next)) if current.is_empty() && next_is_word_boundary(&chars, next) => {
                    words.push(CmdWord::Var(name));
                    index = next;
                }
                | Some((name, true, next)) if current.is_empty() && next_is_word_boundary(&chars, next) => {
                    words.push(CmdWord::Splat(name));
                    index = next;
                }
                | Some(_) => return Err("cmd!(...): interpolation must occupy a full shell word".to_string()),
                | None => {
                    current.push(chars[index]);
                    index += 1;
                }
            },
            | ch => {
                current.push(ch);
                index += 1;
            }
        }
    }
    push_literal(&mut words, &mut current);
    if words.is_empty() {
        Err("cmd!(...): empty command string".to_string())
    } else {
        Ok(words)
    }
}
fn push_literal(words: &mut Vec<CmdWord>, current: &mut String) {
    if !current.is_empty() {
        words.push(CmdWord::Literal(core::mem::take(current)));
    }
}
fn read_interpolation(chars: &[char], start: usize) -> Option<(String, bool, usize)> {
    let close = chars
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, ch)| (*ch == '}').then_some(index))?;
    let content = chars[start + 1..close].iter().collect::<String>();
    if let Some(name) = content.strip_suffix("...") {
        is_ident(name).then(|| (name.to_string(), true, close + 1))
    } else if is_ident(&content) {
        Some((content, false, close + 1))
    } else {
        None
    }
}
fn is_ident(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic()) && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
fn next_is_word_boundary(chars: &[char], index: usize) -> bool {
    chars.get(index).is_none_or(|ch| ch.is_whitespace())
}
fn expand_database_row(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let _name = &input.ident;
    let row_config = extract_row_config(&input.attrs)?;
    let fields = match &input.data {
        | Data::Struct(data) => match &data.fields {
            | Fields::Named(fields) => &fields.named,
            | _ => {
                return Err(syn::Error::new_spanned(input, "DatabaseRow only works with named fields"));
            }
        },
        | _ => {
            return Err(syn::Error::new_spanned(input, "DatabaseRow only works with structs"));
        }
    };
    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap().to_string()).collect();
    let query_fields: Vec<_> = fields
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            if is_option_bool(&field.ty) {
                quote! { #ident => |value| i32::from(value), }
            } else if is_option_datetime(&field.ty) {
                quote! { #ident => |value| value.to_rfc3339(), }
            } else {
                quote! { #ident, }
            }
        })
        .collect();
    let order_by_expr = row_config
        .order_by
        .as_ref()
        .map(|ob| quote! { Some(#ob) })
        .unwrap_or_else(|| quote! { None::<&str> });
    let table_name = row_config.table.unwrap_or_else(|| infer_table_name(_name));
    Ok(quote! {
        impl #_name {
            /// Database table name for this row type.
            pub const TABLE_NAME: &'static str = #table_name;
            /// ORDER BY clause for queries on this row type.
            pub const ORDER_BY: Option<&'static str> = #order_by_expr;
            /// Field list for build_query! macro documentation purposes.
            pub const FIELDS: &'static [&'static str] = &[#(#field_names),*];
            /// Builds a SQL SELECT query with optional WHERE/ORDER BY clauses.
            pub fn build_select_query(
                &self,
                base: &str,
            ) -> crate::io::database::SelectQuery {
                let (query, params) = crate::io::database::macros::build_query!(self, base, Self::ORDER_BY, [#(#query_fields)*]);
                (query, crate::io::database::backend::params_from_iter(params))
            }
        }
        impl crate::io::database::RowMetadata for #_name {
            fn fields() -> &'static [&'static str] {
                Self::FIELDS
            }
        }
    })
}
fn is_option_bool(ty: &syn::Type) -> bool {
    match ty {
        | syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .filter(|segment| segment.ident == "Option")
            .and_then(|segment| match &segment.arguments {
                | syn::PathArguments::AngleBracketed(args) => args.args.first(),
                | _ => None,
            })
            .and_then(|arg| match arg {
                | syn::GenericArgument::Type(syn::Type::Path(inner_path)) => inner_path.path.segments.last(),
                | _ => None,
            })
            .is_some_and(|segment| segment.ident == "bool"),
        | _ => false,
    }
}
fn is_option_datetime(ty: &syn::Type) -> bool {
    match ty {
        | syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .filter(|segment| segment.ident == "Option")
            .and_then(|segment| match &segment.arguments {
                | syn::PathArguments::AngleBracketed(args) => args.args.first(),
                | _ => None,
            })
            .and_then(|arg| match arg {
                | syn::GenericArgument::Type(syn::Type::Path(inner_path)) => inner_path.path.segments.last(),
                | _ => None,
            })
            .is_some_and(|segment| segment.ident == "DateTime"),
        | _ => false,
    }
}
fn extract_row_config(attrs: &[syn::Attribute]) -> syn::Result<RowConfig> {
    let mut order_by = None;
    let mut table = None;
    for attr in attrs {
        if attr.path().is_ident("row") {
            if let Meta::List(meta_list) = &attr.meta {
                let parsed_list =
                    meta_list.parse_args_with(syn::punctuated::Punctuated::<syn::MetaNameValue, syn::token::Comma>::parse_terminated)?;
                for parsed in parsed_list {
                    if parsed.path.is_ident("order_by") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str), ..
                        }) = &parsed.value
                        {
                            order_by = Some(lit_str.value());
                        }
                    }
                    if parsed.path.is_ident("table") {
                        match &parsed.value {
                            | syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str), ..
                            }) => {
                                table = Some(lit_str.value());
                            }
                            | syn::Expr::Path(path_expr) => {
                                if let Some(segment) = path_expr.path.segments.last() {
                                    table = Some(segment.ident.to_string());
                                }
                            }
                            | _ => {}
                        }
                    }
                }
            }
        }
    }
    Ok(RowConfig { order_by, table })
}
fn infer_table_name(name: &syn::Ident) -> String {
    let struct_name = name.to_string();
    let trimmed = struct_name.strip_suffix("Row").unwrap_or(&struct_name);
    let mut output = String::new();
    for (i, c) in trimmed.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                output.push('_');
            }
            output.push(c.to_ascii_lowercase());
        } else {
            output.push(c);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects
    )]
    use super::*;
    use syn::{parse_quote, DeriveInput, Type};

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
    fn test_option_datetime_detection() {
        let datetime_ty: Type = parse_quote!(Option<DateTime<Utc>>);
        let string_ty: Type = parse_quote!(Option<String>);
        assert!(is_option_datetime(&datetime_ty));
        assert!(!is_option_datetime(&string_ty));
    }
    #[test]
    fn test_expand_database_row_generates_expected_tokens() {
        let input: DeriveInput = parse_quote! {
            #[row(order_by = "executed_at DESC", table = catalog)]
            struct ActivityRow {
                id: Option<i64>,
                command: Option<String>,
                executed_at: Option<DateTime<Utc>>,
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
        assert!(tokens.contains("executed_at => | value | value . to_rfc3339 ()"));
        assert!(tokens.contains("success => | value | i32 :: from (value)"));
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
    fn test_expand_database_row_infers_table_name_when_not_provided() {
        let input: DeriveInput = parse_quote! {
            struct ResearchActivityCacheRow {
                id: Option<i64>,
            }
        };
        let tokens = expand_database_row(&input).expect("derive expansion should succeed").to_string();
        assert!(tokens.contains("pub const TABLE_NAME"));
        assert!(tokens.contains("research_activity_cache"));
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
    fn test_cmd_words_rejects_double_quoted_splat() {
        let error = parse_cmd_words("echo \"{args...}\"").expect_err("double quoted splat should fail");
        assert!(error.contains("interpolation must occupy an unquoted shell word"));
    }
    #[test]
    fn test_cmd_words_keeps_single_quoted_splat_literal() {
        let words = parse_cmd_words("echo '{args...}'").expect("single quoted splat should parse");
        assert!(matches!(words.first(), Some(CmdWord::Literal(value)) if value == "echo"));
        assert!(matches!(words.get(1), Some(CmdWord::Literal(value)) if value == "{args...}"));
    }
    #[test]
    fn test_cmd_words_var_parses_non_splat() {
        let words = parse_cmd_words("echo {name}").expect("var should parse");
        assert!(matches!(words.first(), Some(CmdWord::Literal(value)) if value == "echo"));
        assert!(matches!(words.get(1), Some(CmdWord::Var(value)) if value == "name"));
    }
    #[test]
    fn test_cmd_words_var_rejects_non_ident() {
        let words = parse_cmd_words("echo {123}").expect("non-ident should be literal");
        assert!(matches!(words.first(), Some(CmdWord::Literal(value)) if value == "echo"));
        assert!(matches!(words.get(1), Some(CmdWord::Literal(value)) if value == "{123}"));
    }
    #[test]
    fn test_cmd_words_var_must_occupy_full_word() {
        let error = parse_cmd_words("echo{name}").expect_err("joined var should fail");
        assert!(error.contains("interpolation must occupy a full shell word"));
    }
    #[test]
    fn test_cmd_words_var_rejects_double_quoted() {
        let error = parse_cmd_words("echo \"{var}\"").expect_err("double quoted var should fail");
        assert!(error.contains("interpolation must occupy an unquoted shell word"));
    }
    #[test]
    fn test_cmd_words_mixed_var_and_splat() {
        let words = parse_cmd_words("git {cmd} {args...}").expect("mixed should parse");
        assert!(matches!(words.first(), Some(CmdWord::Literal(value)) if value == "git"));
        assert!(matches!(words.get(1), Some(CmdWord::Var(value)) if value == "cmd"));
        assert!(matches!(words.get(2), Some(CmdWord::Splat(value)) if value == "args"));
    }
}
