use quote::{quote, format_ident};
use syn::DeriveInput;

use crate::attributes::MagicConfig;
use crate::model::ModelInfo;

pub fn expand_magic_model(
    input: &DeriveInput,
    config: MagicConfig,
    model: ModelInfo,
) -> proc_macro2::TokenStream {
    let struct_name = &input.ident;
    let vis = &input.vis;
    let new_struct_name = format_ident!("New{}", struct_name);

    let table_name = config.table;

    // Campos para NewStruct
    let new_fields = model.other_fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! { pub #ident: #ty }
    });

    let new_params = model.other_fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! { #ident: #ty }
    });

    let new_inits = model.other_fields.iter().map(|f| {
        let ident = &f.ident;
        quote! { #ident }
    });

    // Columnas: id primero
    let column_names: Vec<String> = std::iter::once(model.id_field.ident.to_string())
        .chain(model.other_fields.iter().map(|f| f.ident.to_string()))
        .collect();

    quote! {
        #vis struct #new_struct_name {
            #( #new_fields, )*
        }

        impl #struct_name {
            pub const TABLE: &'static str = #table_name;

            pub fn new( #( #new_params ),* ) -> #new_struct_name {
                #new_struct_name {
                    #( #new_inits, )*
                }
            }

            pub fn columns() -> &'static [&'static str] {
                &[ #( #column_names ),* ]
            }
        }
    }
}