use std::collections::HashSet;

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

    let unique_tys: HashSet<_> = tys.iter().collect();
    let mut trait_impls = Vec::new();
    for unique_ty in unique_tys {
        let mut getter_arms = Vec::new();
        let mut setter_arms = Vec::new();

        for i in 0..fields.len() {
            let field = fields[i];
            let ty = tys[i];
            if ty == *unique_ty {
                let getter_arm = quote! {
                    stringify!(#field) => cvars.#field,
                };
                getter_arms.push(getter_arm);

                let setter_arm = quote! {
                    stringify!(#field) => cvars.#field = value,
                };
                setter_arms.push(setter_arm);
            }
        }

        // TODO Better error messages (report type of value and of field if both exist)
        // TODO Is there a sane way to automatically convert?
        let trait_impl = quote! {
            impl CvarValue for #unique_ty {
                fn get(cvars: &Cvars, cvar_name: &str) -> Self {
                    match cvar_name {
                        #( #getter_arms )*
                        _ => panic!("Cvar named {} with type {} not found", cvar_name, stringify!(#unique_ty)),
                    }
                }

                fn set(cvars: &mut Cvars, cvar_name: &str, value: Self) {
                    match cvar_name {
                        #( #setter_arms )*
                        _ => panic!("Cvar named {} with type {} not found", cvar_name, stringify!(#unique_ty)),
                    }
                }
            }
        };
        trait_impls.push(trait_impl);
    }

    let expanded = quote! {
        impl #struct_name {
            fn get<T: CvarValue>(&self, cvar_name: &str) -> T {
                CvarValue::get(self, cvar_name)
            }

            fn set<T: CvarValue>(&mut self, cvar_name: &str, value: T) {
                CvarValue::set(self, cvar_name, value);
            }
        }

        trait CvarValue {
            fn get(cvars: &Cvars, cvar_name: &str) -> Self;
            fn set(cvars: &mut Cvars, cvar_name: &str, value: Self);
        }

        #( #trait_impls )*
    };
    TokenStream::from(expanded)
}
