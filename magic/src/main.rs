use magic::{MagicModel};
use sqlx::{SqlitePool};
use magic::has_many;

#[derive(MagicModel, Debug)]
#[magic(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub edad: i32,
    pub email: String,
}


#[derive(MagicModel, Debug)]
#[magic(table = "posts")]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,

    #[FK(User)]
    pub user_id: i64,
}

has_many!(User => Post);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Base de datos en disco (archivo "test.db")
    let pool = SqlitePool::connect("sqlite://test.db").await?;

    create_db(&pool).await?;

    Ok(())
}



async fn create_db(pool : &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            edad INTEGER NOT NULL,
            email TEXT NOT NULL
        );"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            user_id INTEGER NOT NULL
        );"
    ).execute(pool).await?;


    let users= vec![
        User::new("Bob".into(), 33, "bob@example.com".into()),
        User::new("Carol".into(), 28, "carol@example.com".into()),
        User::new("Dave".into(), 40, "dave@example.com".into()),
    ];

    let user_x = User::new("Alice".into(), 30, "alice@gmail.com".into());
    let id_user_x = user_x.insert(pool).await?;

    let posts = vec![
        Post::new("First Post".into(), "This is the content of the first post.".into(), id_user_x),
        Post::new("Second Post".into(), "This is the content of the second post.".into(), id_user_x),
    ];

    for post in posts {
        post.insert(pool).await?;
    }

    for user in users {
        user.insert(pool).await?;
    }

    Ok(())
}