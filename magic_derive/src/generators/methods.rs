use crate::{model::ModelInfo, operations::crud::{
    generate_delete, generate_delete_by_id, generate_get, generate_get_by_id, generate_insert, generate_newstruct_insert, generate_newstruct_put, generate_put
}};

use quote::quote;

pub fn generate_crud_methods(struct_name: &syn::Ident, model: &ModelInfo, table_name: &str) -> proc_macro2::TokenStream {
    let insert_method = generate_insert(struct_name, model, table_name);
    let update_method = generate_put(struct_name, model, table_name);
    let select_method = generate_get(struct_name, model, table_name);
    let select_by_id_method = generate_get_by_id(struct_name, model, table_name);
    let delete_method = generate_delete(table_name);
    let delete_by_id_method = generate_delete_by_id(table_name);
    
    quote! {
        #insert_method
        #update_method
        #select_method
        #select_by_id_method
        #delete_method
        #delete_by_id_method
    }
}
    
pub fn generate_newstruct_methods(struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let newstruct_insert = generate_newstruct_insert(struct_name);
    let newstruct_put = generate_newstruct_put(struct_name);

    quote! {
        #newstruct_insert
        #newstruct_put
    }
}

pub fn generate_fk_methods(fk_fields: &[crate::attrs::FKConfig]) -> proc_macro2::TokenStream {
    let fk_methods = fk_fields.iter().map(|fk| {
        let parent = &fk.model;         // nombre del padre, ej: User
        let column_name = &fk.column;   // nombre de la columna, ej: "user_id"
        let method_name = quote::format_ident!("fk_for_{}", parent.to_string().to_lowercase());

        quote! {
            pub fn #method_name<P>() -> &'static str {
                #column_name
            }
        }
    });
    quote! {
        #( #fk_methods )*
    }
}