use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(Seam)]
pub fn derive_seam(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = &input.ident;
    quote! {
        impl sutures::Seam for #name {}
    }
    .into()
}
