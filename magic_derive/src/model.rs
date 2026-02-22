use syn::{DeriveInput, Data, Fields, Ident, Type};

pub struct FieldInfo {
    pub ident: Ident,
    pub ty: Type,
}

pub struct ModelInfo {
    pub id_field: FieldInfo,
    pub other_fields: Vec<FieldInfo>,
}

pub fn analyze_model(input: &DeriveInput) -> syn::Result<ModelInfo> {
    let data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "MagicModel can only be derived for structs",
            ))
        }
    };

    let fields = match &data.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "MagicModel requires named fields",
            ))
        }
    };

    let mut id_field = None;
    let mut other_fields = Vec::new();

    for field in fields {
        let ident = field.ident.clone().unwrap();
        let ty = field.ty.clone();

        let info = FieldInfo { ident: ident.clone(), ty };

        if ident == "id" {
            id_field = Some(info);
        } else {
            other_fields.push(info);
        }
    }

    let id_field = id_field.ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            "MagicModel requires a field named `id`",
        )
    })?;

    Ok(ModelInfo {
        id_field,
        other_fields,
    })
}