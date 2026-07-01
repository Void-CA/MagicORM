use quote::quote;

pub fn generate_registry_method(struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    let descriptor_body = quote! {
        ::magic_orm::model::ModelDescriptor {
            table: Self::TABLE,
            columns: Self::columns(),
            foreign_keys: Self::foreign_keys(),
        }
    };

    quote! {
        // Inherent method (kept for backward compatibility)
        pub fn descriptor() -> ::magic_orm::model::ModelDescriptor {
            #descriptor_body
        }
    }
}

pub fn generate_describe_impl(struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        impl ::magic_orm::describe::Describe for #struct_name {
            fn descriptor() -> ::magic_orm::model::ModelDescriptor {
                #struct_name::descriptor()
            }
        }
    }
}
