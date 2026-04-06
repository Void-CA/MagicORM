use quote::quote;

pub fn generate_registry_method(_struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn descriptor() -> ::magic_orm::model::ModelDescriptor {
            ::magic_orm::model::ModelDescriptor {
                table: Self::TABLE,
                columns: Self::columns(),
                foreign_keys: Self::foreign_keys(),
            }
        }
    }
}
