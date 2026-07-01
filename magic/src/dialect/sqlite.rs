pub struct SqliteDialect;

use super::SqlDialect;

impl SqlDialect for SqliteDialect {
    fn placeholder(_index: usize) -> String {
        "?".to_string()
    }

    fn quote_identifier(name: &str) -> String {
        // SQLite accepts double-quoted identifiers (or just bare names).
        format!("\"{}\"", name)
    }

    fn insert_returning(table: &str, cols: &[&str], _pk: &str) -> String {
        let cols_joined = cols.iter().map(|c| format!("\"{}\"", c)).collect::<Vec<_>>().join(", ");
        let placeholders = vec!["?"; cols.len()].join(", ");
        format!("INSERT INTO \"{}\" ({}) VALUES ({})", table, cols_joined, placeholders)
    }

    fn last_insert_id_expr() -> Option<&'static str> {
        Some("last_insert_rowid()")
    }

    fn enable_foreign_keys() -> Option<&'static str> {
        Some("PRAGMA foreign_keys = ON;")
    }

    fn map_rust_type(rust_ty: &str) -> &'static str {
        match rust_ty {
            "i32" | "i64" | "u32" | "u64" => "INTEGER",
            "f32" | "f64" => "REAL",
            "String" => "TEXT",
            "bool" => "INTEGER",
            _ => "TEXT",
        }
    }
}
