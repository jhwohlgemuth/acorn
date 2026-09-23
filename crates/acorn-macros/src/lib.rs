//! Procedural macros for ACORN
use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Expr, ExprLit, Fields, GenericArgument, Lit, LitStr, Meta,
    PathArguments, Token, Type,
};

mod command;

use command::expand_cmd_sh_words;
#[derive(Clone)]
enum ValidationRule {
    Required(RuleOptions),
    Length(Box<LengthOptions>),
    Range(Box<RangeOptions>),
    Contains(Expr, RuleOptions),
    Date(RuleOptions),
    Timestamp(RuleOptions),
    Nonempty(RuleOptions),
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
    Not(Box<ValidationRule>, RuleOptions),
    Optional(Box<ValidationRule>),
    Or(Vec<ValidationRule>, RuleOptions),
}
#[derive(Clone, Copy)]
enum WithFieldVisibility {
    Private,
    Public,
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
struct RowConfig {
    order_by: Option<String>,
    table: Option<String>,
}
#[derive(Clone, Default)]
struct RuleOptions {
    code: Option<LitStr>,
    message: Option<LitStr>,
}
struct SchemaRule {
    function: syn::Path,
    skip_on_field_errors: bool,
    asynchronous: bool,
    context: Option<Expr>,
    code: Option<LitStr>,
    message: Option<LitStr>,
    guard: Option<Expr>,
}
struct WithConfig {
    prefix: String,
}
struct WithFieldConfig {
    some: bool,
    skip: bool,
    visibility: Option<WithFieldVisibility>,
}
impl RuleOptions {
    fn has_overrides(&self) -> bool {
        self.code.is_some() || self.message.is_some()
    }
}
impl ValidationRule {
    fn is_async(&self) -> bool {
        match self {
            | Self::Async(_, _) => true,
            | Self::Each(inner) | Self::Not(inner, _) | Self::Optional(inner) => inner.is_async(),
            | Self::Or(alternatives, _) => alternatives.iter().any(Self::is_async),
            | _ => false,
        }
    }
}
fn acorn_path() -> proc_macro2::TokenStream {
    for candidate in ["acorn-schema", "acorn-lib", "acorn-core"] {
        if let Ok(found) = crate_name(candidate) {
            return match found {
                | FoundCrate::Itself => quote! { crate },
                | FoundCrate::Name(name) => {
                    let mapped = match name.as_str() {
                        | "acorn_lib" => "acorn".to_string(),
                        | other => other.replace('-', "_"),
                    };
                    let ident = format_ident!("{}", mapped);
                    quote! { ::#ident }
                }
            };
        }
    }
    quote! { ::acorn }
}
/// Parses a literal `cmd!("...")` command into shell words with `{var}` and `{args...}` interpolation.
#[proc_macro]
pub fn cmd_sh_words(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    expand_cmd_sh_words(&input).unwrap_or_else(|why| why.to_compile_error()).into()
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
/// Derive deterministic, allocation-compatible ACORN validation.
///
/// Asynchronous custom rules use `#[validate(async(function = "path"))]` on fields or structs,
/// and `#[validate(each(async(function = "path")))]` on supported collections.
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_validate(&input).unwrap_or_else(|why| why.to_compile_error()).into()
}
/// Derive immutable consuming `with_*` setters for structs
///
/// Generates `with_<field>(self, value: impl Into<T>) -> Self` for named fields and `with_<index>(self, value: impl Into<T>) -> Self` for tuple structs.
/// The setter consumes `self` and returns an updated `Self`, enabling functional chaining without mutation.
///
/// ### Attributes
/// Container: `#[with(prefix = "with_")]` — customize method prefix (default `with_`).
/// Field:
/// - `#[with(public)]` — generate a public setter.
/// - `#[with(private)]` — generate a private setter.
/// - `#[with(some)]` — accept the inner value for an `Option<T>` field and store `Some(value)`.
/// - `#[with(skip)]` — omit the setter for that field.
///
/// Field arguments may be combined when compatible, such as `#[with(public, some)]`.
/// Fields without a visibility argument use the struct visibility.
/// Owned `some` values use `impl Into<T>`; referenced values use the exact reference type so dereference coercions remain available.
/// `some` requires a field written as `Option<T>` and does not generate a way to assign `None`.
///
/// ### Examples
/// ```ignore
/// use acorn_macros::With;
///
/// #[derive(With)]
/// pub struct Config {
///     #[with(public)]
///     host: String,
///     #[with(private)]
///     port: u16,
///     #[with(public, some)]
///     label: Option<String>,
/// }
/// let c = Config {
///     host: "a".into(),
///     port: 80,
///     label: None,
/// };
/// let c2 = c.with_host("example.com").with_port(8080u16).with_label("production");
///
/// #[derive(With)]
/// pub struct Tuple(String, u32);
/// let t = Tuple("a".into(), 1);
/// let t2 = t.with_0("b").with_1(2u32);
/// ```
#[proc_macro_derive(With, attributes(with))]
pub fn derive_with(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_with(&input).unwrap_or_else(|why| why.to_compile_error()).into()
}
fn ensure_keys<'a>(nested: impl IntoIterator<Item = &'a Meta>, allowed: &[&str]) -> syn::Result<()> {
    nested.into_iter().try_for_each(|meta| {
        allowed
            .iter()
            .any(|name| meta.path().is_ident(name) || is_path_ident(meta.path(), name))
            .then_some(())
            .ok_or_else(|| syn::Error::new_spanned(meta, "unknown validation argument"))
    })
}
fn error_tokens(
    _acorn: &proc_macro2::TokenStream,
    code: &str,
    options: &RuleOptions,
    default_message: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let code = options.code.as_ref().map(|value| quote! { #value }).unwrap_or_else(|| quote! { #code });
    let message = options.message.as_ref().map(|value| quote! { #value }).unwrap_or(default_message);
    quote! { ::acorn_core::validation::ValidationError::new(#code).with_message((#message)) }
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
                quote! { #ident => acorn_core::util::to_rfc3339, }
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
    if matches!(rule, ValidationRule::Optional(_)) {
        let inner = type_argument(ty, "Option").unwrap_or(ty);
        if !is_optional_string(inner) {
            return Err(syn::Error::new_spanned(
                ty,
                "optional can only validate String, str, or optional wrappers",
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
                                __report.append_prefixed(&#acorn::validation::index_path(#path, __index), __nested);
                            }
                        }
                    })
                } else {
                    Ok(quote! {
                        #acorn::validation::Each::each_iter(#value).enumerate().for_each(|(__index, __item)| {
                            if let ::core::result::Result::Err(__nested) = #acorn::validation::Validate::validate(__item) {
                                __report.append_prefixed(&#acorn::validation::index_path(#path, __index), __nested);
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
            | ValidationRule::Nested | ValidationRule::Each(_) | ValidationRule::Required(_) | ValidationRule::Optional(_) => {
                Err(syn::Error::new_spanned(ty, "each accepts one non-nested scalar rule"))
            }
            | ValidationRule::Async(function, options) => {
                let expression = override_error(quote! { #function(__item).await }, options)?;
                Ok(quote! {
                    for (__index, __item) in #acorn::validation::Each::each_iter(#value).enumerate() {
                        if let ::core::result::Result::Err(mut __error) = #expression {
                            __error.add_param("value".into(), __item);
                            __report.add(#acorn::validation::index_path(#path, __index), __error);
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
        | ValidationRule::Optional(inner) => {
            let expression = scalar_expression(acorn, inner, value.clone())?;
            let optional = optional_value(value, ty);
            Ok(quote! {
                if !#acorn::validation::Optional::is_absent(#optional) {
                    if let ::core::result::Result::Err(mut __error) = #expression {
                        __error.add_param("value".into(), #value);
                        __report.add(#path, __error);
                    }
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
fn expand_schema_rule(rule: &SchemaRule, asynchronous: bool) -> proc_macro2::TokenStream {
    let function = &rule.function;
    let condition = if rule.skip_on_field_errors {
        quote! { __report.is_empty() }
    } else {
        quote! { true }
    };
    let context = rule.context.as_ref().map(|expr| quote! { #expr }).unwrap_or_else(|| quote! { () });
    let call = if asynchronous {
        quote! { #function(self, &__context).await }
    } else {
        quote! { #function(self, &__context) }
    };
    let code = rule.code.as_ref().map(|value| quote! { __error = __error.with_code(#value); });
    let message = rule.message.as_ref().map(|value| quote! { __error = __error.with_message((#value)); });
    let remap = if code.is_some() || message.is_some() {
        quote! {
            let mut __error = __issue.error;
            #code
            #message
            __report.add(__issue.path, __error);
        }
    } else {
        quote! { __report.add(__issue.path, __issue.error); }
    };
    let guard = rule.guard.as_ref().map(|expr| quote! { if #expr }).unwrap_or_else(|| quote! {});
    quote! {
        if #condition {
            let __context = #context;
            #guard {
                if let ::core::result::Result::Err(__schema_report) = #call {
                    for __issue in ::core::iter::IntoIterator::into_iter(__schema_report.into_issues()) {
                        #remap
                    }
                }
            }
        }
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
    let schema_validations = schema_rules
        .iter()
        .filter(|rule| !rule.asynchronous)
        .map(|rule| expand_schema_rule(rule, false));
    let async_schema_validations = schema_rules.iter().map(|rule| expand_schema_rule(rule, rule.asynchronous));
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
fn expand_with(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let vis = &input.vis;
    let config = extract_with_config(&input.attrs)?;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let methods = match &input.data {
        | Data::Struct(data) => match &data.fields {
            | Fields::Named(fields) => expand_with_named(&fields.named, &config, vis),
            | Fields::Unnamed(fields) => expand_with_unnamed(&fields.unnamed, &config, vis),
            | Fields::Unit => Ok(Vec::new()),
        },
        | Data::Enum(data) => expand_with_enum(data, &config, vis),
        | _ => Err(syn::Error::new_spanned(input, "With only works with structs or enums")),
    };
    methods.map(|methods| {
        quote! {
            impl #impl_generics #name #type_generics #where_clause {
                #(#methods)*
            }
        }
    })
}
fn expand_with_enum(data: &syn::DataEnum, config: &WithConfig, vis: &syn::Visibility) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let candidates = data
        .variants
        .iter()
        .flat_map(|variant| match &variant.fields {
            | Fields::Named(fields) => fields.named.iter().map(move |field| (variant, field)).collect::<Vec<_>>(),
            | Fields::Unnamed(_) | Fields::Unit => Vec::new(),
        })
        .map(|(variant, field)| extract_with_field_config(&field.attrs).map(|field_config| (variant, field, field_config)))
        .collect::<syn::Result<Vec<_>>>();
    candidates.and_then(|candidates| {
        let duplicate = candidates.iter().enumerate().find_map(|(index, (_, field, config))| {
            let ident = field.ident.as_ref();
            (!config.skip)
                .then_some(ident)
                .flatten()
                .and_then(|ident| {
                    candidates
                        .iter()
                        .skip(index.saturating_add(1))
                        .find(|(_, candidate, candidate_config)| !candidate_config.skip && candidate.ident.as_ref() == Some(ident))
                })
                .and(ident)
        });
        match duplicate {
            | Some(ident) => Err(syn::Error::new_spanned(
                ident,
                "With requires field names to be unique across enum variants",
            )),
            | None => candidates
                .into_iter()
                .filter(|(_, _, field_config)| !field_config.skip)
                .map(|(variant, field, field_config)| expand_with_enum_field(variant, field, field_config, config, vis))
                .collect(),
        }
    })
}
fn expand_with_enum_field(
    variant: &syn::Variant,
    field: &syn::Field,
    field_config: WithFieldConfig,
    config: &WithConfig,
    vis: &syn::Visibility,
) -> syn::Result<proc_macro2::TokenStream> {
    let ident = field
        .ident
        .as_ref()
        .ok_or_else(|| syn::Error::new_spanned(field, "With requires named fields"));
    ident.and_then(|ident| {
        let stripped = ident.to_string().trim_start_matches("r#").to_string();
        let method = format_ident!("{}{}", config.prefix, stripped, span = ident.span());
        let parameter = format_ident!("__with_{}", stripped, span = ident.span());
        let ty = &field.ty;
        let visibility = match field_config.visibility {
            | Some(WithFieldVisibility::Private) => quote! {},
            | Some(WithFieldVisibility::Public) => quote! { pub },
            | None => quote! { #vis },
        };
        expand_with_parameter(ty, &parameter, field_config.some).map(|(parameter_ty, value)| {
            let variant_ident = &variant.ident;
            let bindings = match &variant.fields {
                | Fields::Named(fields) => fields.named.iter().filter_map(|field| field.ident.as_ref()).collect::<Vec<_>>(),
                | Fields::Unnamed(_) | Fields::Unit => Vec::new(),
            };
            let patterns = bindings.iter().map(|binding| match *binding == ident {
                | true => quote! { #binding: _ },
                | false => quote! { #binding },
            });
            let fields = bindings.iter().map(|binding| match *binding == ident {
                | true => quote! { #binding: #value },
                | false => quote! { #binding },
            });
            quote! {
                #[doc = ::core::concat!("Setter for `", stringify!(#ident), "` — consumes `self` and returns updated `Self`.")]
                #visibility fn #method(self, #parameter: #parameter_ty) -> Self {
                    match self {
                        Self::#variant_ident { #(#patterns),* } => Self::#variant_ident { #(#fields),* },
                        other => other,
                    }
                }
            }
        })
    })
}
fn expand_with_named(
    fields: &Punctuated<syn::Field, Token![,]>,
    config: &WithConfig,
    vis: &syn::Visibility,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    fields
        .iter()
        .filter_map(|field| {
            let field_config = match extract_with_field_config(&field.attrs) {
                | Ok(c) => c,
                | Err(why) => return Some(Err(why)),
            };
            if field_config.skip {
                return None;
            }
            let ident = match &field.ident {
                | Some(id) => id,
                | None => return Some(Err(syn::Error::new_spanned(field, "With requires named fields"))),
            };
            let stripped = ident.to_string().trim_start_matches("r#").to_string();
            let method = format_ident!("{}{}", config.prefix, stripped, span = ident.span());
            let ty = &field.ty;
            let visibility = match field_config.visibility {
                | Some(WithFieldVisibility::Private) => quote! {},
                | Some(WithFieldVisibility::Public) => quote! { pub },
                | None => quote! { #vis },
            };
            let (parameter_ty, value) = match expand_with_parameter(ty, ident, field_config.some) {
                | Ok(expansion) => expansion,
                | Err(why) => return Some(Err(why)),
            };
            Some(Ok(quote! {
                #[doc = ::core::concat!("Setter for `", stringify!(#ident), "` — consumes `self` and returns updated `Self`.")]
                #[allow(clippy::needless_update)]
                #visibility fn #method(self, #ident: #parameter_ty) -> Self {
                    Self { #ident: #value, ..self }
                }
            }))
        })
        .collect()
}
fn expand_with_parameter(ty: &Type, parameter: &syn::Ident, some: bool) -> syn::Result<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    match some {
        | true => type_argument(ty, "Option")
            .ok_or_else(|| syn::Error::new_spanned(ty, "some requires an Option<T> field"))
            .map(|inner| match inner {
                | Type::Reference(_) => (quote! { #inner }, quote! { ::core::option::Option::Some(#parameter) }),
                | _ => (
                    quote! { impl ::core::convert::Into<#inner> },
                    quote! { ::core::option::Option::Some(::core::convert::Into::into(#parameter)) },
                ),
            }),
        | false => Ok((
            quote! { impl ::core::convert::Into<#ty> },
            quote! { ::core::convert::Into::into(#parameter) },
        )),
    }
}
fn expand_with_unnamed(
    fields: &Punctuated<syn::Field, Token![,]>,
    config: &WithConfig,
    vis: &syn::Visibility,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let len = fields.len();
    let self_idents: Vec<syn::Ident> = (0..len).map(|i| format_ident!("__self_{}", i)).collect();
    fields
        .iter()
        .enumerate()
        .filter_map(|(index, field)| {
            let field_config = match extract_with_field_config(&field.attrs) {
                | Ok(c) => c,
                | Err(why) => return Some(Err(why)),
            };
            if field_config.skip {
                return None;
            }
            let ty = &field.ty;
            let method = format_ident!("{}{}", config.prefix, index);
            let param = format_ident!("__with_{}", index);
            let visibility = match field_config.visibility {
                | Some(WithFieldVisibility::Private) => quote! {},
                | Some(WithFieldVisibility::Public) => quote! { pub },
                | None => quote! { #vis },
            };
            let (parameter_ty, value) = match expand_with_parameter(ty, &param, field_config.some) {
                | Ok(expansion) => expansion,
                | Err(why) => return Some(Err(why)),
            };
            let args: Vec<proc_macro2::TokenStream> = (0..len)
                .map(|j| {
                    if j == index {
                        value.clone()
                    } else {
                        let ident = &self_idents[j];
                        quote! { #ident }
                    }
                })
                .collect();
            let destructure = &self_idents;
            Some(Ok(quote! {
                #[doc = ::core::concat!("Setter for field `", stringify!(#index), "` — consumes `self` and returns updated `Self`.")]
                #[allow(clippy::needless_update)]
                #visibility fn #method(self, #param: #parameter_ty) -> Self {
                    let Self(#(#destructure),*) = self;
                    Self(#(#args),*)
                }
            }))
        })
        .collect()
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
fn extract_with_config(attrs: &[syn::Attribute]) -> syn::Result<WithConfig> {
    let mut prefix = "with_".to_string();
    for attr in attrs {
        if attr.path().is_ident("with") {
            match &attr.meta {
                | Meta::List(list) => {
                    let metas = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
                    for meta in metas {
                        if meta.path().is_ident("prefix") {
                            if let Meta::NameValue(value) = meta {
                                if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = value.value {
                                    prefix = lit_str.value();
                                } else {
                                    return Err(syn::Error::new_spanned(value, "prefix requires string value"));
                                }
                            } else {
                                return Err(syn::Error::new_spanned(meta, "prefix requires string value"));
                            }
                        } else {
                            return Err(syn::Error::new_spanned(meta, "unknown with container argument"));
                        }
                    }
                }
                | _ => return Err(syn::Error::new_spanned(attr, "with requires parenthesized arguments")),
            }
        }
    }
    Ok(WithConfig { prefix })
}
fn extract_with_field_config(attrs: &[syn::Attribute]) -> syn::Result<WithFieldConfig> {
    let mut some = false;
    let mut skip = false;
    let mut visibility = None;
    for attr in attrs {
        if attr.path().is_ident("with") {
            match &attr.meta {
                | Meta::List(list) => {
                    let metas = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
                    if metas.is_empty() {
                        return Err(syn::Error::new_spanned(attr, "with requires private, public, skip, or some"));
                    }
                    for meta in metas {
                        if meta.path().is_ident("private") {
                            if !matches!(meta, Meta::Path(_)) {
                                return Err(syn::Error::new_spanned(meta, "private takes no value"));
                            }
                            if skip {
                                return Err(syn::Error::new_spanned(meta, "private cannot be combined with skip"));
                            }
                            if visibility.is_some() {
                                return Err(syn::Error::new_spanned(meta, "with visibility may only be specified once"));
                            }
                            visibility = Some(WithFieldVisibility::Private);
                        } else if meta.path().is_ident("public") {
                            if !matches!(meta, Meta::Path(_)) {
                                return Err(syn::Error::new_spanned(meta, "public takes no value"));
                            }
                            if skip {
                                return Err(syn::Error::new_spanned(meta, "public cannot be combined with skip"));
                            }
                            if visibility.is_some() {
                                return Err(syn::Error::new_spanned(meta, "with visibility may only be specified once"));
                            }
                            visibility = Some(WithFieldVisibility::Public);
                        } else if meta.path().is_ident("skip") {
                            match meta {
                                | Meta::Path(_) if skip => {
                                    return Err(syn::Error::new_spanned(meta, "skip may only be specified once"));
                                }
                                | Meta::Path(_) if some || visibility.is_some() => {
                                    return Err(syn::Error::new_spanned(meta, "skip cannot be combined with other with arguments"));
                                }
                                | Meta::Path(_) => skip = true,
                                | _ => return Err(syn::Error::new_spanned(meta, "skip takes no value")),
                            }
                        } else if meta.path().is_ident("some") {
                            if !matches!(meta, Meta::Path(_)) {
                                return Err(syn::Error::new_spanned(meta, "some takes no value"));
                            }
                            if skip {
                                return Err(syn::Error::new_spanned(meta, "some cannot be combined with skip"));
                            }
                            if some {
                                return Err(syn::Error::new_spanned(meta, "some may only be specified once"));
                            }
                            some = true;
                        } else {
                            return Err(syn::Error::new_spanned(meta, "unknown with field argument"));
                        }
                    }
                }
                | Meta::Path(_) if skip => return Err(syn::Error::new_spanned(attr, "skip may only be specified once")),
                | Meta::Path(_) if some || visibility.is_some() => {
                    return Err(syn::Error::new_spanned(attr, "skip cannot be combined with other with arguments"));
                }
                | Meta::Path(_) => skip = true,
                | _ => return Err(syn::Error::new_spanned(attr, "invalid with attribute")),
            }
        }
    }
    Ok(WithFieldConfig { some, skip, visibility })
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
fn is_collection(ty: &Type) -> bool {
    match ty {
        | Type::Array(_) | Type::Slice(_) => true,
        | Type::Reference(reference) => is_collection(&reference.elem),
        | _ => ["Vec", "OneOrMany", "BTreeMap"].iter().any(|name| type_argument(ty, name).is_some()),
    }
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
fn is_optional_string(ty: &Type) -> bool {
    match ty {
        | Type::Path(path) => path.path.segments.last().is_some_and(|segment| segment.ident == "String"),
        | Type::Reference(reference) => match reference.elem.as_ref() {
            | Type::Path(path) => path.path.segments.last().is_some_and(|segment| segment.ident == "str"),
            | _ => false,
        },
        | _ => false,
    }
}
fn is_path_ident(path: &syn::Path, name: &str) -> bool {
    path.get_ident().is_some_and(|ident| ident.to_string().trim_start_matches("r#") == name)
}
fn is_recursive_box(ty: &Type, name: &syn::Ident) -> bool {
    let inner = type_argument(ty, "Option").unwrap_or(ty);
    type_argument(inner, "Box").is_some_and(|boxed| match boxed {
        | Type::Path(path) => path.path.segments.last().is_some_and(|segment| segment.ident == *name),
        | _ => false,
    })
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
fn meta_expr(meta: &Meta, name: &str) -> Option<Expr> {
    match meta {
        | Meta::NameValue(value) if value.path.is_ident(name) => match &value.value {
            | Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => value.parse().ok(),
            | expression => Some(expression.clone()),
        },
        | _ => None,
    }
}
fn meta_guard(meta: &Meta) -> syn::Result<Option<Expr>> {
    if !is_path_ident(meta.path(), "if") {
        return Ok(None);
    }
    match meta {
        | Meta::NameValue(value) => match &value.value {
            | Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) => value.parse().map(Some),
            | expression => Ok(Some(expression.clone())),
        },
        | _ => Err(syn::Error::new_spanned(meta, "if requires a boolean expression")),
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
fn meta_value_expr(meta: &Meta, name: &str) -> Option<Expr> {
    match meta {
        | Meta::NameValue(value) if value.path.is_ident(name) => Some(value.value.clone()),
        | _ => None,
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
fn normalize_async_tokens(tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    tokens
        .into_iter()
        .map(|token| match token {
            | proc_macro2::TokenTree::Ident(ident) if ident == "async" || ident == "if" => {
                let name = ident.to_string();
                proc_macro2::TokenTree::Ident(proc_macro2::Ident::new_raw(&name, ident.span()))
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
fn optional_value(value: &proc_macro2::TokenStream, ty: &Type) -> proc_macro2::TokenStream {
    match ty {
        | Type::Reference(_) => quote! { *(#value) },
        | _ => value.clone(),
    }
}
fn options_from_nested(nested: &[Meta]) -> RuleOptions {
    RuleOptions {
        code: nested.iter().find_map(|value| meta_string(value, "code")),
        message: nested.iter().find_map(|value| meta_string(value, "message")),
    }
}
fn override_error(expression: proc_macro2::TokenStream, options: &RuleOptions) -> syn::Result<proc_macro2::TokenStream> {
    let code = options.code.as_ref().map(|value| quote! { __error = __error.with_code(#value); });
    let message = options.message.as_ref().map(|value| quote! { __error = __error.with_message((#value)); });
    Ok(quote! {
        (#expression).map_err(|mut __error| {
            #code
            #message
            __error
        })
    })
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
fn parse_or_rule(meta: &Meta) -> syn::Result<ValidationRule> {
    let nested = nested_meta(meta)?;
    ensure_keys(nested.iter().filter(|value| matches!(value, Meta::NameValue(_))), &["code", "message"])?;
    let alternatives = nested
        .iter()
        .filter(|value| !matches!(value, Meta::NameValue(_)))
        .try_fold(Vec::new(), |mut alternatives, candidate| match parse_rule(candidate, true)? {
            | ValidationRule::Or(_, options) if options.has_overrides() => {
                Err(syn::Error::new_spanned(candidate, "nested or cannot override code or message"))
            }
            | ValidationRule::Or(nested, _) => {
                alternatives.extend(nested);
                Ok(alternatives)
            }
            | ValidationRule::Required(_)
            | ValidationRule::Nested
            | ValidationRule::Each(_)
            | ValidationRule::Optional(_)
            | ValidationRule::Async(_, _) => Err(syn::Error::new_spanned(candidate, "or accepts only synchronous scalar rules")),
            | alternative => {
                alternatives.push(alternative);
                Ok(alternatives)
            }
        })?;
    if alternatives.len() < 2 {
        Err(syn::Error::new_spanned(meta, "or requires at least two scalar rules"))
    } else {
        Ok(ValidationRule::Or(alternatives, options_from_nested(&nested)))
    }
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
    } else if path.is_ident("timestamp") {
        Ok(ValidationRule::Timestamp(parse_rule_options(meta)?))
    } else if path.is_ident("nonempty") {
        Ok(ValidationRule::Nonempty(parse_rule_options(meta)?))
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
    } else if path.is_ident("not") {
        let nested = nested_meta(meta)?;
        ensure_keys(nested.iter().filter(|value| matches!(value, Meta::NameValue(_))), &["code", "message"])?;
        let rules = nested.iter().filter(|value| !matches!(value, Meta::NameValue(_))).collect::<Vec<_>>();
        if rules.len() != 1 {
            return Err(syn::Error::new_spanned(meta, "not accepts exactly one scalar rule"));
        }
        let inner = parse_rule(rules[0], true)?;
        if matches!(
            inner,
            ValidationRule::Required(_)
                | ValidationRule::Nested
                | ValidationRule::Each(_)
                | ValidationRule::Not(_, _)
                | ValidationRule::Optional(_)
                | ValidationRule::Or(_, _)
                | ValidationRule::Async(_, _)
        ) {
            return Err(syn::Error::new_spanned(rules[0], "not accepts one synchronous scalar rule"));
        }
        Ok(ValidationRule::Not(Box::new(inner), options_from_nested(&nested)))
    } else if path.is_ident("optional") {
        let nested = nested_meta(meta)?;
        if nested.len() != 1 {
            return Err(syn::Error::new_spanned(meta, "optional accepts exactly one scalar rule"));
        }
        let candidate = &nested[0];
        if candidate.path().is_ident("required")
            || candidate.path().is_ident("nested")
            || candidate.path().is_ident("each")
            || candidate.path().is_ident("optional")
            || is_path_ident(candidate.path(), "async")
        {
            return Err(syn::Error::new_spanned(candidate, "optional accepts one synchronous scalar rule"));
        }
        Ok(ValidationRule::Optional(Box::new(parse_rule(candidate, true)?)))
    } else if path.is_ident("or") {
        parse_or_rule(meta)
    } else {
        let supported_pid = ["ark", "arxiv", "doi", "handle", "isbn", "isni", "orcid", "patent", "raid", "ror", "swhid"];
        let supported_scalar = [
            "camelCase",
            "camelcase",
            "commit",
            "country",
            "digest",
            "ipv4",
            "ipv6",
            "json",
            "kebabcase",
            "latitude",
            "longitude",
            "numeric",
            "phone",
            "snake_case",
            "state",
            "uri",
            "unix_epoch",
            "uuid",
            "zon",
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
fn parse_rule_options(meta: &Meta) -> syn::Result<RuleOptions> {
    let nested = nested_meta(meta)?;
    ensure_keys(&nested, &["code", "message"])?;
    Ok(options_from_nested(&nested))
}
fn parse_schema_rule(meta: &Meta) -> syn::Result<Option<SchemaRule>> {
    match meta {
        | Meta::List(list) if list.path.is_ident("schema") || is_path_ident(&list.path, "async") => {
            let nested = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            ensure_keys(&nested, &["function", "skip_on_field_errors", "context", "code", "message", "if"])?;
            let function = nested
                .iter()
                .find_map(|value| meta_string(value, "function"))
                .ok_or_else(|| syn::Error::new_spanned(list, "validation rule requires function = \"path\""))?
                .parse()?;
            let skip_on_field_errors = nested.iter().find_map(|value| meta_bool(value, "skip_on_field_errors")).unwrap_or(true);
            let context = nested.iter().find_map(|value| meta_expr(value, "context"));
            let code = nested.iter().find_map(|value| meta_string(value, "code"));
            let message = nested.iter().find_map(|value| meta_string(value, "message"));
            let guard = nested
                .iter()
                .find(|value| is_path_ident(value.path(), "if"))
                .map(meta_guard)
                .transpose()?
                .flatten();
            Ok(Some(SchemaRule {
                function,
                skip_on_field_errors,
                asynchronous: is_path_ident(&list.path, "async"),
                context,
                code,
                message,
                guard,
            }))
        }
        | _ => Err(syn::Error::new_spanned(meta, "only schema(...) and async(...) are supported on a struct")),
    }
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
        | ValidationRule::Timestamp(options) => override_error(quote! { #acorn::validation::rules::timestamp(#value) }, options),
        | ValidationRule::Nonempty(options) => override_error(quote! { #acorn::validation::rules::nonempty((#value).as_ref()) }, options),
        | ValidationRule::Version(options) => override_error(quote! { #acorn::validation::rules::version((#value).as_ref()) }, options),
        | ValidationRule::Year(options) => override_error(quote! { #acorn::validation::rules::year(#value) }, options),
        | ValidationRule::Custom(function, options) => override_error(quote! { #function(#value) }, options),
        | ValidationRule::Pid(ident, options) => override_error(quote! { #acorn::validation::rules::#ident((#value).as_ref()) }, options),
        | ValidationRule::Scalar(ident, options) => {
            let function = match ident.to_string().as_str() {
                | "camelCase" => syn::Ident::new("camelcase", ident.span()),
                | name => syn::Ident::new(name, ident.span()),
            };
            override_error(quote! { #acorn::validation::rules::#function(#value) }, options)
        }
        | ValidationRule::Not(inner, options) => {
            let nested = scalar_expression(acorn, inner, value)?;
            let error = error_tokens(acorn, "not", options, quote! { "Value satisfies a forbidden validation rule" });
            Ok(quote! {{
                if (#nested).is_err() {
                    ::core::result::Result::Ok(())
                } else {
                    ::core::result::Result::Err(#error)
                }
            }})
        }
        | ValidationRule::Or(alternatives, options) => {
            let expressions = alternatives
                .iter()
                .map(|alternative| scalar_expression(acorn, alternative, value.clone()))
                .collect::<syn::Result<Vec<_>>>()?;
            let error = error_tokens(acorn, "or", options, quote! { "value matches none of the given rules" });
            Ok(quote! {{
                if false #(|| (#expressions).is_ok())* {
                    ::core::result::Result::Ok(())
                } else {
                    ::core::result::Result::Err(#error)
                }
            }})
        }
        | _ => Err(syn::Error::new(proc_macro2::Span::call_site(), "rule is not scalar")),
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
    use crate::command::{parse_cmd_words, CmdWord};
    use syn::{parse_quote, DeriveInput, Type};

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
        assert!(tokens.contains(":: acorn :: validation"));
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
}
