use quote::{quote, format_ident};
use syn::DeriveInput;

use crate::attrs::{FKConfig, MagicConfig};
use crate::model::ModelInfo;

pub use crate::generators::implementations::*;
pub use crate::generators::methods::*;

pub fn expand_magic_model(
    input: &DeriveInput,
    config: MagicConfig,
    model: ModelInfo,
    fk_fields: &[FKConfig]
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

    let crud_methods = generate_crud_methods(struct_name, &model, &table_name);
    let newstruct_methods = generate_newstruct_methods(struct_name);

    let from_row_impl = generate_from_row_impl(struct_name, &model);
    let model_meta_impl = generate_model_meta_impl(struct_name, fk_fields, &model, &table_name);
    let model_impl = generate_model_impl(struct_name, &model);
    let hasfk_impl = generate_hasfk_impl(fk_fields, struct_name);
    

    quote! {
        #vis struct #new_struct_name {
            #( #new_fields, )*
        }

        
        #from_row_impl
        #model_impl

        impl #struct_name {
            pub const TABLE: &'static str = #table_name;

            pub fn new( #( #new_params ),* ) -> #new_struct_name {
                #new_struct_name {
                    #( #new_inits, )*
                }
            }

            #crud_methods

        }
        #newstruct_methods

        #model_meta_impl
        #hasfk_impl
    }
}



