use crate::input::attrs::FKConfig;
use quote::quote;

pub fn generate_hasfk_impl(fk_fields: &[FKConfig], struct_name: &syn::Ident) -> proc_macro2::TokenStream {
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
