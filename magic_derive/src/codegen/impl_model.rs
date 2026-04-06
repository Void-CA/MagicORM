use crate::input::ModelInfo;
use quote::quote;

pub fn generate_model_impl(struct_name: &syn::Ident, model: &ModelInfo) -> proc_macro2::TokenStream {
    let id_type = &model.id_field.ty;

    quote! {
        impl ::magic_orm::model::Model for #struct_name {
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
