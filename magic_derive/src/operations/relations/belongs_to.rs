use quote::{quote, format_ident};
use proc_macro2::TokenStream;
use syn::Ident;

pub fn generate_belongs_to(
    self_ident: &Ident,
    related_ident: &Ident,
    fk_field: &Ident,
) -> TokenStream {

    let method_name = format_ident!(
        "{}",
        related_ident.to_string().to_lowercase()
    );

    quote! {
        pub async fn #method_name(
            &self,
            pool: &sqlx::SqlitePool
        ) -> sqlx::Result<#related_ident> {

            ::magic::relations::load_belongs_to::<#related_ident>(
                pool,
                self.#fk_field
            ).await
        }
    }
}