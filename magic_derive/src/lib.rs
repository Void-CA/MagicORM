extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(MagicModel)]
pub fn derive_magic_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;

    // Generar nombre NewStruct
    let new_struct_name = format_ident!("New{}", struct_name);

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => panic!("MagicModel only works on structs"),
    };

    let named_fields = match fields {
        Fields::Named(fields) => fields.named,
        _ => panic!("MagicModel requires named fields"),
    };

    // Separar campo id del resto
    let mut new_fields = Vec::new();
    let mut new_params = Vec::new();
    let mut new_inits = Vec::new();

    for field in named_fields {
        let field_name = field.ident.unwrap();
        let field_type = field.ty;

        if field_name == "id" {
            continue;
        }

        new_fields.push(quote! {
            pub #field_name: #field_type
        });

        new_params.push(quote! {
            #field_name: #field_type
        });

        new_inits.push(quote! {
            #field_name
        });
    }

    let expanded = quote! {
        pub struct #new_struct_name {
            #( #new_fields, )*
        }

        impl #struct_name {
            pub fn new( #( #new_params ),* ) -> #new_struct_name {
                #new_struct_name {
                    #( #new_inits, )*
                }
            }
        }
    };

    TokenStream::from(expanded)
}