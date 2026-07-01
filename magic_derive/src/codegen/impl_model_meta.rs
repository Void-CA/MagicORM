use crate::input::attrs::FKConfig;
use crate::input::ModelInfo;
use crate::codegen::utils::{map_rust_to_sqlite, is_option};
use quote::quote;

pub fn generate_model_meta_impl(
    struct_name: &syn::Ident,
    fk_fields: &[FKConfig],
    model: &ModelInfo,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let columns_tokens: Vec<_> = std::iter::once(&model.id_field)
        .chain(model.other_fields.iter())
        .map(|f| {
            let name = f.ident.to_string();
            let sql_type = map_rust_to_sqlite(&f.ty);
            let nullable = is_option(&f.ty);
            let is_pk = f.ident == model.id_field.ident;
            let auto_inc = is_pk;
            quote! {
                ::magic_orm::model::ColumnMeta {
                    name: #name.to_string(),
                    sql_type: #sql_type.to_string(),
                    nullable: #nullable,
                    primary_key: #is_pk,
                    auto_increment: #auto_inc,
                }
            }
        }).collect();

    let fk_tokens: Vec<_> = fk_fields.iter().map(|fk| {
        let field_name = fk.field_ident.to_string();
        let related_model = &fk.model;
        let related_column = &fk.column;

        quote! {
            ::magic_orm::model::ForeignKeyMeta {
                field: #field_name.to_string(),
                related_table: <#related_model as ::magic_orm::model::ModelMeta>::TABLE.to_string(),
                related_column: #related_column.to_string(),
            }
        }
    }).collect();

    quote! {
        impl ::magic_orm::model::ModelMeta for #struct_name {
            const TABLE: &'static str = #table_name;

            fn columns() -> Vec<::magic_orm::model::ColumnMeta> {
                vec![
                    #( #columns_tokens, )*
                ]
            }

            fn foreign_keys() -> Vec<::magic_orm::model::ForeignKeyMeta> {
                vec![
                    #( #fk_tokens, )*
                ]
            }
        }
    }
}
