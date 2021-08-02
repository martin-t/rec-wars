use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// TODO alternatives:
//  https://crates.io/crates/tuna
//  https://crates.io/crates/cvar
//  https://crates.io/crates/const-tweaker
//  https://crates.io/crates/inline_tweak

#[proc_macro_derive(Cvars)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as DeriveInput);
    let expanded = quote! {};
    TokenStream::from(expanded)
}
