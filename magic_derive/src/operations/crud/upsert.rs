use crate::input::ModelInfo;
use quote::{format_ident, quote};
use syn::Ident;

pub fn generate_upsert(
    struct_name: &Ident,
    model: &ModelInfo,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let all_idents: Vec<Ident> = std::iter::once(model.id_field.ident.clone())
        .chain(model.other_fields.iter().map(|f| f.ident.clone()))
        .collect();
    let all_column_names: Vec<String> = model.column_names();
    let non_id_idents: Vec<Ident> = model.other_fields.iter().map(|f| f.ident.clone()).collect();
    let non_id_column_names: Vec<String> = model.no_id_column_names();
    let new_struct_name = format_ident!("New{}", struct_name);

    quote! {
        pub async fn upsert<'e, E>(
            executor: E,
            new: &#new_struct_name
        ) -> ::anyhow::Result<i64>
        where
            E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
        {
            let values = vec![
                #( ::magic_orm::query::statement::BindArg::from(&new.#non_id_idents), )*
            ];
            ::magic_orm::crud::upsert::<#struct_name>(
                executor,
                #table_name,
                &[ #( #non_id_column_names ),* ],
                values,
            ).await
        }

        pub async fn upsert_with_id<'e, E>(
            executor: E,
            entity: &#struct_name
        ) -> ::anyhow::Result<i64>
        where
            E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
        {
            let values = vec![
                #( ::magic_orm::query::statement::BindArg::from(&entity.#all_idents), )*
            ];
            ::magic_orm::crud::upsert_with_id::<#struct_name>(
                executor,
                #table_name,
                &[ #( #all_column_names ),* ],
                values,
            ).await
        }
    }
}

pub fn generate_newstruct_upsert(struct_name: &Ident) -> proc_macro2::TokenStream {
    let new_struct_name = format_ident!("New{}", struct_name);
    quote! {
        impl #new_struct_name {
            pub async fn upsert<'e, E>(
                &self,
                executor: E
            ) -> ::anyhow::Result<i64>
            where
                E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
            {
                #struct_name::upsert(executor, self).await
            }
        }
    }
}

pub fn generate_upsert_many(
    struct_name: &Ident,
    model: &ModelInfo,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let idents: Vec<Ident> = model.other_fields.iter().map(|f| f.ident.clone()).collect();
    let column_names: Vec<String> = model.no_id_column_names();
    let new_struct_name = format_ident!("New{}", struct_name);

    quote! {
        pub async fn upsert_many<'e, E>(
            executor: E,
            items: &[#new_struct_name],
        ) -> ::anyhow::Result<u64>
        where
            E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB> + ::std::marker::Copy,
        {
            let values_iter = items.iter().map(|new| {
                vec![
                    #( ::magic_orm::query::statement::BindArg::from(&new.#idents), )*
                ]
            });
            ::magic_orm::crud::upsert_many::<#struct_name, _>(
                executor,
                #table_name,
                &[ #( #column_names ),* ],
                values_iter,
            ).await
        }

        pub async fn upsert_many_in_tx<'e>(
            tx: &mut ::sqlx::Transaction<'e, <#struct_name as ::magic_orm::model::Model>::DB>,
            items: &[#new_struct_name],
        ) -> ::anyhow::Result<u64> {
            let values_iter = items.iter().map(|new| {
                vec![
                    #( ::magic_orm::query::statement::BindArg::from(&new.#idents), )*
                ]
            });
            ::magic_orm::crud::upsert_many_in_tx::<#struct_name, _>(
                tx,
                #table_name,
                &[ #( #column_names ),* ],
                values_iter,
            ).await
        }
    }
}
