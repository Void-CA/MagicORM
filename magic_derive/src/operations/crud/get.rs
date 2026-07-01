use crate::input::ModelInfo;
use proc_macro2::Literal;
use quote::quote;
use syn::{Ident, LitStr};

pub fn generate_get(
    struct_name: &Ident,
    model: &ModelInfo,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let all_columns: Vec<String> = model.column_names();
    let all_columns_literal = LitStr::new(&all_columns.join(", "), proc_macro2::Span::call_site());

    quote! {
        pub async fn get_all<'e, E>(executor: E) -> ::anyhow::Result<Vec<#struct_name>>
        where
            E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
        {
            ::magic_orm::crud::get_all::<#struct_name>(
                executor,
                #all_columns_literal,
                #table_name,
            ).await
        }
    }
}

pub fn generate_get_by_id(
    struct_name: &Ident,
    model: &ModelInfo,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let all_columns: Vec<String> = model.column_names();
    let all_columns_literal = Literal::string(&all_columns.join(", "));
    let id_type = &model.id_field.ty;

    quote! {
        pub async fn get_by_id<'e, E>(executor: E, id: #id_type) -> ::anyhow::Result<Option<#struct_name>>
        where
            E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
        {
            ::magic_orm::crud::get_by_id::<#struct_name>(
                executor,
                #all_columns_literal,
                #table_name,
                id,
            ).await
        }
    }
}
