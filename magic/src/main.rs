use magic::MagicModel;
use sqlx::SqlitePool;

#[derive(MagicModel)]
#[magic(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Base de datos en memoria
    let pool = SqlitePool::connect(":memory:").await?;

    // Crear tabla
    sqlx::query(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    // Instancia de NewUser
    let new_user = NewUser {
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };

    // Insertar
    let id = User::new(new_user.name, new_user.email)
        .insert(&pool)
        .await?;

    println!("User inserted with id: {}", id);

    Ok(())
}