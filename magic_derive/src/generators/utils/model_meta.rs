pub fn is_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

pub fn extract_inner_type(ty: &syn::Type) -> &syn::Type {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return inner_ty;
                    }
                }
            }
        }
    }

    ty
}

pub fn map_rust_to_sqlite(ty: &syn::Type) -> &'static str {
    let ty = extract_inner_type(ty);

    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = segment.ident.to_string();

            return match ident.as_str() {
                "i32" | "i64" | "u32" | "u64" => "INTEGER",
                "f32" | "f64" => "REAL",
                "String" => "TEXT",
                "bool" => "INTEGER",
                "Vec" => "BLOB",
                _ => "TEXT", // fallback razonable
            };
        }
    }

    "TEXT"
}