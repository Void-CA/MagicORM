use magic_orm::{prelude::*, register_models};

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

register_models!(User, Post, Reaction);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Base de datos en disco (archivo "test.db")
    let pool = SqlitePool::connect("sqlite://test.db").await?;
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    create_all::<SqlitePool, AppModels>(&pool).await?;
    Ok(())
}

