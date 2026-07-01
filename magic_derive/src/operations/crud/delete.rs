use crate::input::ModelInfo;
use quote::{quote};

pub fn generate_delete(table_name: &str) -> proc_macro2::TokenStream {
    quote! {
        pub async fn delete_all<'e, E>(executor: E) -> sqlx::Result<usize>
        where
            E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
        {
            let sql = format!(
                "DELETE FROM {}",
                #table_name
            );

            let rows = sqlx::query(&sql)
                .execute(executor)
                .await?;

            Ok(rows.rows_affected().try_into().map_err(|_| {
                sqlx::Error::Protocol("rows_affected overflowed i64".into())
            })?)
        }
    }
}

pub fn generate_delete_by_id(struct_name: &syn::Ident, model: &ModelInfo, table_name: &str) -> proc_macro2::TokenStream {
    let id_type = &model.id_field.ty;
    quote! {
        pub async fn delete_by_id<'e, E>(executor: E, id: #id_type) -> sqlx::Result<usize>
        where
            E: sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
        {
            let sql = format!("DELETE FROM {} WHERE id = ?", #table_name);
            let rows = sqlx::query(&sql)
                .bind(id)
                .execute(executor)
                .await?;
            Ok(rows.rows_affected().try_into().map_err(|_| {
                sqlx::Error::Protocol("rows_affected overflowed i64".into())
            })?)
        }
    }
}
           