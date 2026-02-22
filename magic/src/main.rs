use magic::MagicModel;
use sqlx::{SqlitePool};

#[derive(MagicModel, Debug)]
#[magic(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub edad: i32,
    pub email: String,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Base de datos en disco (archivo "test.db")
    let pool = SqlitePool::connect("sqlite://test.db").await?;

    create_db(&pool).await?;

    let users: Vec<User> = User::query()
    .filter("edad", ">", 30)
    .fetch_all(&pool)
    .await?;

    for user in users {
        println!("{:?}", user);
    }

    Ok(())
}



async fn create_db(pool : &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            edad INTEGER NOT NULL,
            email TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;


    let users= vec![
        User::new("Bob".into(), 33, "bob@example.com".into()),
        User::new("Carol".into(), 28, "carol@example.com".into()),
        User::new("Dave".into(), 40, "dave@example.com".into()),
    ];

    for user in users {
        user.insert(pool).await?;
    }

    Ok(())
}