use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn migrations_dir() -> PathBuf {
    PathBuf::from("migrations")
}

fn timestamp_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}", secs)
}

fn list_migration_files() -> Result<Vec<PathBuf>> {
    let dir = migrations_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut files: Vec<PathBuf> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sql"))
        .map(|e| e.path())
        .collect();

    files.sort();
    Ok(files)
}

fn parse_sql_file(path: &Path) -> Result<(String, String)> {
    let content = fs::read_to_string(path)?;
    let parts: Vec<&str> = content.split("-- DOWN").collect();

    let up = parts
        .first()
        .and_then(|s| s.split("-- UP").nth(1))
        .unwrap_or("")
        .trim()
        .to_string();

    let down = parts
        .get(1)
        .unwrap_or(&"")
        .trim()
        .to_string();

    Ok((up, down))
}

// ---------------------------------------------------------------------------
// Comandos
// ---------------------------------------------------------------------------

/// Crea una migración vacía.
pub fn new(name: &str) -> Result<()> {
    let dir = migrations_dir();
    fs::create_dir_all(&dir)?;

    let ts = timestamp_now();
    let filename = format!("{}_{}.sql", ts, name);
    let path = dir.join(&filename);

    let template = format!(
        "-- UP\n-- Escribe aquí el SQL de migración\n\n\n-- DOWN\n-- Escribe aquí el SQL de rollback\n"
    );

    fs::write(&path, template)?;
    println!("Creada: {}", path.display());
    Ok(())
}

/// Genera una migración desde diff de modelos (stub — requiere setup adicional).
pub async fn generate(_name: &str) -> Result<()> {
    println!("❌ migrate generate requiere conectar el pipeline de descriptors.");
    println!("   Por ahora usa `magic migrate new <name>` y escribe el SQL manualmente.");
    println!("   Próximamente: all_descriptors() + describe_database() + diff + render.");
    Ok(())
}

/// Aplica migraciones pendientes.
pub async fn up(db_path: &str) -> Result<()> {
    let url = format!("sqlite://{}", db_path);
    let pool = sqlx::SqlitePool::connect(&url).await?;

    // Asegurar tabla _migrations
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    // Migraciones ya aplicadas
    let applied: Vec<String> = sqlx::query_as::<_, (String,)>(
        "SELECT name FROM _migrations ORDER BY name"
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|r| r.0)
    .collect();

    let files = list_migration_files()?;
    let pending: Vec<&PathBuf> = files.iter()
        .filter(|f| !applied.contains(&f.file_stem().unwrap().to_string_lossy().to_string()))
        .collect();

    if pending.is_empty() {
        println!("✓ No hay migraciones pendientes.");
        return Ok(());
    }

    for file in &pending {
        let name = file.file_stem().unwrap().to_string_lossy().to_string();
        println!("▶ Aplicando: {}", name);

        let (up_sql, _) = parse_sql_file(file)?;
        if !up_sql.is_empty() {
            for stmt in up_sql.split(';') {
                let stmt = stmt.trim();
                if !stmt.is_empty() {
                    sqlx::query(stmt).execute(&pool).await
                        .with_context(|| format!("Error ejecutando:\n{}", stmt))?;
                }
            }
        }

        sqlx::query("INSERT INTO _migrations (name, applied_at) VALUES (?, datetime('now'))")
            .bind(&name)
            .execute(&pool)
            .await?;

        println!("✓ {} aplicada.", name);
    }

    Ok(())
}

/// Revierte la última migración.
pub async fn down(db_path: &str) -> Result<()> {
    let url = format!("sqlite://{}", db_path);
    let pool = sqlx::SqlitePool::connect(&url).await?;

    let last: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM _migrations ORDER BY id DESC LIMIT 1"
    )
    .fetch_optional(&pool)
    .await?;

    let (last_name,) = match last {
        Some(r) => r,
        None => {
            println!("✓ No hay migraciones para revertir.");
            return Ok(());
        }
    };

    // Buscar el archivo .sql
    let files = list_migration_files()?;
    let file = files.iter().find(|f| {
        f.file_stem().unwrap().to_string_lossy().starts_with(&last_name[..20])
    });

    match file {
        Some(path) => {
            let (_, down_sql) = parse_sql_file(path)?;
            if !down_sql.is_empty() {
                for stmt in down_sql.split(';') {
                    let stmt = stmt.trim();
                    if !stmt.is_empty() {
                        sqlx::query(stmt).execute(&pool).await
                            .with_context(|| format!("Error ejecutando rollback:\n{}", stmt))?;
                    }
                }
            }

            sqlx::query("DELETE FROM _migrations WHERE name = ?")
                .bind(&last_name)
                .execute(&pool)
                .await?;

            println!("✓ Revertida: {}", last_name);
        }
        None => {
            // Si no hay archivo, igual removemos el registro
            sqlx::query("DELETE FROM _migrations WHERE name = ?")
                .bind(&last_name)
                .execute(&pool)
                .await?;
            println!("⚠ Revertida (sin archivo): {}", last_name);
        }
    }

    Ok(())
}

/// Muestra el estado de las migraciones.
pub async fn status(db_path: &str) -> Result<()> {
    let url = format!("sqlite://{}", db_path);
    let pool = sqlx::SqlitePool::connect(&url).await?;

    // Activar claves foráneas
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    // Verificar existencia de tabla _migrations
    let table_exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migrations';"
    )
    .fetch_one(&pool)
    .await?;

    if table_exists.0 == 0 {
        println!("⚠ La tabla de migraciones _migrations no existe. Inicializa la DB primero.");
        return Ok(());
    }

    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, name, applied_at FROM _migrations ORDER BY id;"
    )
    .fetch_all(&pool)
    .await?;

    let files = list_migration_files()?;
    let applied_names: Vec<String> = rows.iter().map(|r| r.1.clone()).collect();

    println!("📊 Migraciones en '{}':", db_path);
    if rows.is_empty() {
        println!("  No hay migraciones aplicadas.");
    } else {
        for (id, name, applied_at) in &rows {
            let file_exists = files.iter().any(|f| {
                f.file_stem().unwrap().to_string_lossy().starts_with(&name[..20])
            });
            let marker = if file_exists { "✓" } else { "⚠" };
            println!("  [{}] {} {} (aplicada en {})", id, marker, name, applied_at);
        }
    }

    // Mostrar pendientes
    for file in &files {
        let name = file.file_stem().unwrap().to_string_lossy().to_string();
        if !applied_names.contains(&name) {
            println!("  · pendiente: {}", name);
        }
    }

    Ok(())
}
