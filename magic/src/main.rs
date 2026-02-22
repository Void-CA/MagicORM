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
    // Base de datos en disco (archivo "test.db")
    let pool = SqlitePool::connect("sqlite://test.db").await?;

    // Crear tabla si no existe
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    // Instancia de NewUser
    let user_1 = User::new("Alice".into(), "alice@example.com".into());
    let user_2 = User::new("Bob".into(), "bob@example.com".into());
    let user_3 = User::new("Charlie".into(), "charlie@example.com".into());

    // Insertar
    let id = user_1.insert(&pool).await?;
    user_2.insert(&pool).await?;
    user_3.insert(&pool).await?;
    println!("User inserted with id: {}", id);

    // Actualizar
    let new_put_user = User::new("Alice".into(), "alice.updated@example.com".into());
    let rows_affected = new_put_user.update(&pool, id).await?;
    println!("User updated with rows affected: {}", rows_affected);

    Ok(())
}