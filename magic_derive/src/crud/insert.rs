use quote::quote;
use syn::Ident;
use crate::model::ModelInfo;

pub fn generate_insert(struct_name: &Ident, model: &ModelInfo, table_name: &str) -> proc_macro2::TokenStream {
    // Convertir cada campo a Ident
    let idents: Vec<Ident> = model.other_fields.iter()
        .map(|f| f.ident.clone())
        .collect();

    // Nombres como &str para la SQL
    let column_names: Vec<String> = idents.iter().map(|i| i.to_string()).collect();

    quote! {
        pub async fn insert(&self, pool: &sqlx::SqlitePool) -> sqlx::Result<i64> {
            let cols = &[ #( #column_names ),* ];
            let placeholders = vec!["?"; cols.len()].join(", ");
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                #table_name,
                cols.join(", "),
                placeholders
            );

            let result = sqlx::query(&sql)
                #( .bind(&self.#idents) )*
                .execute(pool)
                .await?;

            Ok(result.last_insert_rowid())
        }
    }
}