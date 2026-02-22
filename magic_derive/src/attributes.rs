use syn::{DeriveInput, LitStr};

pub struct MagicConfig {
    pub table: String,
}

pub fn parse_magic_attributes(input: &DeriveInput) -> syn::Result<MagicConfig> {
    let mut table_name: Option<String> = None;

    for attr in &input.attrs {
        if attr.path().is_ident("magic") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("table") {
                    let value: LitStr = meta.value()?.parse()?;
                    table_name = Some(value.value());
                    Ok(())
                } else {
                    Err(meta.error("Unsupported magic attribute argument"))
                }
            })?;
        }
    }

    let table = table_name.ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            "MagicModel requires #[magic(table = \"...\")] attribute",
        )
    })?;

    Ok(MagicConfig { table })
}