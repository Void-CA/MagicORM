use anyhow::Result;
use magic_orm::sqlite::Sqlite;
use std::path::Path;

pub async fn init(path: &Option<String>) -> Result<()> {
    let db_path = match path {
        Some(p) => p.clone(),
        None => "magic.db".to_string(),
    };

    // Crear directorio si no existe
    if let Some(parent) = Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let pool = Sqlite::pool(&db_path).await?;

    println!("Base de datos inicializada en '{}'", db_path);

    // Crear tabla de migraciones si no existe
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(())
}
