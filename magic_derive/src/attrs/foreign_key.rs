use syn::{Attribute, LitStr, Result};
use crate::model::{FieldInfo, ModelInfo};

pub struct FKConfig {
    pub model: syn::Ident,        // User
    pub field_ident: syn::Ident,  // user_id
    pub column: String,           // default "id"
    pub on_delete: String,        // default "CASCADE"
}

pub fn parse_model_fks(model: &ModelInfo) -> Result<Vec<FKConfig>> {
    model.other_fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("FK")))
        .map(|f| parse_fk_attributes(f))
        .collect()
}

fn parse_fk_attributes(field: &FieldInfo) -> Result<FKConfig> {
    let attr = field.attrs.iter()
        .find(|a| a.path().is_ident("FK"))
        .ok_or_else(|| syn::Error::new_spanned(&field.ident, "Missing #[FK] attribute"))?;

    parse_fk_attr(attr, &field.ident)
}

fn parse_fk_attr(attr: &Attribute, field_ident: &syn::Ident) -> Result<FKConfig> {
    let mut model_ident: Option<syn::Ident> = None;
    let mut column_name: Option<String> = None;
    let mut on_delete: Option<String> = None;

    attr.parse_nested_meta(|meta| {
        // Caso 1: el primer argumento es el modelo: #[FK(User)]
        if model_ident.is_none() && meta.path.get_ident().is_some() && meta.input.is_empty() {
            model_ident = Some(meta.path.get_ident().unwrap().clone());
            return Ok(());
        }

        // Caso 2: argumentos nombrados
        let ident = meta.path.get_ident()
            .ok_or_else(|| meta.error("Expected identifier"))?
            .to_string();

        match ident.as_str() {
            "column" => {
                if column_name.is_some() {
                    return Err(meta.error("Duplicate `column` argument"));
                }
                let value: LitStr = meta.value()?.parse()?;
                column_name = Some(value.value());
            },
            "on_delete" => {
                if on_delete.is_some() {
                    return Err(meta.error("Duplicate `on_delete` argument"));
                }
                let value: LitStr = meta.value()?.parse()?;
                on_delete = Some(value.value());
            },
            _ => return Err(meta.error("Unsupported FK attribute argument")),
        }

        Ok(())
    })?;

    let model = model_ident
        .ok_or_else(|| syn::Error::new_spanned(attr, "Missing related model in #[FK(...)]"))?;

    Ok(FKConfig {
        model,
        field_ident: field_ident.clone(),
        column: column_name.unwrap_or_else(|| "id".to_string()),
        on_delete: on_delete.unwrap_or_else(|| "CASCADE".to_string()),
    })
}