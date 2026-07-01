use crate::input::FKConfig;
use quote::quote;

pub fn generate_belongs_to_impls(fk_fields: &[FKConfig], struct_name: &syn::Ident) -> proc_macro2::TokenStream {
let belongs_to_impls = fk_fields.iter().map(|fk| {
    let field_ident = &fk.field_ident;
    let parent = &fk.model;
    let method_name = syn::Ident::new(&parent.to_string().to_lowercase(), field_ident.span());

    quote! {
            pub async fn #method_name<'e>(&self, executor: impl sqlx::Executor<'e, Database = <Self as ::magic_orm::model::Model>::DB>) -> anyhow::Result<#parent>
            {
                let id = self.#field_ident;
                ::magic_orm::relations::load_belongs_to::<#parent>(executor, id).await
            }
        }
});
    quote! {
        #( #belongs_to_impls )*
    }
}