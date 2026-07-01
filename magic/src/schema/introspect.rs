use crate::model::{ColumnMeta, ForeignKeyMeta, ModelDescriptor};
use sqlx::Executor;

// =========================================================================
// Introspect — lee el esquema de una base de datos viva y produce
// ModelDescriptors compatibles con los generados por el derive.
// =========================================================================

/// Columna cruda desde PRAGMA table_info
#[derive(Debug, sqlx::FromRow)]
struct TableInfoRow {
    cid: i32,
    name: String,
    #[sqlx(rename = "type")]
    col_type: String,
    notnull: i32,   // SQLite devuelve 0/1 como entero
    dflt_value: Option<String>,
    pk: i32,        // Ídem
}

/// FK cruda desde PRAGMA foreign_key_list
#[derive(Debug, sqlx::FromRow)]
struct ForeignKeyRow {
    id: i32,
    seq: i32,
    table: String,
    from: String,
    to: String,
    on_update: Option<String>,
    on_delete: Option<String>,
}

/// Lista todas las tablas definidas por el usuario en la DB.
pub(crate) async fn list_tables<'e, E>(executor: E) -> anyhow::Result<Vec<String>>
where
    E: Executor<'e, Database = sqlx::Sqlite> + Copy,
{
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_migrations' ORDER BY name"
    )
    .fetch_all(executor)
    .await
    .map_err(|e| anyhow::anyhow!(e))?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Lee las columnas de una tabla vía PRAGMA table_info.
async fn table_columns<'e, E>(executor: E, table: &str) -> anyhow::Result<Vec<ColumnMeta>>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let rows: Vec<TableInfoRow> = sqlx::query_as(&format!("PRAGMA table_info(\"{}\")", table))
        .fetch_all(executor)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let sql_type = r.col_type.clone();
            ColumnMeta {
                name: Box::leak(r.name.into_boxed_str()),
                sql_type: Box::leak(r.col_type.into_boxed_str()),
                nullable: r.notnull == 0,
                primary_key: r.pk != 0,
                auto_increment: r.pk != 0 && sql_type.to_uppercase().contains("INT"),
            }
        })
        .collect())
}

/// Lee las foreign keys de una tabla vía PRAGMA foreign_key_list.
async fn table_foreign_keys<'e, E>(executor: E, table: &str) -> anyhow::Result<Vec<ForeignKeyMeta>>
where
    E: Executor<'e, Database = sqlx::Sqlite>,
{
    let rows: Vec<ForeignKeyRow> =
        sqlx::query_as(&format!("PRAGMA foreign_key_list(\"{}\")", table))
            .fetch_all(executor)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

    Ok(rows
        .into_iter()
        .map(|r| ForeignKeyMeta {
            field: Box::leak(r.from.into_boxed_str()),
            related_column: Box::leak(r.to.into_boxed_str()),
            related_table: Box::leak(r.table.into_boxed_str()),
        })
        .collect())
}

/// Describe todas las tablas definidas por el usuario en la base de datos.
/// Retorna Vec<ModelDescriptor> compatible con `all_descriptors()`.
pub async fn describe_database<'e, E>(executor: E) -> anyhow::Result<Vec<ModelDescriptor>>
where
    E: Executor<'e, Database = sqlx::Sqlite> + Copy,
{
    let tables = list_tables(executor).await?;
    let mut descriptors = Vec::with_capacity(tables.len());

    for table in &tables {
        let columns = table_columns(executor, table).await?;
        let foreign_keys = table_foreign_keys(executor, table).await?;

        descriptors.push(ModelDescriptor {
            table: Box::leak(table.clone().into_boxed_str()),
            columns: Box::leak(columns.into_boxed_slice()),
            foreign_keys: Box::leak(foreign_keys.into_boxed_slice()),
        });
    }

    Ok(descriptors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS posts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_describe_database_basic() {
        let pool = setup_db().await;
        let descriptors = describe_database(&pool).await.unwrap();

        // Should find both tables
        assert_eq!(descriptors.len(), 2);

        // Users table
        let users = descriptors.iter().find(|d| d.table == "users").unwrap();
        assert_eq!(users.columns.len(), 3);
        assert_eq!(users.columns[0].name, "id");
        assert!(users.columns[0].primary_key);
        assert!(users.columns[0].auto_increment);
        // SQLite PRAGMA table_info returns notnull=0 for INTEGER PRIMARY KEY
        // even though the column is effectively NOT NULL. We report what PRAGMA says.
        assert!(users.columns[0].nullable);
        assert_eq!(users.columns[1].name, "name");
        assert!(!users.columns[1].nullable);
        assert_eq!(users.columns[2].name, "email");
        assert!(users.columns[2].nullable);
        assert_eq!(users.foreign_keys.len(), 0);

        // Posts table
        let posts = descriptors.iter().find(|d| d.table == "posts").unwrap();
        assert_eq!(posts.columns.len(), 3);
        assert!(posts.columns[0].primary_key);
        assert_eq!(posts.foreign_keys.len(), 1);
        assert_eq!(posts.foreign_keys[0].field, "user_id");
        assert_eq!(posts.foreign_keys[0].related_table, "users");
        assert_eq!(posts.foreign_keys[0].related_column, "id");
    }
}
