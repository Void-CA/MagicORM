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

pub fn generate_delete_by_id(table_name: &str) -> proc_macro2::TokenStream {
    quote! {
        pub async fn delete_by_id<'e, E>(executor: E, id: i64) -> sqlx::Result<usize>
        where
            E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
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
           