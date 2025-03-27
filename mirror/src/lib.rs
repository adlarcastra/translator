use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// pub trait MirrorTrait {}

#[proc_macro_derive(Mirror)]
pub fn mirror_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // Collect field idents
    let fields = if let syn::Data::Struct(ref data_struct) = input.data {
        data_struct
            .fields
            .iter()
            .filter_map(|f| f.ident.as_ref())
            .collect::<Vec<_>>()
    } else {
        panic!("Mirror can only be derived for structs");
    };

    // Generate string literals for field names.
    let field_names = fields.iter().map(|field| quote! { stringify!(#field) });

    // Generate match arms for the `get` method.
    let get_match_arms = fields.iter().map(|field| {
        quote! {
            stringify!(#field) => Some(&self.#field as &dyn std::any::Any),
        }
    });

    // Generate match arms for the `set` method.
    let set_match_arms = fields.iter().map(|field| {
        quote! {
            stringify!(#field) => {
                if let Some(field_ref) = (&mut self.#field as &mut dyn std::any::Any)
                    .downcast_mut::<T>()
                {
                    *field_ref = new_value;
                    Some(())
                } else {
                    None
                }
            },
        }
    });

    let expanded = quote! {
        impl MirrorTrait for #struct_name {
            fn field_names(&self) -> &'static [&'static str] {
                &[#(#field_names),*]
            }

            fn get<T: std::any::Any>(&self, field: &str) -> Option<&T> {
                match field {
                    #(#get_match_arms)*
                    _ => None,
                }.and_then(|val| val.downcast_ref::<T>())
            }

            fn set<T: std::any::Any>(&mut self, field: &str, new_value: T) -> Option<()> {
                match field {
                    #(#set_match_arms)*
                    _ => None,
                }
            }
        }
    };

    TokenStream::from(expanded)
}
 