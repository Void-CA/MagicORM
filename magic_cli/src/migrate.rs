use sqlx::SqlitePool;
use anyhow::Result;

pub async fn status(path: &Option<String>) -> Result<()> {
    // Ruta de la DB
    let db_path = match path {
        Some(p) => p.clone(),
        None => "magic.db".to_string(),
    };

    let url = format!("sqlite://{}", db_path);
    let pool = SqlitePool::connect(&url).await?;

    // Activar claves foráneas (opcional, pero consistente)
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    // Verificar existencia de tabla de migraciones
    let table_exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migrations';"
    )
    .fetch_one(&pool)
    .await?;

    if table_exists.0 == 0 {
        println!("La tabla de migraciones _migrations no existe. Inicializa la DB primero.");
        return Ok(());
    }

    // Leer migraciones aplicadas
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, name, applied_at FROM _migrations ORDER BY id;"
    )
    .fetch_all(&pool)
    .await?;

    println!("📊 Estado de migraciones en '{}':", db_path);
    if rows.is_empty() {
        println!("  No hay migraciones aplicadas todavía.");
    } else {
        for (id, name, applied_at) in rows {
            println!("  [{}] {} (aplicada en {})", id, name, applied_at);
        }
    }

    Ok(())
}