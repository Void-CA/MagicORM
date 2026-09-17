use syn::{Attribute, DeriveInput, LitStr, Token, punctuated::Punctuated};

pub struct MagicConfig {
    pub table: String,
    pub indexes: Vec<IndexConfig>,
}

pub struct IndexConfig {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

pub fn parse_magic_attributes(input: &DeriveInput) -> syn::Result<MagicConfig> {
    // Si hay #[magic(...)], parsearlo. Si no, inferir.
    let table = match extract_magic_attribute(input) {
        Ok(attr) => parse_magic_attr(attr)?,
        Err(_) => infer_table_name(&input.ident.to_string()),
    };

    let indexes = parse_index_attributes(input)?;

    Ok(MagicConfig { table, indexes })
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

fn parse_magic_attr(attr: &Attribute) -> syn::Result<String> {
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

    match table_name {
        Some(t) => Ok(t),
        // Sintaxis bare: #[magic("users")]
        None => {
            // Intentar leer el primer argumento como string literal
            Err(syn::Error::new_spanned(
                attr,
                "Missing `table` argument. Use #[magic(table = \"...\")] or omit #[magic] entirely",
            ))
        }
    }
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
    } else if word.ends_with('y')
        && word.len() > 2
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

fn parse_index_attributes(input: &DeriveInput) -> syn::Result<Vec<IndexConfig>> {
    let mut indexes = Vec::new();

    for attr in &input.attrs {
        if !attr.path().is_ident("magic") {
            continue;
        }

        // Parse #[magic(index(name = "...", columns = ["...", "..."]))]
        // Use a simpler approach: parse the token stream directly
        let tokens = attr.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)?;

        for meta in tokens {
            if let syn::Meta::List(meta_list) = meta {
                if meta_list.path.is_ident("index") {
                    let mut name = None;
                    let mut columns = None;
                    let mut unique = false;

                    // Parse the nested metas inside index(...)
                    let nested: Punctuated<syn::Meta, Token![,]> =
                        meta_list.parse_args_with(Punctuated::parse_terminated)?;

                    for nested_meta in nested {
                        if let syn::Meta::NameValue(nv) = nested_meta {
                            if nv.path.is_ident("name") {
                                if let syn::Expr::Lit(expr_lit) = nv.value {
                                    if let syn::Lit::Str(lit_str) = expr_lit.lit {
                                        name = Some(lit_str.value());
                                    }
                                }
                            } else if nv.path.is_ident("columns") {
                                if let syn::Expr::Array(arr) = nv.value {
                                    let mut cols = Vec::new();
                                    for elem in arr.elems {
                                        if let syn::Expr::Lit(expr_lit) = elem {
                                            if let syn::Lit::Str(lit_str) = expr_lit.lit {
                                                cols.push(lit_str.value());
                                            }
                                        }
                                    }
                                    columns = Some(cols);
                                }
                            } else if nv.path.is_ident("unique") {
                                if let syn::Expr::Lit(expr_lit) = nv.value {
                                    if let syn::Lit::Bool(lit_bool) = expr_lit.lit {
                                        unique = lit_bool.value;
                                    }
                                }
                            }
                        }
                    }

                    if let (Some(name), Some(columns)) = (name, columns) {
                        indexes.push(IndexConfig {
                            name,
                            columns,
                            unique,
                        });
                    }
                }
            }
        }
    }

    Ok(indexes)
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
