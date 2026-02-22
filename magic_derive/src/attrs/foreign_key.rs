use syn::{Attribute, LitStr, Result};
use crate::model::{FieldInfo, ModelInfo};

pub struct FKConfig {
    pub table: String,
    pub column: String,
    pub on_delete: String,
}

pub fn parse_model_fks(model: &ModelInfo) -> Result<Vec<FKConfig>> {
    model.other_fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("FK")))
        .map(|f| parse_fk_attributes(f))
        .collect()
}

fn parse_fk_attributes(field: &FieldInfo) -> Result<FKConfig> {
    // Busca el atributo #[FK] en el campo
    let attr = field.attrs.iter()
        .find(|a| a.path().is_ident("FK"))
        .ok_or_else(|| syn::Error::new_spanned(&field.ident, "Missing #[FK] attribute"))?;

    parse_fk_attr(attr)
}

fn parse_fk_attr(attr: &Attribute) -> Result<FKConfig> {
    let mut table_name: Option<String> = None;
    let mut column_name: Option<String> = None;
    let mut on_delete: Option<String> = None;

    attr.parse_nested_meta(|meta| {
        let ident = meta.path.get_ident().ok_or_else(|| meta.error("Expected identifier"))?.to_string();

        match ident.as_str() {
            "table" => {
                if table_name.is_some() {
                    return Err(meta.error("Duplicate `table` argument"));
                }
                let value: LitStr = meta.value()?.parse()?;
                table_name = Some(value.value());
            },
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

    let table = table_name.ok_or_else(|| syn::Error::new_spanned(attr, "Missing required `table` argument"))?;
    let column = column_name.unwrap_or_else(|| "id".to_string());
    let on_delete = on_delete.unwrap_or_else(|| "CASCADE".to_string());

    Ok(FKConfig { table, column, on_delete })
}