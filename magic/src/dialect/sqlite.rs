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
        let cols_joined = cols
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = vec!["?"; cols.len()].join(", ");
        format!(
            "INSERT INTO \"{}\" ({}) VALUES ({})",
            table, cols_joined, placeholders
        )
    }

    fn upsert_returning(table: &str, cols: &[&str], pk: &str) -> String {
        let cols_joined = cols
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = vec!["?"; cols.len()].join(", ");
        let set_clause: Vec<String> = cols
            .iter()
            .map(|c| format!("\"{}\" = excluded.\"{}\"", c, c))
            .collect();
        format!(
            "INSERT INTO \"{}\" ({}) VALUES ({}) ON CONFLICT(\"{}\") DO UPDATE SET {}",
            table,
            cols_joined,
            placeholders,
            pk,
            set_clause.join(", "),
        )
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
            "Uuid" => "TEXT",
            _ => "TEXT",
        }
    }

    fn drop_foreign_key(
        table: &str,
        _fk: &crate::model::ForeignKeyMeta,
        columns: &[crate::model::ColumnMeta],
        foreign_keys: &[crate::model::ForeignKeyMeta],
    ) -> String {
        let qt = Self::quote_identifier(table);
        let old = format!("{}__old", table);
        let qold = Self::quote_identifier(&old);

        let col_names: Vec<String> = columns
            .iter()
            .map(|c| Self::quote_identifier(&c.name))
            .collect();

        let col_defs: Vec<String> = columns
            .iter()
            .map(|c| {
                let mut def = format!("    {} {}", Self::quote_identifier(&c.name), c.sql_type);
                if c.primary_key {
                    def.push_str(" PRIMARY KEY");
                }
                if c.auto_increment {
                    def.push_str(" AUTOINCREMENT");
                }
                if !c.nullable && !c.primary_key {
                    def.push_str(" NOT NULL");
                }
                def
            })
            .collect();

        let mut fk_defs: Vec<String> = Vec::new();
        for fk in foreign_keys {
            fk_defs.push(format!(
                "    FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE CASCADE",
                Self::quote_identifier(&fk.field),
                Self::quote_identifier(&fk.related_table),
                Self::quote_identifier(&fk.related_column),
            ));
        }

        let mut all_defs = col_defs;
        all_defs.extend(fk_defs);
        let create_body = all_defs.join(",\n");

        let select_cols = col_names.join(", ");

        format!(
            "PRAGMA foreign_keys=OFF;\n\
             ALTER TABLE {qt} RENAME TO {qold};\n\
             CREATE TABLE {qt} (\n{create_body}\n);\n\
             INSERT INTO {qt} ({select_cols}) SELECT {select_cols} FROM {qold};\n\
             DROP TABLE {qold};\n\
             PRAGMA foreign_keys=ON;",
        )
    }
}
