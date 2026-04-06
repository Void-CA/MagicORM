use crate::input::attrs::FKConfig;
use crate::input::ModelInfo;
use crate::codegen::utils::{map_rust_to_sqlite, is_option, write_model_json, ColumnJsonMeta, ForeignKeyJsonMeta};
use quote::quote;

pub fn generate_model_meta_impl(
    struct_name: &syn::Ident,
    fk_fields: &[FKConfig],
    model: &ModelInfo,
    table_name: &str,
) -> proc_macro2::TokenStream {
    // metadata tokens
    let columns_meta_tokens = std::iter::once(&model.id_field)
        .chain(model.other_fields.iter())
        .map(|f| {
            let name = f.ident.to_string();
            let sql_type = map_rust_to_sqlite(&f.ty);
            let nullable = is_option(&f.ty);
            let is_pk = f.ident == model.id_field.ident;
            quote! {
                ::magic_orm::model::ColumnMeta {
                    name: #name,
                    sql_type: #sql_type,
                    nullable: #nullable,
                    primary_key: #is_pk,
                }
            }
        });

    let fk_meta_tokens = fk_fields.iter().map(|fk| {
        let field_name = fk.field_ident.to_string();
        let related_model = &fk.model;
        let related_column = &fk.column;

        quote! {
            ::magic_orm::model::ForeignKeyMeta {
                field: #field_name,
                related_table: <#related_model as ::magic_orm::model::ModelMeta>::TABLE,
                related_column: #related_column,
            }
        }
    });

    // JSON metadata
    let columns_json: Vec<ColumnJsonMeta> = std::iter::once(&model.id_field)
        .chain(model.other_fields.iter())
        .map(|f| ColumnJsonMeta {
            name: f.ident.to_string(),
            sql_type: map_rust_to_sqlite(&f.ty).to_owned(),
            nullable: is_option(&f.ty),
            primary_key: f.ident == model.id_field.ident,
        })
        .collect();

    let fk_json: Vec<ForeignKeyJsonMeta> = fk_fields
        .iter()
        .map(|fk| ForeignKeyJsonMeta {
            field: fk.field_ident.to_string(),
            related_table: fk.model.to_string(),
            related_column: fk.column.clone(),
        })
        .collect();

    write_model_json(table_name, columns_json, fk_json);

    quote! {
        impl ::magic_orm::model::ModelMeta for #struct_name {
            const TABLE: &'static str = #table_name;

            fn columns() -> &'static [::magic_orm::model::ColumnMeta] {
                static COLUMNS: &[::magic_orm::model::ColumnMeta] = &[
                    #( #columns_meta_tokens, )*
                ];
                COLUMNS
            }

            fn foreign_keys() -> &'static [::magic_orm::model::ForeignKeyMeta] {
                static FK_META: &[::magic_orm::model::ForeignKeyMeta] = &[
                    #( #fk_meta_tokens, )*
                ];
                FK_META
            }
        }
    }
}
