//! Procedural macros for ACORN database utilities
use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Expr, ExprLit, Fields, GenericArgument, Lit, LitStr, Meta,
    PathArguments, Token, Type,
};

/// Derive deterministic, allocation-compatible ACORN validation.
///
/// Asynchronous custom rules use `#[validate(async(function = "path"))]` on fields or structs,
/// and `#[validate(each(async(function = "path")))]` on supported collections.
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_validate(&input).unwrap_or_else(|why| why.to_compile_error()).into()
}

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
            } else if is_option_timestamp(&field.ty) {
                quote! { #ident => crate::util::to_rfc3339, }
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
fn is_option_timestamp(ty: &syn::Type) -> bool {
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
            .is_some_and(|segment| segment.ident == "Timestamp"),
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
#[derive(Clone)]
enum ValidationRule {
    Required(RuleOptions),
    Length(Box<LengthOptions>),
    Range(Box<RangeOptions>),
    Contains(Expr, RuleOptions),
    Date(RuleOptions),
    Url(RuleOptions),
    Email(RuleOptions),
    Custom(syn::Path, RuleOptions),
    Async(syn::Path, RuleOptions),
    Nested,
    Pid(syn::Ident, RuleOptions),
    Scalar(syn::Ident, RuleOptions),
    Version(RuleOptions),
    Year(RuleOptions),
    Each(Box<ValidationRule>),
}
#[derive(Clone, Default)]
struct RuleOptions {
    code: Option<LitStr>,
    message: Option<LitStr>,
}
#[derive(Clone, Default)]
struct LengthOptions {
    min: Option<Expr>,
    max: Option<Expr>,
    equal: Option<Expr>,
    rule: RuleOptions,
}
#[derive(Clone, Default)]
struct RangeOptions {
    min: Option<Expr>,
    max: Option<Expr>,
    rule: RuleOptions,
}
struct SchemaRule {
    function: syn::Path,
    skip_on_field_errors: bool,
    asynchronous: bool,
}
impl ValidationRule {
    fn is_async(&self) -> bool {
        match self {
            | Self::Async(_, _) => true,
            | Self::Each(inner) => inner.is_async(),
            | _ => false,
        }
    }
}
fn acorn_path() -> proc_macro2::TokenStream {
    match crate_name("acorn-lib") {
        | Ok(FoundCrate::Itself) => quote! { crate },
        | Ok(FoundCrate::Name(name)) => {
            let name = match name.as_str() {
                | "acorn_lib" => format_ident!("acorn"),
                | _ => format_ident!("{name}"),
            };
            quote! { ::#name }
        }
        | Err(_) => quote! { ::acorn },
    }
}
fn expand_validate(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let acorn = acorn_path();
    let name = &input.ident;
    let fields = match &input.data {
        | Data::Struct(data) => match &data.fields {
            | Fields::Named(fields) => &fields.named,
            | _ => return Err(syn::Error::new_spanned(input, "Validate only works with named structs")),
        },
        | _ => return Err(syn::Error::new_spanned(input, "Validate only works with structs")),
    };
    let schema_rules = input
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("validate"))
        .flat_map(parse_validate_meta)
        .map(|meta| parse_schema_rule(&meta))
        .collect::<syn::Result<Vec<Option<SchemaRule>>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let field_rules = fields
        .iter()
        .map(|field| {
            let ident = field
                .ident
                .as_ref()
                .ok_or_else(|| syn::Error::new_spanned(field, "Validate requires named fields"))?;
            let path = ident.to_string().trim_start_matches("r#").to_string();
            let rules = field
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident("validate"))
                .flat_map(parse_validate_meta)
                .map(|meta| parse_rule(&meta, false))
                .collect::<syn::Result<Vec<_>>>()?;
            Ok((ident, &field.ty, path, rules))
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let field_validations = field_rules
        .iter()
        .map(|(ident, ty, path, rules)| {
            let statements = rules
                .iter()
                .filter(|rule| !rule.is_async())
                .map(|rule| expand_field_rule(&acorn, rule, ident, ty, path, false))
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(quote! { #(#statements)* })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let async_field_validations = field_rules
        .iter()
        .map(|(ident, ty, path, rules)| {
            let statements = rules
                .iter()
                .map(|rule| expand_field_rule(&acorn, rule, ident, ty, path, true))
                .collect::<syn::Result<Vec<_>>>()?;
            Ok(quote! { #(#statements)* })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let schema_validations = schema_rules.iter().filter(|rule| !rule.asynchronous).map(|rule| {
        let function = &rule.function;
        let condition = if rule.skip_on_field_errors {
            quote! { __report.is_empty() }
        } else {
            quote! { true }
        };
        quote! {
            if #condition {
                if let ::core::result::Result::Err(__error) = #function(self) {
                    __report.add("", __error);
                }
            }
        }
    });
    let async_schema_validations = schema_rules.iter().map(|rule| {
        let function = &rule.function;
        let condition = if rule.skip_on_field_errors {
            quote! { __report.is_empty() }
        } else {
            quote! { true }
        };
        let call = if rule.asynchronous {
            quote! { #function(self).await }
        } else {
            quote! { #function(self) }
        };
        quote! {
            if #condition {
                if let ::core::result::Result::Err(__error) = #call {
                    __report.add("", __error);
                }
            }
        }
    });
    let needs_async = schema_rules.iter().any(|rule| rule.asynchronous)
        || field_rules.iter().any(|(_, ty, _, rules)| {
            rules
                .iter()
                .any(|rule| rule.is_async() || matches!(rule, ValidationRule::Nested) && !is_recursive_box(ty, name))
        });
    let async_method = needs_async.then(|| {
        quote! {
            async fn validate_async(&self) -> ::core::result::Result<(), #acorn::validation::ValidationReport> {
                let mut __report = #acorn::validation::ValidationReport::new();
                #(#async_field_validations)*
                #(#async_schema_validations)*
                __report.finish()
            }
        }
    });
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics #acorn::validation::Validate for #name #type_generics #where_clause {
            fn validate(&self) -> ::core::result::Result<(), #acorn::validation::ValidationReport> {
                let mut __report = #acorn::validation::ValidationReport::new();
                #(#field_validations)*
                #(#schema_validations)*
                __report.finish()
            }
            #async_method
        }
    })
}
fn parse_validate_meta(attribute: &syn::Attribute) -> Vec<Meta> {
    let tokens = match &attribute.meta {
        | Meta::List(list) => normalize_async_tokens(list.tokens.clone()),
        | _ => proc_macro2::TokenStream::new(),
    };
    Punctuated::<Meta, Token![,]>::parse_terminated
        .parse2(tokens)
        .map(Punctuated::into_iter)
        .map(Iterator::collect)
        .unwrap_or_else(|_| {
            vec![Meta::Path(syn::Path::from(syn::Ident::new(
                "invalid_validate_attribute",
                proc_macro2::Span::call_site(),
            )))]
        })
}
fn parse_schema_rule(meta: &Meta) -> syn::Result<Option<SchemaRule>> {
    match meta {
        | Meta::List(list) if list.path.is_ident("schema") || is_path_ident(&list.path, "async") => {
            let nested = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            ensure_keys(&nested, &["function", "skip_on_field_errors"])?;
            let function = nested
                .iter()
                .find_map(|value| meta_string(value, "function"))
                .ok_or_else(|| syn::Error::new_spanned(list, "validation rule requires function = \"path\""))?
                .parse()?;
            let skip_on_field_errors = nested.iter().find_map(|value| meta_bool(value, "skip_on_field_errors")).unwrap_or(true);
            Ok(Some(SchemaRule {
                function,
                skip_on_field_errors,
                asynchronous: is_path_ident(&list.path, "async"),
            }))
        }
        | _ => Err(syn::Error::new_spanned(meta, "only schema(...) and async(...) are supported on a struct")),
    }
}
fn parse_rule(meta: &Meta, inside_each: bool) -> syn::Result<ValidationRule> {
    let path = meta.path();
    if path.is_ident("required") {
        Ok(ValidationRule::Required(parse_rule_options(meta)?))
    } else if path.is_ident("length") {
        Ok(ValidationRule::Length(Box::new(parse_length_options(meta)?)))
    } else if path.is_ident("range") {
        Ok(ValidationRule::Range(Box::new(parse_range_options(meta)?)))
    } else if path.is_ident("contains") {
        let nested = nested_meta(meta)?;
        ensure_keys(&nested, &["pattern", "code", "message"])?;
        let pattern = nested
            .iter()
            .find_map(|value| meta_value_expr(value, "pattern"))
            .ok_or_else(|| syn::Error::new_spanned(meta, "contains requires pattern"))?;
        Ok(ValidationRule::Contains(pattern, options_from_nested(&nested)))
    } else if path.is_ident("date") {
        Ok(ValidationRule::Date(parse_rule_options(meta)?))
    } else if path.is_ident("url") {
        Ok(ValidationRule::Url(parse_rule_options(meta)?))
    } else if path.is_ident("email") {
        Ok(ValidationRule::Email(parse_rule_options(meta)?))
    } else if path.is_ident("custom") {
        let nested = nested_meta(meta)?;
        ensure_keys(&nested, &["function", "code", "message"])?;
        let function = nested
            .iter()
            .find_map(|value| meta_string(value, "function"))
            .ok_or_else(|| syn::Error::new_spanned(meta, "custom requires function = \"path\""))?
            .parse()?;
        Ok(ValidationRule::Custom(function, options_from_nested(&nested)))
    } else if is_path_ident(path, "async") {
        let nested = nested_meta(meta)?;
        ensure_keys(&nested, &["function", "code", "message"])?;
        let function = nested
            .iter()
            .find_map(|value| meta_string(value, "function"))
            .ok_or_else(|| syn::Error::new_spanned(meta, "async requires function = \"path\""))?
            .parse()?;
        Ok(ValidationRule::Async(function, options_from_nested(&nested)))
    } else if path.is_ident("nested") {
        if inside_each {
            Err(syn::Error::new_spanned(meta, "nested validation is not supported inside each"))
        } else {
            Ok(ValidationRule::Nested)
        }
    } else if path.is_ident("version") {
        Ok(ValidationRule::Version(parse_rule_options(meta)?))
    } else if path.is_ident("year") {
        Ok(ValidationRule::Year(parse_rule_options(meta)?))
    } else if path.is_ident("each") {
        if inside_each {
            return Err(syn::Error::new_spanned(meta, "recursive each validation is not supported"));
        }
        let nested = nested_meta(meta)?;
        if nested.len() != 1 {
            return Err(syn::Error::new_spanned(meta, "each accepts exactly one scalar rule"));
        }
        Ok(ValidationRule::Each(Box::new(parse_rule(&nested[0], true)?)))
    } else {
        let supported_pid = ["ark", "arxiv", "doi", "handle", "isbn", "isni", "orcid", "patent", "raid", "ror", "swhid"];
        let supported_scalar = [
            "phone",
            "commit",
            "latitude",
            "longitude",
            "state",
            "kebabcase",
            "camelCase",
            "camelcase",
            "snake_case",
        ];
        match path.get_ident() {
            | Some(ident) if supported_pid.iter().any(|rule| ident == rule) => Ok(ValidationRule::Pid(ident.clone(), parse_rule_options(meta)?)),
            | Some(ident) if supported_scalar.iter().any(|rule| ident == rule) => {
                Ok(ValidationRule::Scalar(ident.clone(), parse_rule_options(meta)?))
            }
            | _ => Err(syn::Error::new_spanned(meta, "unknown validation rule")),
        }
    }
}
fn nested_meta(meta: &Meta) -> syn::Result<Vec<Meta>> {
    match meta {
        | Meta::List(list) => Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(normalize_async_tokens(list.tokens.clone()))
            .map(Punctuated::into_iter)
            .map(Iterator::collect),
        | _ => Ok(Vec::new()),
    }
}
fn is_path_ident(path: &syn::Path, name: &str) -> bool {
    path.get_ident().is_some_and(|ident| ident.to_string().trim_start_matches("r#") == name)
}
fn normalize_async_tokens(tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    tokens
        .into_iter()
        .map(|token| match token {
            | proc_macro2::TokenTree::Ident(ident) if ident == "async" => {
                proc_macro2::TokenTree::Ident(proc_macro2::Ident::new_raw("async", ident.span()))
            }
            | proc_macro2::TokenTree::Group(group) => {
                let mut normalized = proc_macro2::Group::new(group.delimiter(), normalize_async_tokens(group.stream()));
                normalized.set_span(group.span());
                proc_macro2::TokenTree::Group(normalized)
            }
            | token => token,
        })
        .collect()
}
fn parse_rule_options(meta: &Meta) -> syn::Result<RuleOptions> {
    let nested = nested_meta(meta)?;
    ensure_keys(&nested, &["code", "message"])?;
    Ok(options_from_nested(&nested))
}
fn options_from_nested(nested: &[Meta]) -> RuleOptions {
    RuleOptions {
        code: nested.iter().find_map(|value| meta_string(value, "code")),
        message: nested.iter().find_map(|value| meta_string(value, "message")),
    }
}
fn parse_length_options(meta: &Meta) -> syn::Result<LengthOptions> {
    let nested = nested_meta(meta)?;
    ensure_keys(&nested, &["min", "max", "equal", "code", "message"])?;
    Ok(LengthOptions {
        min: nested.iter().find_map(|value| meta_expr(value, "min")),
        max: nested.iter().find_map(|value| meta_expr(value, "max")),
        equal: nested.iter().find_map(|value| meta_expr(value, "equal")),
        rule: options_from_nested(&nested),
    })
}
fn parse_range_options(meta: &Meta) -> syn::Result<RangeOptions> {
    let nested = nested_meta(meta)?;
    ensure_keys(&nested, &["min", "max", "code", "message"])?;
    Ok(RangeOptions {
        min: nested.iter().find_map(|value| meta_expr(value, "min")),
        max: nested.iter().find_map(|value| meta_expr(value, "max")),
        rule: options_from_nested(&nested),
    })
}
fn ensure_keys<'a>(nested: impl IntoIterator<Item = &'a Meta>, allowed: &[&str]) -> syn::Result<()> {
    nested.into_iter().try_for_each(|meta| {
        allowed
            .iter()
            .any(|name| meta.path().is_ident(name))
            .then_some(())
            .ok_or_else(|| syn::Error::new_spanned(meta, "unknown validation argument"))
    })
}
fn meta_expr(meta: &Meta, name: &str) -> Option<Expr> {
    match meta {
        | Meta::NameValue(value) if value.path.is_ident(name) => match &value.value {
            | Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => value.parse().ok(),
            | expression => Some(expression.clone()),
        },
        | _ => None,
    }
}
fn meta_value_expr(meta: &Meta, name: &str) -> Option<Expr> {
    match meta {
        | Meta::NameValue(value) if value.path.is_ident(name) => Some(value.value.clone()),
        | _ => None,
    }
}
fn meta_string(meta: &Meta, name: &str) -> Option<LitStr> {
    match meta {
        | Meta::NameValue(value) if value.path.is_ident(name) => match &value.value {
            | Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => Some(value.clone()),
            | _ => None,
        },
        | _ => None,
    }
}
fn meta_bool(meta: &Meta, name: &str) -> Option<bool> {
    match meta {
        | Meta::NameValue(value) if value.path.is_ident(name) => match &value.value {
            | Expr::Lit(ExprLit { lit: Lit::Bool(value), .. }) => Some(value.value),
            | _ => None,
        },
        | _ => None,
    }
}
fn type_argument<'a>(ty: &'a Type, wrapper: &str) -> Option<&'a Type> {
    match ty {
        | Type::Path(value) => value
            .path
            .segments
            .last()
            .filter(|segment| segment.ident == wrapper)
            .and_then(|segment| match &segment.arguments {
                | PathArguments::AngleBracketed(arguments) => arguments.args.iter().find_map(|argument| match argument {
                    | GenericArgument::Type(ty) => Some(ty),
                    | _ => None,
                }),
                | _ => None,
            }),
        | _ => None,
    }
}
fn is_recursive_box(ty: &Type, name: &syn::Ident) -> bool {
    let inner = type_argument(ty, "Option").unwrap_or(ty);
    type_argument(inner, "Box").is_some_and(|boxed| match boxed {
        | Type::Path(path) => path.path.segments.last().is_some_and(|segment| segment.ident == *name),
        | _ => false,
    })
}
fn is_collection(ty: &Type) -> bool {
    match ty {
        | Type::Array(_) | Type::Slice(_) => true,
        | Type::Reference(reference) => is_collection(&reference.elem),
        | _ => ["Vec", "OneOrMany", "BTreeMap"].iter().any(|name| type_argument(ty, name).is_some()),
    }
}
fn expand_field_rule(
    acorn: &proc_macro2::TokenStream,
    rule: &ValidationRule,
    ident: &syn::Ident,
    ty: &Type,
    path: &str,
    asynchronous: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    if matches!(rule, ValidationRule::Each(_)) {
        let inner = type_argument(ty, "Option").unwrap_or(ty);
        if !is_collection(inner) {
            return Err(syn::Error::new_spanned(
                ty,
                "each can only validate Vec, arrays, slices, OneOrMany, or optional wrappers",
            ));
        }
    }
    if let ValidationRule::Required(options) = rule {
        let error = error_tokens(acorn, "required", options, quote! { "required" });
        return match type_argument(ty, "Option") {
            | Some(_) => Ok(quote! {
                if self.#ident.is_none() {
                    __report.add(#path, #error);
                }
            }),
            | None => Err(syn::Error::new_spanned(ty, "required can only validate Option fields")),
        };
    }
    let (value, inner_ty, wrapper) = match type_argument(ty, "Option") {
        | Some(inner) => (
            quote! { __value },
            inner,
            Some(quote! { if let ::core::option::Option::Some(__value) = self.#ident.as_ref() }),
        ),
        | None => (quote! { &self.#ident }, ty, None),
    };
    let body = expand_rule_body(acorn, rule, &value, inner_ty, path, asynchronous)?;
    Ok(match wrapper {
        | Some(wrapper) => quote! { #wrapper { #body } },
        | None => body,
    })
}
fn expand_rule_body(
    acorn: &proc_macro2::TokenStream,
    rule: &ValidationRule,
    value: &proc_macro2::TokenStream,
    ty: &Type,
    path: &str,
    asynchronous: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    match rule {
        | ValidationRule::Required(_) => Err(syn::Error::new_spanned(ty, "required cannot be nested in this rule")),
        | ValidationRule::Nested => {
            if is_collection(ty) {
                if asynchronous {
                    Ok(quote! {
                        for (__index, __item) in #acorn::validation::Each::each_iter(#value).enumerate() {
                            if let ::core::result::Result::Err(__nested) = #acorn::validation::Validate::validate_async(__item).await {
                                __report.append_prefixed(&#acorn::prelude::alloc::format!("{}[{}]", #path, __index), __nested);
                            }
                        }
                    })
                } else {
                    Ok(quote! {
                        #acorn::validation::Each::each_iter(#value).enumerate().for_each(|(__index, __item)| {
                            if let ::core::result::Result::Err(__nested) = #acorn::validation::Validate::validate(__item) {
                                __report.append_prefixed(&#acorn::prelude::alloc::format!("{}[{}]", #path, __index), __nested);
                            }
                        });
                    })
                }
            } else if type_argument(ty, "Box").is_some() {
                if asynchronous {
                    Ok(quote! {
                        if let ::core::result::Result::Err(__nested) = #acorn::validation::Validate::validate_async(&**(#value)).await {
                            __report.append_prefixed(#path, __nested);
                        }
                    })
                } else {
                    Ok(quote! {
                        if let ::core::result::Result::Err(__nested) = #acorn::validation::Validate::validate(&**(#value)) {
                            __report.append_prefixed(#path, __nested);
                        }
                    })
                }
            } else if asynchronous {
                Ok(quote! {
                    if let ::core::result::Result::Err(__nested) = #acorn::validation::Validate::validate_async(#value).await {
                        __report.append_prefixed(#path, __nested);
                    }
                })
            } else {
                Ok(quote! {
                    if let ::core::result::Result::Err(__nested) = #acorn::validation::Validate::validate(#value) {
                        __report.append_prefixed(#path, __nested);
                    }
                })
            }
        }
        | ValidationRule::Each(inner) => match inner.as_ref() {
            | ValidationRule::Nested | ValidationRule::Each(_) | ValidationRule::Required(_) => {
                Err(syn::Error::new_spanned(ty, "each accepts one non-nested scalar rule"))
            }
            | ValidationRule::Async(function, options) => {
                let expression = override_error(quote! { #function(__item).await }, options)?;
                Ok(quote! {
                    for (__index, __item) in #acorn::validation::Each::each_iter(#value).enumerate() {
                        if let ::core::result::Result::Err(mut __error) = #expression {
                            __error.add_param("value".into(), __item);
                            __report.add(#acorn::prelude::alloc::format!("{}[{}]", #path, __index), __error);
                        }
                    }
                })
            }
            | _ => {
                let expression = scalar_expression(acorn, inner, quote! { __item })?;
                Ok(quote! {
                    #acorn::validation::apply_each(&mut __report, #path, #value, |__item| #expression);
                })
            }
        },
        | ValidationRule::Async(function, options) => {
            let expression = override_error(quote! { #function(#value).await }, options)?;
            Ok(quote! {
                if let ::core::result::Result::Err(mut __error) = #expression {
                    __error.add_param("value".into(), #value);
                    __report.add(#path, __error);
                }
            })
        }
        | _ => {
            let expression = scalar_expression(acorn, rule, value.clone())?;
            Ok(quote! {
                if let ::core::result::Result::Err(mut __error) = #expression {
                    __error.add_param("value".into(), #value);
                    __report.add(#path, __error);
                }
            })
        }
    }
}
fn scalar_expression(
    acorn: &proc_macro2::TokenStream,
    rule: &ValidationRule,
    value: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    match rule {
        | ValidationRule::Length(options) => {
            let min = options
                .min
                .as_ref()
                .map(|min| quote! { __length < (#min) as usize })
                .unwrap_or_else(|| quote! { false });
            let max = options
                .max
                .as_ref()
                .map(|max| quote! { __length > (#max) as usize })
                .unwrap_or_else(|| quote! { false });
            let equal = options
                .equal
                .as_ref()
                .map(|equal| quote! { __length != (#equal) as usize })
                .unwrap_or_else(|| quote! { false });
            let error = error_tokens(acorn, "length", &options.rule, quote! { "length" });
            let min_param = options.min.as_ref().map(|min| quote! { __error.add_param("min".into(), &(#min)); });
            let max_param = options.max.as_ref().map(|max| quote! { __error.add_param("max".into(), &(#max)); });
            let equal_param = options
                .equal
                .as_ref()
                .map(|equal| quote! { __error.add_param("equal".into(), &(#equal)); });
            Ok(quote! {{
                let __length = (#value).len();
                let __invalid = [#min, #max, #equal].into_iter().any(::core::convert::identity);
                if __invalid {
                    let mut __error = #error;
                    #min_param
                    #max_param
                    #equal_param
                    ::core::result::Result::Err(__error)
                } else {
                    ::core::result::Result::Ok(())
                }
            }})
        }
        | ValidationRule::Range(options) => {
            let min = options
                .min
                .as_ref()
                .map(|min| quote! { *(#value) < (#min) })
                .unwrap_or_else(|| quote! { false });
            let max = options
                .max
                .as_ref()
                .map(|max| quote! { *(#value) > (#max) })
                .unwrap_or_else(|| quote! { false });
            let error = error_tokens(acorn, "range", &options.rule, quote! { "range" });
            let min_param = options.min.as_ref().map(|min| quote! { __error.add_param("min".into(), &(#min)); });
            let max_param = options.max.as_ref().map(|max| quote! { __error.add_param("max".into(), &(#max)); });
            Ok(quote! {{
                let __invalid = [#min, #max].into_iter().any(::core::convert::identity);
                if __invalid {
                    let mut __error = #error;
                    #min_param
                    #max_param
                    ::core::result::Result::Err(__error)
                } else {
                    ::core::result::Result::Ok(())
                }
            }})
        }
        | ValidationRule::Contains(pattern, options) => {
            let error = error_tokens(acorn, "contains", options, quote! { "contains" });
            Ok(quote! {{
                if (#value).contains(#pattern) {
                    ::core::result::Result::Ok(())
                } else {
                    ::core::result::Result::Err(#error)
                }
            }})
        }
        | ValidationRule::Date(options) => override_error(quote! { #acorn::validation::rules::date((#value).as_ref()) }, options),
        | ValidationRule::Url(options) => override_error(quote! { #acorn::validation::rules::url((#value).as_ref()) }, options),
        | ValidationRule::Email(options) => override_error(quote! { #acorn::validation::rules::email((#value).as_ref()) }, options),
        | ValidationRule::Version(options) => override_error(quote! { #acorn::validation::rules::version((#value).as_ref()) }, options),
        | ValidationRule::Year(options) => override_error(quote! { #acorn::validation::rules::year(#value) }, options),
        | ValidationRule::Custom(function, options) => override_error(quote! { #function(#value) }, options),
        | ValidationRule::Pid(ident, options) => {
            let function = syn::Ident::new(&format!("is_{ident}"), ident.span());
            override_error(quote! { #acorn::schema::validate::#function((#value).as_ref()) }, options)
        }
        | ValidationRule::Scalar(ident, options) => {
            let function = match ident.to_string().as_str() {
                | "phone" => syn::Ident::new("is_phone_number", ident.span()),
                | "camelCase" | "camelcase" => syn::Ident::new("is_camelcase", ident.span()),
                | name => syn::Ident::new(&format!("is_{name}"), ident.span()),
            };
            override_error(quote! { #acorn::schema::validate::#function(#value) }, options)
        }
        | _ => Err(syn::Error::new(proc_macro2::Span::call_site(), "rule is not scalar")),
    }
}
fn error_tokens(
    acorn: &proc_macro2::TokenStream,
    code: &str,
    options: &RuleOptions,
    default_message: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let code = options.code.as_ref().map(|value| quote! { #value }).unwrap_or_else(|| quote! { #code });
    let message = options.message.as_ref().map(|value| quote! { #value }).unwrap_or(default_message);
    quote! { #acorn::validation::ValidationError::new(#code).with_message((#message).into()) }
}
fn override_error(expression: proc_macro2::TokenStream, options: &RuleOptions) -> syn::Result<proc_macro2::TokenStream> {
    let code = options.code.as_ref().map(|value| quote! { __error = __error.with_code(#value); });
    let message = options
        .message
        .as_ref()
        .map(|value| quote! { __error = __error.with_message((#value).into()); });
    Ok(quote! {
        (#expression).map_err(|mut __error| {
            #code
            #message
            __error
        })
    })
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
    fn test_option_timestamp_detection() {
        let timestamp_ty: Type = parse_quote!(Option<Timestamp>);
        let string_ty: Type = parse_quote!(Option<String>);
        assert!(is_option_timestamp(&timestamp_ty));
        assert!(!is_option_timestamp(&string_ty));
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
        assert!(tokens.contains("executed_at => crate :: util :: to_rfc3339"));
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
            struct CanonicalRecordRow {
                id: Option<i64>,
            }
        };
        let tokens = expand_database_row(&input).expect("derive expansion should succeed").to_string();
        assert!(tokens.contains("pub const TABLE_NAME"));
        assert!(tokens.contains("canonical_record"));
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
        assert!(tokens.contains(":: acorn :: validation"));
        assert!(tokens.contains("validate_async"));
        assert!(tokens.contains("validate_value") && tokens.contains("await"));
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
