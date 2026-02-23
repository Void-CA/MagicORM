use crate::{attrs::FKConfig, model::ModelInfo};
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
        impl ::magic::traits::Model for #struct_name {
            type Id = #id_type;

            fn table_name() -> &'static str {
                Self::TABLE
            }

            fn columns() -> &'static [&'static str] {
                Self::columns()
            }

            fn id_column() -> &'static str {
                "id"
            }
            
            fn query<'a>() -> ::magic::query::QueryBuilder<'a, Self> {
                ::magic::query::QueryBuilder::new(Self::TABLE)
            }

            fn id(&self) -> &Self::Id { &self.id }
        }   
    }
}

pub fn generate_model_meta_impl(
    struct_name: &syn::Ident,
    fk_fields: &[FKConfig],
) -> proc_macro2::TokenStream {
    let fk_meta = fk_fields.iter().map(|fk| {
        let field_name = fk.field_ident.to_string();
        let related_model = &fk.model;
        let related_column = &fk.column;

        quote! {
            ::magic::meta::ForeignKeyMeta {
                field: #field_name,
                related_table: <#related_model as ::magic::meta::ModelMeta>::TABLE,
                related_column: #related_column,
            }
        }
    });

    quote! {
        impl ::magic::meta::ModelMeta for #struct_name {
            const TABLE: &'static str = Self::TABLE;

            fn foreign_keys() -> &'static [::magic::meta::ForeignKeyMeta] {
                static FK_META: &[::magic::meta::ForeignKeyMeta] = &[
                    #( #fk_meta, )*
                ];
                FK_META
            }
        }

        impl #struct_name {
            pub fn table() -> &'static str {
                <Self as ::magic::meta::ModelMeta>::TABLE
            }

            pub fn foreign_keys() -> &'static [::magic::meta::ForeignKeyMeta] {
                <Self as ::magic::meta::ModelMeta>::foreign_keys()
            }
        }
    }
}

           