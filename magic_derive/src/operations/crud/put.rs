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
    let id_type = &model.id_field.ty;

    quote! {
        pub async fn put<'e, E>(
            executor: E,
            id: #id_type,
            new: &#new_struct_name
        ) -> ::anyhow::Result<i64>
        where
            E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
        {
            let values = vec![
                #( ::magic_orm::query::statement::BindArg::from(&new.#idents), )*
            ];
            let affected = ::magic_orm::crud::put::<#struct_name>(
                executor,
                #table_name,
                &[ #( #column_names ),* ],
                values,
                id,
            ).await?;
            Ok(affected as i64)
        }
    }
}

pub fn generate_newstruct_put(struct_name: &Ident, model: &ModelInfo) -> proc_macro2::TokenStream {
    let new_struct_name = format_ident!("New{}", struct_name);
    let id_type = &model.id_field.ty;
    quote! {
        impl #new_struct_name {
            pub async fn put<'e, E>(&self, executor: E, id: #id_type) -> ::anyhow::Result<i64>
            where
                E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
            {
                #struct_name::put(executor, id, self).await
            }
        }
    }
}
