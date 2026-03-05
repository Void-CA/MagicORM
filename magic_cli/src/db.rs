use sqlx::SqlitePool;
use anyhow::Result;
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

    let url = format!("sqlite://{}", db_path);
    let pool = SqlitePool::connect(&url).await?;

    // Activar claves foráneas
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

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