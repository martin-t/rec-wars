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
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // TODO error msg
    let named_fields = match input.data {
        syn::Data::Struct(struct_data) => match struct_data.fields {
            syn::Fields::Named(named_fields) => named_fields,
            syn::Fields::Unnamed(_) => unimplemented!(),
            syn::Fields::Unit => unimplemented!(),
        },
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Union(_) => unimplemented!(),
    };

    let mut fields = Vec::new();
    let mut tys = Vec::new();
    for field in &named_fields.named {
        let ident = field.ident.as_ref().unwrap();
        fields.push(ident);
        tys.push(&field.ty);
    }

    let expanded = quote! {
        impl #struct_name {
            fn get(&self, cvar_name: &str) -> i32 {
                match cvar_name{
                    #(
                        stringify!(#fields) => self.#fields,
                    )*
                    other => panic!("Unknown cvar {}", other),
                }
            }
        }
    };
    TokenStream::from(expanded)
}
