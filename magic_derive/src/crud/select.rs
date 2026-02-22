use crate::model::ModelInfo;
use proc_macro2::Literal;
use quote::{quote};
use syn::{Ident, LitStr};

pub fn generate_select(struct_name: &Ident, model: &ModelInfo, table_name: &str) -> proc_macro2::TokenStream {
    let struct_name = struct_name;
    let all_columns: Vec<String> = model.column_names();
    let all_columns_literal = LitStr::new(&all_columns.join(", "), proc_macro2::Span::call_site());

    quote! {
        pub async fn select_all(pool: &sqlx::SqlitePool) -> sqlx::Result<Vec<#struct_name>> {
            let sql = format!(
                "SELECT {} FROM {}",
                #all_columns_literal,
                #table_name
            );

            let rows = sqlx::query_as::<_, #struct_name>(&sql)
                .fetch_all(pool)
                .await?;

            Ok(rows)
        }
    }
}

pub fn generate_select_by_id(struct_name: &Ident, model: &ModelInfo, table_name: &str) -> proc_macro2::TokenStream {
    let all_columns: Vec<String> = model.column_names();
    let all_columns_literal = Literal::string(&all_columns.join(", "));

    quote! {
        pub async fn select_by_id(pool: &sqlx::SqlitePool, id: i64) -> sqlx::Result<Option<#struct_name>> {
            let sql = format!("SELECT {} FROM {} WHERE id = ?", 
                #all_columns_literal, 
                #table_name
            );
            let row = sqlx::query_as::<_, #struct_name>(&sql)
                .bind(id)
                .fetch_optional(pool)
                .await?;
            Ok(row)
        }
    }
}