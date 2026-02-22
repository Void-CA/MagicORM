use quote::quote;
use proc_macro2::TokenStream;
use syn::Ident;

pub fn generate_has_many(
    parent_ident: &Ident,
    child_ident: &Ident,
    fk_column: &str,
) -> TokenStream {
    let method_name = syn::Ident::new(
        &format!("{}s", child_ident.to_string().to_lowercase()),
        child_ident.span(),
    );

    quote! {
        pub async fn #method_name(
            &self,
            pool: &sqlx::SqlitePool
        ) -> sqlx::Result<Vec<#child_ident>> {

            magic::relations::load_has_many::<#parent_ident, #child_ident>(
                self,
                pool,
                #fk_column
            ).await
        }
    }
}