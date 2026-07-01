pub struct PostgresDialect;

use super::SqlDialect;

impl SqlDialect for PostgresDialect {
    fn placeholder(index: usize) -> String {
        format!("${}", index)
    }

    fn quote_identifier(name: &str) -> String {
        format!("\"{}\"", name.to_lowercase())
    }

    fn insert_returning(table: &str, cols: &[&str], pk: &str) -> String {
        let cols_joined = cols.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(", ");
        let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("${}", i)).collect();
        let placeholders = placeholders.join(", ");
        format!(
            "INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING \"{}\"",
            table, cols_joined, placeholders, pk
        )
    }

    fn last_insert_id_expr() -> Option<&'static str> {
        None // RETURNING covers this
    }

    fn enable_foreign_keys() -> Option<&'static str> {
        None // Postgres has FK enforcement on by default
    }

    fn map_rust_type(rust_ty: &str) -> &'static str {
        match rust_ty {
            "i32" => "INTEGER",
            "i64" => "BIGINT",
            "u32" | "u64" => "NUMERIC(20)",
            "f32" => "REAL",
            "f64" => "DOUBLE PRECISION",
            "String" => "TEXT",
            "bool" => "BOOLEAN",
            "Uuid" => "UUID",
            _ => "TEXT",
        }
    }
}
