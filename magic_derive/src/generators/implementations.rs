use crate::{attrs::FKConfig, model::ModelInfo};
use crate::generators::files::{ColumnJsonMeta, ForeignKeyJsonMeta, write_model_json};
use crate::generators::utils::model_meta::{map_rust_to_sqlite, is_option};
use quote::{quote};

pub fn generate_from_row_impl(struct_name: &syn::Ident, model: &ModelInfo) -> proc_macro2::TokenStream {
    let id_ident = &model.id_field.ident;
    let id_name = id_ident.to_string();

    let other_idents: Vec<_> = model.other_fields.iter().map(|f| &f.ident).collect();
    let other_names: Vec<_> = other_idents.iter().map(|i| i.to_string()).collect();

    quote! {
        impl sqlx::FromRow<'_, sqlx::sqlite::SqliteRow> for #struct_name {
            fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
                use sqlx::Row;
                Ok(Self {
                    #id_ident: row.try_get(#id_name)?,
                    #(
                        #other_idents: row.try_get(#other_names)?,
                    )*
                })
            }
        }
    }
}

pub fn generate_model_impl(struct_name: &syn::Ident, model: &ModelInfo) -> proc_macro2::TokenStream {
    let id_type = &model.id_field.ty;

    quote! {
        impl ::magic_orm::traits::Model for #struct_name {
            type Id = #id_type;

            fn id_column() -> &'static str {
                "id"
            }
            
            fn query<'a>() -> ::magic_orm::query::QueryBuilder<'a, Self> {
                ::magic_orm::query::QueryBuilder::new(Self::TABLE)
            }

            fn id(&self) -> &Self::Id { &self.id }
        }   
    }
}

pub fn generate_model_meta_impl(
    struct_name: &syn::Ident,
    fk_fields: &[FKConfig],
    model: &ModelInfo,
    table_name: &str,
) -> proc_macro2::TokenStream {
    // --- Preparar metadata ---
    let columns_meta_tokens = std::iter::once(&model.id_field)
        .chain(model.other_fields.iter())
        .map(|f| {
            let name = f.ident.to_string();
            let sql_type = map_rust_to_sqlite(&f.ty);
            let nullable = is_option(&f.ty);
            let is_pk = f.ident == model.id_field.ident;
            quote! {
                ::magic_orm::meta::ColumnMeta {
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
            ::magic_orm::meta::ForeignKeyMeta {
                field: #field_name,
                related_table: <#related_model as ::magic_orm::meta::ModelMeta>::TABLE,
                related_column: #related_column,
            }
        }
    });

    // --- Preparar para JSON ---
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

    // --- Llamar a la función auxiliar ---
    write_model_json(table_name, columns_json, fk_json);

    // --- Código del impl original ---
    quote! {
        impl ::magic_orm::meta::ModelMeta for #struct_name {
            const TABLE: &'static str = #table_name;

            fn columns() -> &'static [::magic_orm::meta::ColumnMeta] {
                static COLUMNS: &[::magic_orm::meta::ColumnMeta] = &[
                    #( #columns_meta_tokens, )*
                ];
                COLUMNS
            }

            fn foreign_keys() -> &'static [::magic_orm::meta::ForeignKeyMeta] {
                static FK_META: &[::magic_orm::meta::ForeignKeyMeta] = &[
                    #( #fk_meta_tokens, )*
                ];
                FK_META
            }
        }
    }
}

pub fn generate_hasfk_impl(fk_fields: &[crate::attrs::FKConfig], struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let fk_impls = fk_fields.iter().map(|fk| {
        let parent = &fk.model;
        let column_name = &fk.field_ident.to_string();

        quote! {
            impl magic_orm::relations::traits::HasFK<#parent> for #struct_name {
                fn fk_for_parent() -> &'static str {
                    #column_name
                }
            }
        }
    });

    quote! {
        #( #fk_impls )*
    }
}