use quote::{format_ident, quote};
use syn::Ident;
use crate::model::ModelInfo;

pub fn generate_insert(struct_name: &Ident, model: &ModelInfo, table_name: &str) -> proc_macro2::TokenStream {
    let idents: Vec<Ident> = model.other_fields.iter().map(|f| f.ident.clone()).collect();
    let column_names: Vec<String> = model.no_id_column_names();
    let new_struct_name = format_ident!("New{}", struct_name);

    quote! {
        pub async fn insert(pool: &sqlx::SqlitePool, new: &#new_struct_name) -> sqlx::Result<i64> {
            let cols = &[ #( #column_names ),* ];
            let placeholders = vec!["?"; cols.len()].join(", ");
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                #table_name,
                cols.join(", "),
                placeholders
            );

            let mut query = sqlx::query(&sql);
            #( query = query.bind(&new.#idents); )*
            let result = query.execute(pool).await?;

            Ok(result.last_insert_rowid() as i64)
        }
    }
}

pub fn generate_newstruct_insert(struct_name: &Ident) -> proc_macro2::TokenStream {
    let new_struct_name = format_ident!("New{}", struct_name);
    quote! {
        impl #new_struct_name {
            pub async fn insert(&self, pool: &sqlx::SqlitePool) -> sqlx::Result<i64> {
                #struct_name::insert(pool, self).await
            }
        }
    }
}