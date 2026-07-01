use syn::{Attribute, DeriveInput, LitStr};

pub struct MagicConfig {
    pub table: String,
}

pub fn parse_magic_attributes(input: &DeriveInput) -> syn::Result<MagicConfig> {
    // Si hay #[magic(...)], parsearlo. Si no, inferir.
    match extract_magic_attribute(input) {
        Ok(attr) => parse_magic_attr(attr),
        Err(_) => Ok(MagicConfig {
            table: infer_table_name(&input.ident.to_string()),
        }),
    }
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
            "MagicModel requires #[magic(table = \"...\")] attribute (or omit it for auto-inference)",
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

    let table = match table_name {
        Some(t) => t,
        // Sintaxis bare: #[magic("users")]
        None => {
            // Intentar leer el primer argumento como string literal
            return Err(syn::Error::new_spanned(
                attr,
                "Missing `table` argument. Use #[magic(table = \"...\")] or omit #[magic] entirely",
            ));
        }
    };

    Ok(MagicConfig { table })
}

/// Infiere el nombre de tabla desde el nombre del struct.
/// User → users, PostCategory → post_categories
fn infer_table_name(struct_name: &str) -> String {
    let snake = camel_to_snake(struct_name);
    pluralize(&snake)
}

fn camel_to_snake(name: &str) -> String {
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

fn pluralize(word: &str) -> String {
    // Simple: agrega 's'. Casos comunes.
    if word.ends_with('s')
        || word.ends_with("sh")
        || word.ends_with("ch")
        || word.ends_with('x')
        || word.ends_with('z')
    {
        format!("{}es", word)
    } else if word.ends_with('y') && word.len() > 2
        && !"aeiou".contains(word.chars().nth(word.len() - 2).unwrap())
    {
        // consonant + y → ies
        format!("{}ies", &word[..word.len() - 1])
    } else if word.ends_with('f') {
        // f → ves (aproximado)
        format!("{}ves", &word[..word.len() - 1])
    } else {
        format!("{}s", word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_to_snake() {
        assert_eq!(camel_to_snake("User"), "user");
        assert_eq!(camel_to_snake("PostCategory"), "post_category");
        assert_eq!(camel_to_snake("HTMLParser"), "html_parser");
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize("user"), "users");
        assert_eq!(pluralize("post"), "posts");
        assert_eq!(pluralize("category"), "categories");
        assert_eq!(pluralize("box"), "boxes");
        assert_eq!(pluralize("address"), "addresses");
    }

    #[test]
    fn test_infer_table_name() {
        assert_eq!(infer_table_name("User"), "users");
        assert_eq!(infer_table_name("Post"), "posts");
        assert_eq!(infer_table_name("PostCategory"), "post_categories");
        assert_eq!(infer_table_name("Category"), "categories");
    }

    #[test]
    fn test_infer_table_name_uppercase() {
        assert_eq!(infer_table_name("HTMLParser"), "html_parsers");
    }
}
