use crate::input::attrs::FKConfig;
use quote::quote;

pub fn generate_hasfk_impl(
    fk_fields: &[FKConfig],
    struct_name: &syn::Ident,
) -> proc_macro2::TokenStream {
    let fk_impls = fk_fields.iter().map(|fk| {
        let parent = &fk.model;
        let column_name = fk.field_ident.to_string();
        let field_ident = &fk.field_ident;

        quote! {
            impl magic_orm::relations::traits::HasFK<#parent> for #struct_name {
                fn fk_for_parent() -> &'static str {
                    #column_name
                }

                fn fk_value(&self) -> <#parent as magic_orm::model::Model>::Id {
                    self.#field_ident
                }
            }
        }
    });

    quote! {
        #( #fk_impls )*
    }
}