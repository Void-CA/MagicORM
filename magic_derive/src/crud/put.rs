use quote::{format_ident, quote};
use syn::Ident;
use crate::model::ModelInfo;

pub fn generate_put(struct_name: &Ident, model: &ModelInfo, table_name: &str) -> proc_macro2::TokenStream {
    let idents: Vec<Ident> = model.other_fields.iter().map(|f| f.ident.clone()).collect();
    let column_names: Vec<String> = model.no_id_column_names();
    let new_struct_name = format_ident!("New{}", struct_name);

    quote! {
        pub async fn put(pool: &SqlitePool, id: i64, new: &#new_struct_name) -> sqlx::Result<i64> {
            let cols = &[ #( #column_names ),* ];
            let sql = format!(
                "UPDATE {} SET {} WHERE id = ?",
                #table_name,
                cols.iter().map(|c| format!("{} = ?", c)).collect::<Vec<_>>().join(", ")
            );

            let mut query = sqlx::query(&sql);
            #( query = query.bind(&new.#idents); )*
            query = query.bind(id);
            let result = query.execute(pool).await?;
            Ok(result.rows_affected().try_into().map_err(|_| {
                sqlx::Error::Protocol("rows_affected overflowed i64".into())
            })?)
        }
    }
}
                
pub fn generate_newstruct_put(struct_name: &Ident) -> proc_macro2::TokenStream {
    let new_struct_name = format_ident!("New{}", struct_name);
    quote! {
        impl #new_struct_name {
            pub async fn put(&self, pool: &sqlx::SqlitePool, id: i64) -> sqlx::Result<i64> {
                #struct_name::put(pool, id, self).await
            }
        }
    }
}