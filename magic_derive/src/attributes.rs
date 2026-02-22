use syn::{Attribute, DeriveInput, LitStr};

pub struct MagicConfig {
    pub table: String,
}

pub fn parse_magic_attributes(input: &DeriveInput) -> syn::Result<MagicConfig> {
    let attr = extract_magic_attribute(input)?;
    parse_magic_attr(attr)
}

fn extract_magic_attribute(input: &DeriveInput) -> syn::Result<&Attribute> {
    let mut found: Option<&Attribute> = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("magic") {
            continue;
        }

        if found.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "Duplicate #[magic(...)] attribute",
            ));
        }

        found = Some(attr);
    }

    found.ok_or_else(|| {
        syn::Error::new_spanned(
            &input.ident,
            "MagicModel requires #[magic(table = \"...\")] attribute",
        )
    })
}


fn parse_magic_attr(attr: &Attribute) -> syn::Result<MagicConfig> {
    let mut table_name: Option<String> = None;

    attr.parse_nested_meta(|meta| {
        if !meta.path.is_ident("table") {
            return Err(meta.error("Unsupported magic attribute argument"));
        }

        if table_name.is_some() {
            return Err(meta.error("Duplicate `table` argument"));
        }

        let value: LitStr = meta.value()?.parse()?;
        table_name = Some(value.value());

        Ok(())
    })?;

    let table = table_name.ok_or_else(|| {
        syn::Error::new_spanned(
            attr,
            "Missing required `table` argument",
        )
    })?;

    Ok(MagicConfig { table })
}