use crate::input::ModelInfo;
use quote::{format_ident, quote};
use syn::Ident;

pub fn generate_insert(
    struct_name: &Ident,
    model: &ModelInfo,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let idents: Vec<Ident> = model.other_fields.iter().map(|f| f.ident.clone()).collect();
    let column_names: Vec<String> = model.no_id_column_names();
    let new_struct_name = format_ident!("New{}", struct_name);

    quote! {
        pub async fn insert<'e, E>(
            executor: E,
            new: &#new_struct_name
        ) -> ::anyhow::Result<i64>
        where
            E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
        {
            let values = vec![
                #( ::magic_orm::query::statement::BindArg::from(&new.#idents), )*
            ];
            ::magic_orm::crud::insert::<#struct_name>(
                executor,
                #table_name,
                &[ #( #column_names ),* ],
                values,
            ).await
        }
    }
}

pub fn generate_newstruct_insert(struct_name: &Ident) -> proc_macro2::TokenStream {
    let new_struct_name = format_ident!("New{}", struct_name);
    quote! {
        impl #new_struct_name {
            pub async fn insert<'e, E>(
                &self,
                executor: E
            ) -> ::anyhow::Result<i64>
            where
                E: ::sqlx::Executor<'e, Database = <#struct_name as ::magic_orm::model::Model>::DB>,
            {
                #struct_name::insert(executor, self).await
            }
        }
    }
}
