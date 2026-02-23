use crate::{attrs::FKConfig, model::ModelInfo, operations::crud::{
    generate_delete, generate_delete_by_id, generate_get, generate_get_by_id, generate_insert, generate_newstruct_insert, generate_newstruct_put, generate_put
}};

use crate::operations::relations::belongs_to::generate_belongs_to;
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
    
pub fn generate_relations_methods(struct_name: &syn::Ident, fk_fields: &[FKConfig]) -> proc_macro2::TokenStream {
    let has_many_methods = fk_fields.iter().map(|fk| {
        let related_model = &fk.model;
        let fk_column = &fk.column;
        crate::operations::relations::has_many::generate_has_many(struct_name, related_model, fk_column)
    });

    let belongs_to_methods = fk_fields.iter().map(|fk| {
        let related_model = &fk.model;
        let fk_field = &fk.field_ident;
        generate_belongs_to(struct_name, related_model, fk_field)
    });

    quote! {
        #( #has_many_methods )*
        #( #belongs_to_methods )*
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