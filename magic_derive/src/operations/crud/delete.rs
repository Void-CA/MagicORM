use quote::{quote};

pub fn generate_delete(table_name: &str) -> proc_macro2::TokenStream {
    quote! {
        pub async fn delete_all(pool: &sqlx::SqlitePool) -> sqlx::Result<usize> {
            let sql = format!(
                "DELETE FROM {}",
                #table_name
            );

            let rows = sqlx::query(&sql)
                .execute(pool)
                .await?;

            Ok(rows.rows_affected().try_into().map_err(|_| {
                sqlx::Error::Protocol("rows_affected overflowed i64".into())
            })?)
        }
    }
}

pub fn generate_delete_by_id(table_name: &str) -> proc_macro2::TokenStream {
    quote! {
        pub async fn delete_by_id(pool: &sqlx::SqlitePool, id: i64) -> sqlx::Result<usize> {
            let sql = format!("DELETE FROM {} WHERE id = ?", #table_name);
            let rows = sqlx::query(&sql)
                .bind(id)
                .execute(pool)
                .await?;
            Ok(rows.rows_affected().try_into().map_err(|_| {
                sqlx::Error::Protocol("rows_affected overflowed i64".into())
            })?)
        }
    }
}
           