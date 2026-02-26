use magic_orm::prelude::*;

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

#[derive(MagicModel, Debug)]
#[magic(table = "reactions")]
pub struct Reaction {
    pub id: i64,
    pub reaction_type: String,

    #[FK(Post)]
    pub post_id: i64,

    #[FK(User)]
    pub user_id: i64,
}

has_many!(User => Post, Reaction);
has_many!(Post => Reaction);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Base de datos en disco (archivo "test.db")
    let pool = SqlitePool::connect("sqlite://test.db").await?;
    create_db(&pool).await?;
    let user = User::query().filter("id", "=", 1).fetch_one(&pool).await?;
    let posts = user.posts(&pool).await?;
    let reactions = user.reactions(&pool).await?;
    println!("User: {:?}", user);
    println!("Posts: {:?}", posts);
    println!("Reactions: {:?}", reactions);
    Ok(())
}



use sqlx::{SqlitePool, Row};
use anyhow::Result;

pub async fn create_db(pool: &SqlitePool) -> Result<()> {
    // Crear tabla users
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            edad INTEGER NOT NULL,
            email TEXT NOT NULL
        );
        "#
    )
    .execute(pool)
    .await?;

    // Crear tabla posts
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            user_id INTEGER NOT NULL,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#
    )
    .execute(pool)
    .await?;

    // Crear tabla reactions
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS reactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            reaction_type TEXT NOT NULL,
            post_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            FOREIGN KEY(post_id) REFERENCES posts(id) ON DELETE CASCADE,
            FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#
    )
    .execute(pool)
    .await?;

    // Insertar usuarios de ejemplo
    let users = vec![
        ("Alice", 30, "alice@gmail.com"),
        ("Bob", 33, "bob@example.com"),
        ("Carol", 28, "carol@example.com"),
        ("Dave", 40, "dave@example.com"),
    ];

    for (name, edad, email) in users {
        sqlx::query("INSERT INTO users (name, edad, email) VALUES (?, ?, ?)")
            .bind(name)
            .bind(edad)
            .bind(email)
            .execute(pool)
            .await?;
    }

    // Insertar posts de ejemplo
    let posts = vec![
        ("First Post", "This is the content of the first post.", 1),
        ("Second Post", "This is the content of the second post.", 1),
        ("Another Post", "Carol's post content", 3),
    ];

    for (title, content, user_id) in posts {
        sqlx::query("INSERT INTO posts (title, content, user_id) VALUES (?, ?, ?)")
            .bind(title)
            .bind(content)
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    // Insertar reacciones de ejemplo
    let reactions = vec![
        ("Like", 1, 2),   // Bob le dio like al primer post de Alice
        ("Love", 1, 3),   // Carol reaccionó al primer post de Alice
        ("Wow", 3, 1),    // Alice reaccionó al post de Carol
    ];

    for (reaction_type, post_id, user_id) in reactions {
        sqlx::query("INSERT INTO reactions (reaction_type, post_id, user_id) VALUES (?, ?, ?)")
            .bind(reaction_type)
            .bind(post_id)
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}