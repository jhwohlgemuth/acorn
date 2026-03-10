//! Procedural macros for ACORN database utilities
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

/// Generates field list fragments for use with the `build_query!` macro.
///
/// Attributes:
/// - `#[row(order_by = "...")]` – optional ORDER BY clause
///
/// This derive exposes constants `ORDER_BY` and `FIELDS` that list struct field names
/// with proper type conversions (e.g., bool as i32).
///
/// Example:
/// ```ignore
/// #[derive(DatabaseRow)]
/// #[row(order_by = "executed_at DESC")]
/// pub struct ActivityRow { ... }
///
/// // Use in select method:
/// let (query, params) = build_query!(self, base_query, Self::ORDER_BY, [
///     id, command, user_path, success => |value| value as i32,
/// ]);
/// ```
#[proc_macro_derive(DatabaseRow, attributes(row))]
pub fn derive_database_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let _name = &input.ident;
    let order_by = extract_order_by(&input.attrs);
    let fields = match &input.data {
        | Data::Struct(data) => match &data.fields {
            | Fields::Named(fields) => &fields.named,
            | _ => {
                return syn::Error::new_spanned(&input, "DatabaseRow only works with named fields")
                    .to_compile_error()
                    .into()
            }
        },
        | _ => {
            return syn::Error::new_spanned(&input, "DatabaseRow only works with structs")
                .to_compile_error()
                .into()
        }
    };
    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap().to_string()).collect();
    let order_by_expr = order_by
        .as_ref()
        .map(|ob| quote! { Some(#ob) })
        .unwrap_or_else(|| quote! { None::<&str> });
    let expanded = quote! {
        impl #_name {
            /// ORDER BY clause for queries on this row type.
            pub const ORDER_BY: Option<&'static str> = #order_by_expr;
            /// Field list for build_query! macro documentation purposes.
            #[allow(dead_code)]
            pub const FIELDS: &'static [&'static str] = &[#(#field_names),*];
        }
    };
    TokenStream::from(expanded)
}

fn extract_order_by(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("row") {
            if let Meta::List(meta_list) = &attr.meta {
                if let Ok(parsed) = meta_list.parse_args::<syn::MetaNameValue>() {
                    if parsed.path.is_ident("order_by") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str), ..
                        }) = &parsed.value
                        {
                            return Some(lit_str.value());
                        }
                    }
                }
            }
        }
    }
    None
}
