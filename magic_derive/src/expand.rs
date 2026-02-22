use quote::{quote, format_ident};
use syn::DeriveInput;

use crate::attrs::{FKConfig, MagicConfig};
use crate::model::{self, ModelInfo};

use crate::operations::crud::{
    generate_delete, generate_delete_by_id, 
    generate_insert, generate_newstruct_insert, 
    generate_put, generate_newstruct_put, 
    generate_get, generate_get_by_id
};

pub fn expand_magic_model(
    input: &DeriveInput,
    config: MagicConfig,
    model: ModelInfo,
    fk_fields: &[FKConfig]
) -> proc_macro2::TokenStream {
    let struct_name = &input.ident;
    let vis = &input.vis;
    let new_struct_name = format_ident!("New{}", struct_name);

    let table_name = config.table;

    // Campos para NewStruct
    let new_fields = model.other_fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! { pub #ident: #ty }
    });

    let new_params = model.other_fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        quote! { #ident: #ty }
    });

    let new_inits = model.other_fields.iter().map(|f| {
        let ident = &f.ident;
        quote! { #ident }
    });

    // Columnas: id primero
    let column_names: Vec<String> = std::iter::once(model.id_field.ident.to_string())
        .chain(model.other_fields.iter().map(|f| f.ident.to_string()))
        .collect();

    let crud_methods = generate_crud_methods(struct_name, &model, &table_name);
    let newstruct_methods = generate_newstruct_methods(struct_name);

    let from_row_impl = generate_from_row_impl(struct_name, &model);
    let model_meta_impl = generate_model_meta_impl(struct_name, fk_fields);
    quote! {
        #vis struct #new_struct_name {
            #( #new_fields, )*
        }

        
        #from_row_impl

        unsafe impl Send for #struct_name {}
        
        impl Unpin for #struct_name {}

        impl #struct_name {
            pub const TABLE: &'static str = #table_name;

            pub fn new( #( #new_params ),* ) -> #new_struct_name {
                #new_struct_name {
                    #( #new_inits, )*
                }
            }

            pub fn columns() -> &'static [&'static str] {
                &[ #( #column_names ),* ]
            }

            pub fn query<'a>() -> ::magic::query::QueryBuilder<'a, Self> {
                ::magic::query::QueryBuilder::new(Self::TABLE)
            }


            #crud_methods
        }

        #newstruct_methods

        #model_meta_impl
    }
}

fn generate_crud_methods(struct_name: &syn::Ident, model: &ModelInfo, table_name: &str) -> proc_macro2::TokenStream {
    let insert_method = generate_insert(struct_name, model, table_name);
    let update_method = generate_put(struct_name, model, table_name);
    let select_method = generate_get(struct_name, model, table_name);
    let select_by_id_method = generate_get_by_id(struct_name, model, table_name);
    let delete_method = generate_delete(table_name);
    let delete_by_id_method = generate_delete_by_id(table_name);
    
    quote! {
        #insert_method
        #update_method
        #select_method
        #select_by_id_method
        #delete_method
        #delete_by_id_method
    }
}

fn generate_newstruct_methods(struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let newstruct_insert = generate_newstruct_insert(struct_name);
    let newstruct_put = generate_newstruct_put(struct_name);

    quote! {
        #newstruct_insert
        #newstruct_put
    }
}

fn generate_from_row_impl(struct_name: &syn::Ident, model: &ModelInfo) -> proc_macro2::TokenStream {
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

fn generate_model_meta_impl(
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
                related_table: <#related_model as ::magic::meta::ModelMeta>::table,
                related_column: #related_column,
            }
        }
    });

    quote! {
        impl ::magic::meta::ModelMeta for #struct_name {
            fn table() -> &'static str {
                Self::TABLE
            }

            fn foreign_keys() -> &'static [::magic::meta::ForeignKeyMeta] {
                static FK_META: &[::magic::meta::ForeignKeyMeta] = &[
                    #( #fk_meta, )*
                ];
                FK_META
            }
        }
    }
}

           