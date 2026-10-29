use acorn_core::big_o::Expression;
use quote::quote;
use syn::LitStr;

pub(crate) fn expand_bigo(input: &LitStr) -> syn::Result<proc_macro2::TokenStream> {
    Expression::parse(&input.value())
        .map(|_| quote! { #input })
        .map_err(|why| syn::Error::new(input.span(), format!("invalid Big-O notation: {why}")))
}
