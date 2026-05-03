use crate::input::ModelInfo;
use quote::{format_ident, quote};
use syn::Ident;

pub fn generate_put(
    struct_name: &Ident,
    model: &ModelInfo,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let idents: Vec<Ident> = model.other_fields.iter().map(|f| f.ident.clone()).collect();
    let column_names: Vec<String> = model.no_id_column_names();
    let new_struct_name = format_ident!("New{}", struct_name);

    // Resolvemos el tipo de ID del struct para la firma
    let id_type = &model.id_field.ty;
    quote! {
        pub async fn put<'e, E>(
            executor: E,
            id: #id_type,
            new: &#new_struct_name
        ) -> sqlx::Result<i64>
        where
            E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
        {
            let cols = &[ #( #column_names ),* ];
            let sql = format!(
                "UPDATE {} SET {} WHERE id = ?",
                #table_name,
                cols.iter().map(|c| format!("{} = ?", c)).collect::<Vec<_>>().join(", ")
            );

            let mut query = sqlx::query(&sql);
            #( query = query.bind(&new.#idents); )*
            query = query.bind(id);
            let result = query.execute(executor).await?;
            Ok(result.rows_affected().try_into().map_err(|_| {
                sqlx::Error::Protocol("rows_affected overflowed i64".into())
            })?)
        }
    }
}

pub fn generate_newstruct_put(struct_name: &Ident, model: &ModelInfo) -> proc_macro2::TokenStream {
    let new_struct_name = format_ident!("New{}", struct_name);
    let id_type = &model.id_field.ty;
    quote! {
        impl #new_struct_name {
            pub async fn put<'e, E>(&self, executor: E, id: #id_type) -> sqlx::Result<i64>
            where
                E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
            {
                #struct_name::put(executor, id, self).await
            }
        }
    }
}
