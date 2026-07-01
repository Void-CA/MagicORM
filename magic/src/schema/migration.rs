use std::collections::HashSet;

use crate::dialect::SqlDialect;
use crate::model::{ColumnMeta, ForeignKeyMeta, ModelDescriptor};

// =========================================================================
// MigrationStep — una operación atómica de migración
// =========================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum MigrationStep {
    CreateTable {
        table: String,
        columns: Vec<ColumnMeta>,
        foreign_keys: Vec<ForeignKeyMeta>,
    },
    DropTable {
        table: String,
    },
    AddColumn {
        table: String,
        column: ColumnMeta,
    },
    DropColumn {
        table: String,
        column: String,
    },
    AddForeignKey {
        table: String,
        fk: ForeignKeyMeta,
    },
    DropForeignKey {
        table: String,
        fk: ForeignKeyMeta,
    },
}

// =========================================================================
// Diff — compara dos conjuntos de descriptors y produce pasos de migración
// =========================================================================

/// Compara el esquema deseado (from models) contra el actual (from DB)
/// y produce los pasos necesarios para que actual → deseado.
pub fn diff(desired: &[ModelDescriptor], actual: &[ModelDescriptor]) -> Vec<MigrationStep> {
    let mut steps = Vec::new();

    let desired_tables: HashSet<&str> = desired.iter().map(|d| d.table).collect();
    let actual_tables: HashSet<&str> = actual.iter().map(|d| d.table).collect();

    // Tablas nuevas: CREATE TABLE
    for desc in desired.iter().filter(|d| !actual_tables.contains(d.table)) {
        steps.push(MigrationStep::CreateTable {
            table: desc.table.to_string(),
            columns: desc.columns.to_vec(),
            foreign_keys: desc.foreign_keys.to_vec(),
        });
    }

    // Tablas eliminadas: DROP TABLE
    for desc in actual.iter().filter(|d| !desired_tables.contains(d.table)) {
        steps.push(MigrationStep::DropTable {
            table: desc.table.to_string(),
        });
    }

    // Tablas existentes: diff columnas y FKs
    for desired_desc in desired.iter() {
        let Some(actual_desc) = actual.iter().find(|d| d.table == desired_desc.table) else {
            continue; // ya se manejó como CreateTable
        };

        let desired_cols: HashSet<&str> = desired_desc.columns.iter().map(|c| c.name).collect();
        let actual_cols: HashSet<&str> = actual_desc.columns.iter().map(|c| c.name).collect();

        // Columnas nuevas
        for col in desired_desc.columns.iter().filter(|c| !actual_cols.contains(c.name)) {
            steps.push(MigrationStep::AddColumn {
                table: desired_desc.table.to_string(),
                column: col.clone(),
            });
        }

        // Columnas eliminadas
        for col in actual_desc.columns.iter().filter(|c| !desired_cols.contains(c.name)) {
            steps.push(MigrationStep::DropColumn {
                table: desired_desc.table.to_string(),
                column: col.name.to_string(),
            });
        }

        // FKs nuevas
        let actual_fks: HashSet<(&str, &str)> = actual_desc
            .foreign_keys
            .iter()
            .map(|fk| (fk.field, fk.related_table))
            .collect();

        for fk in desired_desc.foreign_keys.iter() {
            if !actual_fks.contains(&(fk.field, fk.related_table)) {
                steps.push(MigrationStep::AddForeignKey {
                    table: desired_desc.table.to_string(),
                    fk: fk.clone(),
                });
            }
        }

        // FKs eliminadas
        let desired_fks: HashSet<(&str, &str)> = desired_desc
            .foreign_keys
            .iter()
            .map(|fk| (fk.field, fk.related_table))
            .collect();

        for fk in actual_desc.foreign_keys.iter() {
            if !desired_fks.contains(&(fk.field, fk.related_table)) {
                steps.push(MigrationStep::DropForeignKey {
                    table: desired_desc.table.to_string(),
                    fk: fk.clone(),
                });
            }
        }
    }

    steps
}

// =========================================================================
// Render SQL — convierte MigrationStep a SQL según el dialecto
// =========================================================================

/// Convierte un MigrationStep a SQL usando el dialecto indicado.
pub fn render_step<D: SqlDialect>(step: &MigrationStep) -> String {
    match step {
        MigrationStep::CreateTable { table, columns, foreign_keys } => {
            let mut sql = format!("CREATE TABLE {} (\n", D::quote_identifier(table));
            let mut defs: Vec<String> = columns
                .iter()
                .map(|col| format_column_def::<D>(col))
                .collect();

            for fk in foreign_keys {
                defs.push(format!(
                    "    FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE CASCADE",
                    D::quote_identifier(&fk.field),
                    D::quote_identifier(&fk.related_table),
                    D::quote_identifier(&fk.related_column),
                ));
            }

            sql.push_str(&defs.join(",\n"));
            sql.push_str("\n);");
            sql
        }

        MigrationStep::DropTable { table } => {
            format!("DROP TABLE IF EXISTS {};", D::quote_identifier(table))
        }

        MigrationStep::AddColumn { table, column } => {
            let col_def = format_column_def::<D>(column);
            format!(
                "ALTER TABLE {} ADD COLUMN {};",
                D::quote_identifier(table),
                col_def,
            )
        }

        MigrationStep::DropColumn { table, column } => {
            format!(
                "ALTER TABLE {} DROP COLUMN {};",
                D::quote_identifier(table),
                D::quote_identifier(column),
            )
        }

        MigrationStep::AddForeignKey { table, fk } => {
            format!(
                "ALTER TABLE {} ADD FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE CASCADE;",
                D::quote_identifier(table),
                D::quote_identifier(&fk.field),
                D::quote_identifier(&fk.related_table),
                D::quote_identifier(&fk.related_column),
            )
        }

        MigrationStep::DropForeignKey { table, fk } => {
            // SQLite no soporta DROP FOREIGN KEY directamente.
            // Postgres: ALTER TABLE ... DROP CONSTRAINT ...
            // Por ahora emitimos un comentario con la FK a eliminar.
            format!(
                "-- TODO: DROP FOREIGN KEY {} REFERENCES {} ({}) ON TABLE {}",
                fk.field, fk.related_table, fk.related_column, table,
            )
        }
    }
}

/// Renderiza una secuencia completa de pasos como SQL (separados por \n\n).
pub fn render_migration<D: SqlDialect>(steps: &[MigrationStep]) -> String {
    steps
        .iter()
        .map(|s| render_step::<D>(s))
        .collect::<Vec<_>>()
        .join("\n\n")
}

// =========================================================================
// Helpers internos
// =========================================================================

fn format_column_def<D: SqlDialect>(col: &ColumnMeta) -> String {
    let mut def = format!("    {} {}", D::quote_identifier(col.name), col.sql_type);
    if col.primary_key {
        def.push_str(" PRIMARY KEY");
    }
    if col.auto_increment {
        def.push_str(" AUTOINCREMENT");
    }
    if !col.nullable && !col.primary_key {
        def.push_str(" NOT NULL");
    }
    def
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::SqliteDialect;

    fn user_descriptor() -> ModelDescriptor {
        ModelDescriptor {
            table: "users",
            columns: &[
                ColumnMeta { name: "id", sql_type: "INTEGER", nullable: false, primary_key: true, auto_increment: true },
                ColumnMeta { name: "name", sql_type: "TEXT", nullable: false, primary_key: false, auto_increment: false },
                ColumnMeta { name: "email", sql_type: "TEXT", nullable: true, primary_key: false, auto_increment: false },
            ],
            foreign_keys: &[],
        }
    }

    fn post_descriptor() -> ModelDescriptor {
        ModelDescriptor {
            table: "posts",
            columns: &[
                ColumnMeta { name: "id", sql_type: "INTEGER", nullable: false, primary_key: true, auto_increment: true },
                ColumnMeta { name: "title", sql_type: "TEXT", nullable: false, primary_key: false, auto_increment: false },
                ColumnMeta { name: "user_id", sql_type: "INTEGER", nullable: false, primary_key: false, auto_increment: false },
            ],
            foreign_keys: &[
                ForeignKeyMeta { field: "user_id", related_table: "users", related_column: "id" },
            ],
        }
    }

    #[test]
    fn test_diff_create_table() {
        let desired = vec![user_descriptor()];
        let actual = vec![];
        let steps = diff(&desired, &actual);
        assert_eq!(steps.len(), 1);
        assert!(matches!(&steps[0], MigrationStep::CreateTable { table, .. } if table == "users"));
    }

    #[test]
    fn test_diff_drop_table() {
        let desired = vec![];
        let actual = vec![user_descriptor()];
        let steps = diff(&desired, &actual);
        assert_eq!(steps.len(), 1);
        assert!(matches!(&steps[0], MigrationStep::DropTable { table, .. } if table == "users"));
    }

    #[test]
    fn test_diff_add_column() {
        let mut user = user_descriptor();
        user.columns = &[
            ColumnMeta { name: "id", sql_type: "INTEGER", nullable: false, primary_key: true, auto_increment: true },
            ColumnMeta { name: "name", sql_type: "TEXT", nullable: false, primary_key: false, auto_increment: false },
            ColumnMeta { name: "email", sql_type: "TEXT", nullable: true, primary_key: false, auto_increment: false },
            ColumnMeta { name: "age", sql_type: "INTEGER", nullable: true, primary_key: false, auto_increment: false },
        ];

        let desired = vec![user];
        let actual = vec![user_descriptor()];
        let steps = diff(&desired, &actual);
        assert_eq!(steps.len(), 1);
        assert!(matches!(&steps[0], MigrationStep::AddColumn { column, .. } if column.name == "age"));
    }

    #[test]
    fn test_diff_drop_column() {
        let mut user = user_descriptor();
        user.columns = &[
            ColumnMeta { name: "id", sql_type: "INTEGER", nullable: false, primary_key: true, auto_increment: true },
            ColumnMeta { name: "name", sql_type: "TEXT", nullable: false, primary_key: false, auto_increment: false },
            // email fue eliminado
        ];

        let desired = vec![user];
        let actual = vec![user_descriptor()];
        let steps = diff(&desired, &actual);
        assert_eq!(steps.len(), 1);
        assert!(matches!(&steps[0], MigrationStep::DropColumn { column, .. } if column == "email"));
    }

    #[test]
    fn test_diff_add_foreign_key() {
        let desired = vec![post_descriptor()];
        let actual = vec![ModelDescriptor {
            table: "posts",
            columns: post_descriptor().columns,
            foreign_keys: &[], // sin FK
        }];
        let steps = diff(&desired, &actual);
        assert_eq!(steps.len(), 1);
        assert!(matches!(&steps[0], MigrationStep::AddForeignKey { .. }));
    }

    #[test]
    fn test_diff_no_changes() {
        let desired = vec![user_descriptor()];
        let actual = vec![user_descriptor()];
        let steps = diff(&desired, &actual);
        assert!(steps.is_empty());
    }

    #[test]
    fn test_diff_multiple_changes() {
        // desired tiene users + posts, actual solo tiene users sin FK
        let mut user_no_email = user_descriptor();
        user_no_email.columns = &user_no_email.columns[..2]; // sacamos email

        let actual = vec![user_no_email];
        let desired = vec![user_descriptor(), post_descriptor()];

        let steps = diff(&desired, &actual);
        // 2 cambios: add column email, create table posts (FK incluida en CREATE TABLE)
        assert_eq!(steps.len(), 2);
    }

    // =====================================================================
    // Render SQL
    // =====================================================================

    #[test]
    fn test_render_create_table() {
        let step = MigrationStep::CreateTable {
            table: "users".to_string(),
            columns: vec![
                ColumnMeta { name: "id", sql_type: "INTEGER", nullable: false, primary_key: true, auto_increment: true },
                ColumnMeta { name: "name", sql_type: "TEXT", nullable: false, primary_key: false, auto_increment: false },
            ],
            foreign_keys: vec![],
        };
        let sql = render_step::<SqliteDialect>(&step);
        assert!(sql.starts_with("CREATE TABLE"));
        assert!(sql.contains("\"id\" INTEGER PRIMARY KEY AUTOINCREMENT"));
        assert!(sql.contains("\"name\" TEXT NOT NULL"));
    }

    #[test]
    fn test_render_drop_table() {
        let sql = render_step::<SqliteDialect>(&MigrationStep::DropTable { table: "users".to_string() });
        assert_eq!(sql, "DROP TABLE IF EXISTS \"users\";");
    }

    #[test]
    fn test_render_add_column() {
        let step = MigrationStep::AddColumn {
            table: "users".to_string(),
            column: ColumnMeta { name: "age", sql_type: "INTEGER", nullable: true, primary_key: false, auto_increment: false },
        };
        let sql = render_step::<SqliteDialect>(&step);
        assert!(sql.contains("ALTER TABLE"));
        assert!(sql.contains("ADD COLUMN"));
        assert!(sql.contains("\"age\" INTEGER"));
    }

    #[test]
    fn test_render_drop_column() {
        let sql = render_step::<SqliteDialect>(&MigrationStep::DropColumn { table: "users".to_string(), column: "email".to_string() });
        assert!(sql.contains("DROP COLUMN"));
        assert!(sql.contains("email"));
    }

    #[test]
    fn test_render_add_foreign_key() {
        let step = MigrationStep::AddForeignKey {
            table: "posts".to_string(),
            fk: ForeignKeyMeta { field: "user_id", related_table: "users", related_column: "id" },
        };
        let sql = render_step::<SqliteDialect>(&step);
        assert!(sql.contains("ADD FOREIGN KEY"));
        assert!(sql.contains("user_id"));
        assert!(sql.contains("users"));
    }

    #[test]
    fn test_render_migration() {
        let steps = vec![
            MigrationStep::CreateTable {
                table: "users".to_string(),
                columns: vec![
                    ColumnMeta { name: "id", sql_type: "INTEGER", nullable: false, primary_key: true, auto_increment: true },
                    ColumnMeta { name: "name", sql_type: "TEXT", nullable: false, primary_key: false, auto_increment: false },
                ],
                foreign_keys: vec![],
            },
            MigrationStep::AddColumn {
                table: "users".to_string(),
                column: ColumnMeta { name: "age", sql_type: "INTEGER", nullable: true, primary_key: false, auto_increment: false },
            },
        ];
        let sql = render_migration::<SqliteDialect>(&steps);
        assert!(sql.contains("CREATE TABLE"));
        assert!(sql.contains("ADD COLUMN"));
        // steps separated by blank line
        assert!(sql.contains("\n\n"));
    }
}
