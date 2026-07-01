use crate::input::ModelInfo;
use quote::quote;

pub fn generate_delete(table_name: &str) -> proc_macro2::TokenStream {
    quote! {
        pub async fn delete_all<'e, E>(executor: E) -> ::anyhow::Result<u64>
        where
            E: ::sqlx::Executor<'e, Database = ::magic_orm::db::DefaultDB>,
        {
            ::magic_orm::crud::delete_all(
                executor,
                #table_name,
            ).await
        }
    }
}

pub fn generate_delete_by_id(
    struct_name: &syn::Ident,
    model: &ModelInfo,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let id_type = &model.id_field.ty;
    quote! {
        pub async fn delete_by_id<'e, E>(executor: E, id: #id_type) -> ::anyhow::Result<usize>
        where
            E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
        {
            let affected = ::magic_orm::crud::delete_by_id::<#struct_name>(
                executor,
                #table_name,
                id,
            ).await?;
            Ok(affected as usize)
        }
    }
}
