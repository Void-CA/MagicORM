use crate::input::parser::ModelInfo;
use quote::quote;

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
